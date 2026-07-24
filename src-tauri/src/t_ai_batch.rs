use crate::t_ai_online::{self, OnlineAiAnalysisResult};
use serde::{Deserialize, Serialize};

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
        assert_eq!(normalize_file_ids(vec![0, 4, 4, -2, 7]).unwrap(), vec![4, 7]);
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
}
