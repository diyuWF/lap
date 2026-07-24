use crate::{t_dam, t_sqlite};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestionRow {
    pub id: i64,
    pub file_id: i64,
    pub file_name: String,
    pub file_path: Option<String>,
    pub kind: String,
    pub value: String,
    pub confidence: Option<f64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub reviewed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSuggestionReviewResult {
    pub id: i64,
    pub status: String,
    pub applied_tag_ids: Vec<i64>,
}

#[tauri::command]
pub fn list_ai_suggestions(
    status: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<AiSuggestionRow>, String> {
    t_dam::ensure_schema()?;
    let status = status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("pending");
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let conn = t_sqlite::open_conn()?;
    let mut statement = conn
        .prepare(
            "SELECT s.id, s.file_id, COALESCE(f.name, ''), f.file_path,
                    s.kind, s.value, s.confidence, s.provider, s.model,
                    s.status, s.created_at, s.reviewed_at
             FROM dam_ai_suggestions s
             LEFT JOIN afiles f ON f.id = s.file_id
             WHERE (?1 = 'all' OR s.status = ?1)
             ORDER BY CASE WHEN s.status = 'pending' THEN 0 ELSE 1 END,
                      s.created_at DESC, s.id DESC
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![status, limit], |row| {
            Ok(AiSuggestionRow {
                id: row.get(0)?,
                file_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                kind: row.get(4)?,
                value: row.get(5)?,
                confidence: row.get(6)?,
                provider: row.get(7)?,
                model: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
                reviewed_at: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn review_ai_suggestion(
    suggestion_id: i64,
    action: String,
) -> Result<AiSuggestionReviewResult, String> {
    t_dam::ensure_schema()?;
    let action = action.trim().to_ascii_lowercase();
    if action != "accept" && action != "reject" {
        return Err("审核动作必须是 accept 或 reject".to_string());
    }

    let conn = t_sqlite::open_conn()?;
    let suggestion = conn
        .query_row(
            "SELECT file_id, kind, value, status FROM dam_ai_suggestions WHERE id = ?1",
            params![suggestion_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "未找到该 AI 建议".to_string())?;

    if suggestion.3 != "pending" {
        return Err("该 AI 建议已经处理".to_string());
    }

    let mut applied_tag_ids = Vec::new();
    let status = if action == "accept" {
        if suggestion.1 == "tag" {
            applied_tag_ids = t_dam::apply_tags(suggestion.0, &[suggestion.2.clone()])?;
        }
        "accepted"
    } else {
        "rejected"
    };

    let changed = conn
        .execute(
            "UPDATE dam_ai_suggestions
             SET status = ?1, reviewed_at = ?2
             WHERE id = ?3 AND status = 'pending'",
            params![status, Utc::now().timestamp_millis(), suggestion_id],
        )
        .map_err(|error| error.to_string())?;
    if changed == 0 {
        return Err("AI 建议状态已变化，请刷新后重试".to_string());
    }

    Ok(AiSuggestionReviewResult {
        id: suggestion_id,
        status: status.to_string(),
        applied_tag_ids,
    })
}

#[tauri::command]
pub fn clear_reviewed_ai_suggestions() -> Result<usize, String> {
    t_dam::ensure_schema()?;
    t_sqlite::open_conn()?
        .execute(
            "DELETE FROM dam_ai_suggestions WHERE status IN ('accepted', 'rejected', 'applied')",
            [],
        )
        .map_err(|error| error.to_string())
}
