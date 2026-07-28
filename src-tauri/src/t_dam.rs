use crate::t_sqlite::{self, AFolder, ATag};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSourceMetadata {
    pub source_url: String,
    pub page_url: Option<String>,
    pub page_title: Option<String>,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub alt_text: Option<String>,
    #[serde(default)]
    pub metadata: JsonValue,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DamFolder {
    pub id: i64,
    pub album_id: i64,
    pub name: String,
    pub path: String,
}

fn table_has_column(table: &str, column: &str) -> Result<bool, String> {
    let conn = t_sqlite::open_conn()?;
    let pragma = format!("PRAGMA table_info({})", table);
    let mut statement = conn.prepare(&pragma).map_err(|error| error.to_string())?;
    let mut rows = statement.query([]).map_err(|error| error.to_string())?;

    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let name: String = row.get(1).map_err(|error| error.to_string())?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn ensure_schema() -> Result<(), String> {
    let conn = t_sqlite::open_conn()?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS dam_tag_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            color TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dam_tag_aliases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tag_id INTEGER NOT NULL,
            alias TEXT NOT NULL,
            normalized_alias TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (tag_id) REFERENCES atags(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dam_tag_aliases_tag_id ON dam_tag_aliases(tag_id);

        CREATE TABLE IF NOT EXISTS dam_tag_relations (
            parent_tag_id INTEGER NOT NULL,
            child_tag_id INTEGER NOT NULL,
            relation_type TEXT NOT NULL DEFAULT 'parent',
            created_at INTEGER NOT NULL,
            PRIMARY KEY (parent_tag_id, child_tag_id, relation_type),
            FOREIGN KEY (parent_tag_id) REFERENCES atags(id) ON DELETE CASCADE,
            FOREIGN KEY (child_tag_id) REFERENCES atags(id) ON DELETE CASCADE,
            CHECK (parent_tag_id <> child_tag_id)
        );
        CREATE INDEX IF NOT EXISTS idx_dam_tag_relations_child ON dam_tag_relations(child_tag_id);

        CREATE TABLE IF NOT EXISTS dam_file_sources (
            file_id INTEGER PRIMARY KEY,
            source_url TEXT,
            page_url TEXT,
            page_title TEXT,
            author TEXT,
            site_name TEXT,
            alt_text TEXT,
            captured_at INTEGER NOT NULL,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dam_file_sources_source_url ON dam_file_sources(source_url);
        CREATE INDEX IF NOT EXISTS idx_dam_file_sources_page_url ON dam_file_sources(page_url);
        CREATE INDEX IF NOT EXISTS idx_dam_file_sources_captured_at ON dam_file_sources(captured_at DESC);

        CREATE TABLE IF NOT EXISTS dam_file_workflow (
            file_id INTEGER PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'inbox',
            reviewed_at INTEGER,
            archived_at INTEGER,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dam_file_workflow_status ON dam_file_workflow(status, updated_at DESC);

        CREATE TABLE IF NOT EXISTS dam_ai_suggestions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            kind TEXT NOT NULL,
            value TEXT NOT NULL,
            confidence REAL,
            provider TEXT,
            model TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at INTEGER NOT NULL,
            reviewed_at INTEGER,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dam_ai_suggestions_file ON dam_ai_suggestions(file_id, status);
        CREATE INDEX IF NOT EXISTS idx_dam_ai_suggestions_status ON dam_ai_suggestions(status, created_at DESC);
        ",
    )
    .map_err(|error| error.to_string())?;
    drop(conn);

    if !table_has_column("atags", "group_id")? {
        t_sqlite::open_conn()?
            .execute("ALTER TABLE atags ADD COLUMN group_id INTEGER", [])
            .map_err(|error| error.to_string())?;
    }
    if !table_has_column("atags", "parent_id")? {
        t_sqlite::open_conn()?
            .execute("ALTER TABLE atags ADD COLUMN parent_id INTEGER", [])
            .map_err(|error| error.to_string())?;
    }
    if !table_has_column("atags", "color")? {
        t_sqlite::open_conn()?
            .execute("ALTER TABLE atags ADD COLUMN color TEXT", [])
            .map_err(|error| error.to_string())?;
    }
    if !table_has_column("atags", "description")? {
        t_sqlite::open_conn()?
            .execute("ALTER TABLE atags ADD COLUMN description TEXT", [])
            .map_err(|error| error.to_string())?;
    }
    if !table_has_column("atags", "normalized_name")? {
        t_sqlite::open_conn()?
            .execute("ALTER TABLE atags ADD COLUMN normalized_name TEXT", [])
            .map_err(|error| error.to_string())?;
    }

    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "UPDATE atags SET normalized_name = lower(trim(name)) WHERE normalized_name IS NULL OR normalized_name = ''",
        [],
    )
    .map_err(|error| error.to_string())?;
    // Existing libraries may already contain tags that differ only by case.
    // Keep this index non-unique so schema initialization never blocks startup.
    conn.execute("DROP INDEX IF EXISTS idx_atags_normalized_name", [])
        .map_err(|error| error.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_atags_normalized_name_lookup ON atags(normalized_name)",
        [],
    )
    .map_err(|error| error.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_atags_group_parent ON atags(group_id, parent_id, name)",
        [],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn list_folders() -> Result<Vec<DamFolder>, String> {
    ensure_schema()?;
    let mut folders = AFolder::get_all()?
        .into_iter()
        .filter_map(|folder| {
            Some(DamFolder {
                id: folder.id?,
                album_id: folder.album_id,
                name: folder.name,
                path: folder.path,
            })
        })
        .collect::<Vec<_>>();
    folders.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    Ok(folders)
}

pub fn find_folder(path: Option<&str>) -> Result<Option<DamFolder>, String> {
    let folders = list_folders()?;
    if let Some(path) = path.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(folders.into_iter().find(|folder| folder.path == path));
    }

    Ok(folders
        .iter()
        .find(|folder| {
            matches!(
                folder.name.trim().to_lowercase().as_str(),
                "inbox" | "待整理" | "待整理区域"
            )
        })
        .cloned()
        .or_else(|| folders.into_iter().next()))
}

pub fn find_file_by_source_url(source_url: &str) -> Result<Option<i64>, String> {
    ensure_schema()?;
    let conn = t_sqlite::open_conn()?;
    conn.query_row(
        "SELECT file_id FROM dam_file_sources WHERE source_url = ?1 ORDER BY captured_at DESC LIMIT 1",
        params![source_url],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub fn upsert_source_metadata(
    file_id: i64,
    source: &CaptureSourceMetadata,
) -> Result<(), String> {
    ensure_schema()?;
    let captured_at = Utc::now().timestamp_millis();
    let metadata_json = serde_json::to_string(&source.metadata).map_err(|error| error.to_string())?;
    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "
        INSERT INTO dam_file_sources (
            file_id, source_url, page_url, page_title, author, site_name,
            alt_text, captured_at, metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(file_id) DO UPDATE SET
            source_url = excluded.source_url,
            page_url = excluded.page_url,
            page_title = excluded.page_title,
            author = excluded.author,
            site_name = excluded.site_name,
            alt_text = excluded.alt_text,
            captured_at = excluded.captured_at,
            metadata_json = excluded.metadata_json
        ",
        params![
            file_id,
            source.source_url,
            source.page_url,
            source.page_title,
            source.author,
            source.site_name,
            source.alt_text,
            captured_at,
            metadata_json,
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn set_workflow_status(file_id: i64, status: &str) -> Result<(), String> {
    ensure_schema()?;
    let normalized = match status.trim().to_lowercase().as_str() {
        "reviewed" => "reviewed",
        "selected" => "selected",
        "archived" => "archived",
        _ => "inbox",
    };
    let now = Utc::now().timestamp_millis();
    let reviewed_at = matches!(normalized, "reviewed" | "selected").then_some(now);
    let archived_at = (normalized == "archived").then_some(now);
    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "
        INSERT INTO dam_file_workflow (file_id, status, reviewed_at, archived_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(file_id) DO UPDATE SET
            status = excluded.status,
            reviewed_at = COALESCE(excluded.reviewed_at, dam_file_workflow.reviewed_at),
            archived_at = excluded.archived_at,
            updated_at = excluded.updated_at
        ",
        params![file_id, normalized, reviewed_at, archived_at, now],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn normalize_tag(value: &str) -> String {
    value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn ensure_group(name: &str) -> Result<i64, String> {
    let normalized = normalize_tag(name);
    let now = Utc::now().timestamp_millis();
    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "INSERT OR IGNORE INTO dam_tag_groups (name, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![normalized, now],
    )
    .map_err(|error| error.to_string())?;
    conn.query_row(
        "SELECT id FROM dam_tag_groups WHERE name = ?1 COLLATE NOCASE",
        params![normalized],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn ensure_tag(raw_tag: &str) -> Result<Option<i64>, String> {
    let raw_tag = raw_tag.trim();
    if raw_tag.is_empty() {
        return Ok(None);
    }

    let (group_name, display_name) = raw_tag
        .split_once(':')
        .map(|(group, value)| (Some(group.trim()), value.trim()))
        .unwrap_or((None, raw_tag));
    if display_name.is_empty() {
        return Ok(None);
    }

    let normalized_name = normalize_tag(raw_tag);
    let group_id = match group_name.filter(|value| !value.is_empty()) {
        Some(group) => Some(ensure_group(group)?),
        None => None,
    };

    let conn = t_sqlite::open_conn()?;
    let existing_id = conn
        .query_row(
            "SELECT id FROM atags WHERE normalized_name = ?1 OR name = ?2 COLLATE NOCASE ORDER BY id LIMIT 1",
            params![normalized_name, raw_tag],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    drop(conn);

    let tag_id = match existing_id {
        Some(id) => id,
        None => ATag::add(raw_tag)?.id,
    };

    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "UPDATE atags SET group_id = COALESCE(group_id, ?2), normalized_name = ?3 WHERE id = ?1",
        params![tag_id, group_id, normalize_tag(raw_tag)],
    )
    .map_err(|error| error.to_string())?;
    Ok(Some(tag_id))
}

pub fn apply_tags(file_id: i64, tags: &[String]) -> Result<Vec<i64>, String> {
    ensure_schema()?;
    let mut tag_ids = Vec::new();
    for tag in tags {
        if let Some(tag_id) = ensure_tag(tag)? {
            if !tag_ids.contains(&tag_id) {
                tag_ids.push(tag_id);
            }
        }
    }

    if tag_ids.is_empty() {
        return Ok(tag_ids);
    }

    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    {
        let mut statement = transaction
            .prepare_cached("INSERT OR IGNORE INTO afile_tags (file_id, tag_id) VALUES (?1, ?2)")
            .map_err(|error| error.to_string())?;
        for tag_id in &tag_ids {
            statement
                .execute(params![file_id, tag_id])
                .map_err(|error| error.to_string())?;
        }
    }
    transaction
        .execute("UPDATE afiles SET has_tags = 1 WHERE id = ?1", params![file_id])
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(tag_ids)
}
