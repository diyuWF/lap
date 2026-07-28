use crate::t_ai_online::{
    self, AiFolderSuggestionValue, OnlineAiAnalysisResult, OnlineAiFolderOption,
};
use crate::t_sqlite::{AFile, AFolder};
use crate::{t_cmds, t_dam, t_sqlite};
use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

const MAX_BATCH_FILES: usize = 200;
const MAX_AI_FOLDER_CANDIDATES: usize = 800;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiBatchInput {
    pub file_ids: Vec<i64>,
    pub provider_id: String,
    pub force_auto_apply: Option<bool>,
    #[serde(default = "default_continue_on_error")]
    pub continue_on_error: bool,
    pub organization_mode: Option<String>,
    pub root_folder_id: Option<i64>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFolderPlanRow {
    pub suggestion_id: i64,
    pub file_id: i64,
    pub file_name: String,
    pub file_path: Option<String>,
    pub target_folder_id: i64,
    pub target_folder_path: String,
    pub reason: String,
    pub confidence: Option<f64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub status: String,
    pub target_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAiFolderPlansInput {
    pub suggestion_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFolderPlanExecution {
    pub suggestion_id: i64,
    pub file_id: i64,
    pub target_folder_id: i64,
    pub target_folder_path: String,
    pub moved_path: String,
    pub already_in_target: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFolderPlanExecutionFailure {
    pub suggestion_id: i64,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAiFolderPlansResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<AiFolderPlanExecution>,
    pub failures: Vec<AiFolderPlanExecutionFailure>,
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

fn normalize_folder_path(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

fn path_is_same_or_descendant(parent: &str, candidate: &str) -> bool {
    let parent = normalize_folder_path(parent).to_lowercase();
    let candidate = normalize_folder_path(candidate).to_lowercase();
    candidate == parent
        || candidate
            .strip_prefix(&parent)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn folder_display_path(folder: &t_dam::DamFolder, all: &[t_dam::DamFolder]) -> String {
    let root = all
        .iter()
        .filter(|candidate| {
            candidate.album_id == folder.album_id
                && path_is_same_or_descendant(&candidate.path, &folder.path)
        })
        .min_by_key(|candidate| normalize_folder_path(&candidate.path).len())
        .unwrap_or(folder);
    let root_path = normalize_folder_path(&root.path);
    let folder_path = normalize_folder_path(&folder.path);
    let relative = folder_path
        .get(root_path.len()..)
        .unwrap_or_default()
        .trim_start_matches('/');
    if relative.is_empty() {
        root.name.clone()
    } else {
        format!("{}/{}", root.name, relative)
    }
}

fn is_inbox_folder(folder: &t_dam::DamFolder) -> bool {
    matches!(
        folder.name.trim().to_lowercase().as_str(),
        "inbox" | "待整理" | "待整理区域"
    )
}

fn organization_folder_options(
    mode: &str,
    root_folder_id: Option<i64>,
) -> Result<Vec<OnlineAiFolderOption>, String> {
    let all = t_dam::list_folders()?;
    let scoped = match mode {
        "within_folder" => {
            let root_id = root_folder_id.ok_or_else(|| "请选择要整理到的大文件夹".to_string())?;
            let root = all
                .iter()
                .find(|folder| folder.id == root_id)
                .ok_or_else(|| "选择的大文件夹已不存在，请刷新后重试".to_string())?;
            all.iter()
                .filter(|folder| {
                    folder.album_id == root.album_id
                        && path_is_same_or_descendant(&root.path, &folder.path)
                })
                .cloned()
                .collect::<Vec<_>>()
        }
        "library" => all.clone(),
        _ => return Err("未知的 AI 整理范围".to_string()),
    };

    let mut options = scoped
        .into_iter()
        .filter(|folder| !is_inbox_folder(folder))
        .map(|folder| OnlineAiFolderOption {
            folder_id: folder.id,
            display_path: folder_display_path(&folder, &all),
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| {
        left.display_path
            .to_lowercase()
            .cmp(&right.display_path.to_lowercase())
            .then(left.folder_id.cmp(&right.folder_id))
    });
    options.dedup_by_key(|folder| folder.folder_id);

    if options.is_empty() {
        return Err("所选范围没有可用的目标文件夹".to_string());
    }
    if options.len() > MAX_AI_FOLDER_CANDIDATES {
        return Err(format!(
            "当前范围包含 {} 个文件夹，超过 AI 单次可安全读取的 {} 个。请选择一个更具体的大文件夹。",
            options.len(),
            MAX_AI_FOLDER_CANDIDATES
        ));
    }
    Ok(options)
}

fn normalize_suggestion_ids(suggestion_ids: Vec<i64>) -> Result<Vec<i64>, String> {
    let mut normalized = Vec::new();
    for suggestion_id in suggestion_ids {
        if suggestion_id <= 0 || normalized.contains(&suggestion_id) {
            continue;
        }
        normalized.push(suggestion_id);
        if normalized.len() > MAX_BATCH_FILES {
            return Err(format!("单次最多执行 {} 条分类方案", MAX_BATCH_FILES));
        }
    }
    if normalized.is_empty() {
        return Err("没有可执行的 AI 分类方案".to_string());
    }
    Ok(normalized)
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
    let organization_mode = input
        .organization_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let folder_options = organization_mode
        .map(|mode| organization_folder_options(mode, input.root_folder_id))
        .transpose()?;
    let total = file_ids.len();
    let mut results = Vec::new();
    let mut failures = Vec::new();

    for file_id in file_ids {
        let analyzed = match folder_options.as_ref() {
            Some(folders) => {
                t_ai_online::analyze_file_for_organization(
                    file_id,
                    provider_id.clone(),
                    folders.clone(),
                    input.force_auto_apply,
                )
                .await
            }
            None => {
                t_ai_online::analyze_file_with_online_ai(
                    file_id,
                    provider_id.clone(),
                    input.force_auto_apply,
                )
                .await
            }
        };
        match analyzed {
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

#[tauri::command]
pub fn list_ai_folder_suggestions(limit: Option<i64>) -> Result<Vec<AiFolderPlanRow>, String> {
    t_dam::ensure_schema()?;
    let limit = limit.unwrap_or(200).clamp(1, 1000);
    let available_folder_ids = t_dam::list_folders()?
        .into_iter()
        .map(|folder| folder.id)
        .collect::<HashSet<_>>();
    let conn = t_sqlite::open_conn()?;
    let mut statement = conn
        .prepare(
            "SELECT suggestion.id,
                    suggestion.file_id,
                    COALESCE(file.name, ''),
                    current_folder.path,
                    suggestion.value,
                    suggestion.confidence,
                    suggestion.provider,
                    suggestion.model,
                    suggestion.status
             FROM dam_ai_suggestions suggestion
             LEFT JOIN afiles file ON file.id = suggestion.file_id
             LEFT JOIN afolders current_folder ON current_folder.id = file.folder_id
             WHERE suggestion.kind = 'folder'
               AND suggestion.status IN ('pending', 'accepted')
             ORDER BY suggestion.created_at DESC, suggestion.id DESC
             LIMIT ?1",
        )
        .map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params![limit])
        .map_err(|error| error.to_string())?;
    let mut plans = Vec::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let value: String = row.get(4).map_err(|error| error.to_string())?;
        let parsed: AiFolderSuggestionValue = match serde_json::from_str(&value) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };
        let file_name: String = row.get(2).map_err(|error| error.to_string())?;
        let current_folder_path: Option<String> =
            row.get(3).map_err(|error| error.to_string())?;
        let file_path = current_folder_path.map(|folder| {
            Path::new(&folder)
                .join(&file_name)
                .to_string_lossy()
                .into_owned()
        });
        plans.push(AiFolderPlanRow {
            suggestion_id: row.get(0).map_err(|error| error.to_string())?,
            file_id: row.get(1).map_err(|error| error.to_string())?,
            file_name,
            file_path,
            target_folder_id: parsed.folder_id,
            target_folder_path: parsed.display_path,
            reason: parsed.reason,
            confidence: row.get(5).map_err(|error| error.to_string())?,
            provider: row.get(6).map_err(|error| error.to_string())?,
            model: row.get(7).map_err(|error| error.to_string())?,
            status: row.get(8).map_err(|error| error.to_string())?,
            target_available: available_folder_ids.contains(&parsed.folder_id),
        });
    }
    Ok(plans)
}

fn execute_one_folder_plan(suggestion_id: i64) -> Result<AiFolderPlanExecution, String> {
    t_dam::ensure_schema()?;
    let conn = t_sqlite::open_conn()?;
    let suggestion = conn
        .query_row(
            "SELECT file_id, value, status
             FROM dam_ai_suggestions
             WHERE id = ?1 AND kind = 'folder'",
            params![suggestion_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "未找到该 AI 文件夹方案".to_string())?;
    if !matches!(suggestion.2.as_str(), "pending" | "accepted") {
        return Err("该 AI 文件夹方案已经处理".to_string());
    }
    let plan: AiFolderSuggestionValue =
        serde_json::from_str(&suggestion.1).map_err(|_| "AI 文件夹方案格式无效".to_string())?;
    let target = AFolder::get_by_id(plan.folder_id)?
        .ok_or_else(|| "AI 建议的目标文件夹已不存在".to_string())?;
    let target_id = target
        .id
        .ok_or_else(|| "目标文件夹缺少数据库 ID".to_string())?;
    let file = AFile::get_file_info(suggestion.0)?
        .ok_or_else(|| "待整理素材已不存在".to_string())?;
    let current_path = file
        .file_path
        .clone()
        .ok_or_else(|| "待整理素材路径不可用".to_string())?;
    let already_in_target = file.folder_id == target_id;
    let moved_path = if already_in_target {
        current_path
    } else {
        t_cmds::move_file(
            suggestion.0,
            &current_path,
            target_id,
            &target.path,
            "keep_both",
        )?
    };

    let changed = conn
        .execute(
            "UPDATE dam_ai_suggestions
             SET status = 'applied', reviewed_at = ?1
             WHERE id = ?2 AND status IN ('pending', 'accepted')",
            params![Utc::now().timestamp_millis(), suggestion_id],
        )
        .map_err(|error| {
            format!(
                "文件已移动，但 AI 方案状态更新失败，请刷新资料库后检查：{}",
                error
            )
        })?;
    if changed == 0 {
        return Err("文件已移动，但 AI 方案状态已变化，请刷新后检查".to_string());
    }
    conn.execute(
        "UPDATE dam_ai_suggestions
         SET status = 'rejected', reviewed_at = ?1
         WHERE file_id = ?2
           AND kind = 'folder'
           AND id <> ?3
           AND status IN ('pending', 'accepted')",
        params![
            Utc::now().timestamp_millis(),
            suggestion.0,
            suggestion_id
        ],
    )
    .map_err(|error| {
        format!(
            "文件已移动，但旧的冲突分类方案未能关闭，请刷新后检查：{}",
            error
        )
    })?;
    t_dam::set_workflow_status(suggestion.0, "reviewed")?;

    Ok(AiFolderPlanExecution {
        suggestion_id,
        file_id: suggestion.0,
        target_folder_id: target_id,
        target_folder_path: plan.display_path,
        moved_path,
        already_in_target,
    })
}

#[tauri::command]
pub fn execute_ai_folder_suggestions(
    input: ExecuteAiFolderPlansInput,
) -> Result<ExecuteAiFolderPlansResult, String> {
    let suggestion_ids = normalize_suggestion_ids(input.suggestion_ids)?;
    let total = suggestion_ids.len();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    for suggestion_id in suggestion_ids {
        match execute_one_folder_plan(suggestion_id) {
            Ok(result) => results.push(result),
            Err(error) => failures.push(AiFolderPlanExecutionFailure {
                suggestion_id,
                error,
            }),
        }
    }
    Ok(ExecuteAiFolderPlansResult {
        total,
        succeeded: results.len(),
        failed: failures.len(),
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

    #[test]
    fn recognizes_nested_folder_paths_with_mixed_separators() {
        assert!(path_is_same_or_descendant(
            "C:\\素材\\游戏原画",
            "C:/素材/游戏原画/三渲二"
        ));
        assert!(!path_is_same_or_descendant(
            "C:\\素材\\游戏原画",
            "C:/素材/游戏原画备份"
        ));
    }

    #[test]
    fn normalizes_folder_plan_ids() {
        assert_eq!(
            normalize_suggestion_ids(vec![0, 8, 8, -1, 11]).unwrap(),
            vec![8, 11]
        );
    }
}
