use crate::t_dam;
use crate::t_sqlite;
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyGroup {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: i64,
    pub tag_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyTag {
    pub id: i64,
    pub name: String,
    pub group_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomySnapshot {
    pub groups: Vec<TaxonomyGroup>,
    pub tags: Vec<TaxonomyTag>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyGroupInput {
    pub id: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxonomyTagInput {
    pub tag_id: i64,
    pub group_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub color: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_string()).filter(|item| !item.is_empty())
}

fn validate_color(value: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = clean_optional(value) else {
        return Ok(None);
    };
    let valid = value.len() == 7
        && value.starts_with('#')
        && value.chars().skip(1).all(|character| character.is_ascii_hexdigit());
    if !valid {
        return Err("颜色必须使用 #RRGGBB 格式".to_string());
    }
    Ok(Some(value.to_uppercase()))
}

fn would_create_cycle(tag_id: i64, parent_id: i64) -> Result<bool, String> {
    if tag_id == parent_id {
        return Ok(true);
    }
    let conn = t_sqlite::open_conn()?;
    let mut current = Some(parent_id);
    let mut visited = HashSet::new();
    while let Some(id) = current {
        if id == tag_id || !visited.insert(id) {
            return Ok(true);
        }
        current = conn
            .query_row(
                "SELECT parent_id FROM atags WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .flatten();
    }
    Ok(false)
}

#[tauri::command]
pub fn taxonomy_get_snapshot() -> Result<TaxonomySnapshot, String> {
    t_dam::ensure_schema()?;
    let conn = t_sqlite::open_conn()?;

    let groups = {
        let mut statement = conn
            .prepare(
                "SELECT g.id, g.name, g.color, g.sort_order, COUNT(t.id)
                 FROM dam_tag_groups g
                 LEFT JOIN atags t ON t.group_id = g.id
                 GROUP BY g.id
                 ORDER BY g.sort_order, g.name COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(TaxonomyGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    sort_order: row.get(3)?,
                    tag_count: row.get(4)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?
    };

    let mut aliases: HashMap<i64, Vec<String>> = HashMap::new();
    {
        let mut statement = conn
            .prepare("SELECT tag_id, alias FROM dam_tag_aliases ORDER BY alias COLLATE NOCASE")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (tag_id, alias) = row.map_err(|error| error.to_string())?;
            aliases.entry(tag_id).or_default().push(alias);
        }
    }

    let tags = {
        let mut statement = conn
            .prepare(
                "SELECT t.id, t.name, t.group_id, t.parent_id, t.color, t.description,
                        COUNT(DISTINCT ft.file_id)
                 FROM atags t
                 LEFT JOIN afile_tags ft ON ft.tag_id = t.id
                 GROUP BY t.id
                 ORDER BY COALESCE(t.group_id, 0), t.name COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                Ok(TaxonomyTag {
                    id,
                    name: row.get(1)?,
                    group_id: row.get(2)?,
                    parent_id: row.get(3)?,
                    color: row.get(4)?,
                    description: row.get(5)?,
                    aliases: aliases.remove(&id).unwrap_or_default(),
                    file_count: row.get(6)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?
    };

    Ok(TaxonomySnapshot { groups, tags })
}

#[tauri::command]
pub fn taxonomy_save_group(input: TaxonomyGroupInput) -> Result<TaxonomyGroup, String> {
    t_dam::ensure_schema()?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("标签分组名称不能为空".to_string());
    }
    let color = validate_color(input.color)?;
    let now = Utc::now().timestamp_millis();
    let conn = t_sqlite::open_conn()?;
    let id = if let Some(id) = input.id {
        conn.execute(
            "UPDATE dam_tag_groups SET name = ?1, color = ?2, sort_order = COALESCE(?3, sort_order), updated_at = ?4 WHERE id = ?5",
            params![name, color, input.sort_order, now, id],
        )
        .map_err(|error| error.to_string())?;
        id
    } else {
        let sort_order = input.sort_order.unwrap_or_else(|| {
            conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM dam_tag_groups",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
        });
        conn.execute(
            "INSERT INTO dam_tag_groups (name, color, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![name, color, sort_order, now],
        )
        .map_err(|error| error.to_string())?;
        conn.last_insert_rowid()
    };
    conn.query_row(
        "SELECT g.id, g.name, g.color, g.sort_order, COUNT(t.id) FROM dam_tag_groups g LEFT JOIN atags t ON t.group_id = g.id WHERE g.id = ?1 GROUP BY g.id",
        params![id],
        |row| Ok(TaxonomyGroup {
            id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, sort_order: row.get(3)?, tag_count: row.get(4)?,
        }),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn taxonomy_delete_group(group_id: i64, move_tags_to_group_id: Option<i64>) -> Result<(), String> {
    t_dam::ensure_schema()?;
    if move_tags_to_group_id == Some(group_id) {
        return Err("不能将标签移动到即将删除的分组".to_string());
    }
    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE atags SET group_id = ?1 WHERE group_id = ?2",
            params![move_tags_to_group_id, group_id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM dam_tag_groups WHERE id = ?1", params![group_id])
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn taxonomy_save_tag(input: TaxonomyTagInput) -> Result<TaxonomyTag, String> {
    t_dam::ensure_schema()?;
    if let Some(parent_id) = input.parent_id {
        if would_create_cycle(input.tag_id, parent_id)? {
            return Err("父级标签设置会形成循环层级".to_string());
        }
    }
    let color = validate_color(input.color)?;
    let description = clean_optional(input.description);
    let aliases = input
        .aliases
        .into_iter()
        .map(|alias| alias.trim().to_string())
        .filter(|alias| !alias.is_empty())
        .collect::<Vec<_>>();
    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE atags SET group_id = ?1, parent_id = ?2, color = ?3, description = ?4 WHERE id = ?5",
            params![input.group_id, input.parent_id, color, description, input.tag_id],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute("DELETE FROM dam_tag_aliases WHERE tag_id = ?1", params![input.tag_id])
        .map_err(|error| error.to_string())?;
    let now = Utc::now().timestamp_millis();
    for alias in &aliases {
        transaction
            .execute(
                "INSERT INTO dam_tag_aliases (tag_id, alias, normalized_alias, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![input.tag_id, alias, normalize(alias), now],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction
        .execute(
            "DELETE FROM dam_tag_relations WHERE child_tag_id = ?1 AND relation_type = 'parent'",
            params![input.tag_id],
        )
        .map_err(|error| error.to_string())?;
    if let Some(parent_id) = input.parent_id {
        transaction
            .execute(
                "INSERT INTO dam_tag_relations (parent_tag_id, child_tag_id, relation_type, created_at) VALUES (?1, ?2, 'parent', ?3)",
                params![parent_id, input.tag_id, now],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;

    taxonomy_get_snapshot()?
        .tags
        .into_iter()
        .find(|tag| tag.id == input.tag_id)
        .ok_or_else(|| "标签不存在".to_string())
}
