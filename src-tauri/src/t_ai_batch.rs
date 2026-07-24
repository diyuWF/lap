use crate::t_ai_online::{self, OnlineAiAnalysisResult};
use crate::{t_dam, t_sqlite};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_BATCH_FILES: usize = 200;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiBatchInput {
    pub file_ids: Vec<i64>,
    pub provider_id: String,
    pub force_auto_apply: Option<bool>,
    #[serde(default = "default_continue_on_error")]
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiBatchFailure {
    pub file_id: i64,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiBatchResult {
    pub total: usize,
    pub completed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<OnlineAiAnalysisResult>,
    pub failures: Vec<OnlineAiBatchFailure>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiBatchCandidate {
    pub file_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_type: i64,
    pub workflow_status: String,
    pub has_ai_suggestions: bool,
}

fn default_continue_on_error() -> bool {
    true
}

fn normalize_file_ids(file_ids: Vec<i64>) -> Result<Vec<i64>, String> {
    let mut normalized = Vec::new();
    for file_id in file_ids {
        if file_id <= 0 || normalized.contains(&file_id) {
            continue;
        }
        normalized.push(file_id);
        if normalized.len() > MAX_BATCH_FILES {
            return Err(format!("单次最多分析 {} 个文件", MAX_BATCH_FILES));
        }
    }
    if normalized.is_empty() {
        return Err("没有可分析的文件".to_string());
    }
    Ok(normalized)
}

fn normalize_workflow_status(value: Option<String>) -> String {
    match value
        .as_deref()
        .map(str::trim)
        .unwrap_or("inbox")
        .to_ascii_lowercase()
        .as_str()
    {
        "reviewed" => "reviewed".to_string(),
        "selected" => "selected".to_string(),
        "archived" => "archived".to_string(),
        "all" => "all".to_string(),
        _ => "inbox".to_string(),
    }
}

#[tauri::command]
pub fn list_online_ai_batch_candidates(
    workflow_status: Option<String>,
    limit: Option<i64>,
    include_analyzed: Option<bool>,
) -> Result<Vec<OnlineAiBatchCandidate>, String> {
    t_dam::ensure_schema()?;
    let workflow_status = normalize_workflow_status(workflow_status);
    let limit = limit.unwrap_or(100).clamp(1, MAX_BATCH_FILES as i64);
    let include_analyzed = include_analyzed.unwrap_or(false);
    let conn = t_sqlite::open_conn()?;
    let mut statement = conn
        .prepare(
            "SELECT f.id,
                    f.name,
                    folder.path,
                    COALESCE(f.file_type, 0),
                    COALESCE(workflow.status, 'inbox'),
                    EXISTS(SELECT 1 FROM dam_ai_suggestions suggestions WHERE suggestions.file_id = f.id)
             FROM afiles f
             JOIN afolders folder ON folder.id = f.folder_id
             LEFT JOIN dam_file_workflow workflow ON workflow.file_id = f.id
             WHERE COALESCE(f.file_type, 0) IN (1, 2, 3)
               AND f.id NOT IN (
                   SELECT live_photo_video_id
                   FROM afiles
                   WHERE live_photo_video_id IS NOT NULL
               )
               AND (?1 = 'all' OR COALESCE(workflow.status, 'inbox') = ?1)
               AND (?2 = 1 OR NOT EXISTS(
                   SELECT 1 FROM dam_ai_suggestions previous WHERE previous.file_id = f.id
               ))
             ORDER BY COALESCE(f.modified_at, f.created_at, 0) DESC, f.id DESC
             LIMIT ?3",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![workflow_status, if include_analyzed { 1 } else { 0 }, limit],
            |row| {
                let file_name: String = row.get(1)?;
                let folder_path: String = row.get(2)?;
                Ok(OnlineAiBatchCandidate {
                    file_id: row.get(0)?,
                    file_path: Path::new(&folder_path)
                        .join(&file_name)
                        .to_string_lossy()
                        .into_owned(),
                    file_name,
                    file_type: row.get(3)?,
                    workflow_status: row.get(4)?,
                    has_ai_suggestions: row.get::<_, i64>(5)? != 0,
                })
            },
        )
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn analyze_files_with_online_ai(
    input: OnlineAiBatchInput,
) -> Result<OnlineAiBatchResult, String> {
    let provider_id = input.provider_id.trim().to_string();
    if provider_id.is_empty() {
        return Err("未指定在线 AI 服务".to_string());
    }
    let file_ids = normalize_file_ids(input.file_ids)?;
    let total = file_ids.len();
    let mut results = Vec::new();
    let mut failures = Vec::new();

    for file_id in file_ids {
        match t_ai_online::analyze_file_with_online_ai(
            file_id,
            provider_id.clone(),
            input.force_auto_apply,
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(error) => {
                failures.push(OnlineAiBatchFailure { file_id, error });
                if !input.continue_on_error {
                    break;
                }
            }
        }
    }

    let succeeded = results.len();
    let failed = failures.len();
    Ok(OnlineAiBatchResult {
        total,
        completed: succeeded + failed,
        succeeded,
        failed,
        results,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_duplicate_and_invalid_file_ids() {
        assert_eq!(
            normalize_file_ids(vec![0, 4, 4, -2, 7]).unwrap(),
            vec![4, 7]
        );
    }

    #[test]
    fn rejects_empty_batches() {
        assert!(normalize_file_ids(vec![0, -1]).is_err());
    }

    #[test]
    fn rejects_oversized_batches() {
        let ids = (1..=(MAX_BATCH_FILES as i64 + 1)).collect();
        assert!(normalize_file_ids(ids).is_err());
    }

    #[test]
    fn normalizes_unknown_workflow_status_to_inbox() {
        assert_eq!(
            normalize_workflow_status(Some("unknown".to_string())),
            "inbox"
        );
        assert_eq!(normalize_workflow_status(Some("ALL".to_string())), "all");
    }
}
