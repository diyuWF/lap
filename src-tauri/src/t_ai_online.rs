use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineAiProviderKind {
    OpenaiCompatible,
    Gemini,
    Anthropic,
}

impl Default for OnlineAiProviderKind {
    fn default() -> Self {
        Self::OpenaiCompatible
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiProviderInput {
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub kind: OnlineAiProviderKind,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: OnlineAiProviderKind,
    pub model: String,
    pub enabled: bool,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiAnalysisResult {
    pub file_id: i64,
    pub tags: Vec<String>,
}

#[tauri::command]
pub fn list_online_ai_providers() -> Result<Vec<OnlineAiProviderSummary>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn save_online_ai_provider(_input: OnlineAiProviderInput) -> Result<OnlineAiProviderSummary, String> {
    Err("在线 AI 配置模块正在初始化，请稍后启用。".to_string())
}

#[tauri::command]
pub fn delete_online_ai_provider(_provider_id: String) -> Result<bool, String> {
    Ok(false)
}

#[tauri::command]
pub async fn test_online_ai_provider(_provider_id: String) -> Result<(), String> {
    Err("在线 AI 测试接口待接入。".to_string())
}

#[tauri::command]
pub async fn analyze_file_with_online_ai(
    _file_id: i64,
    _provider_id: String,
    _force_auto_apply: Option<bool>,
) -> Result<OnlineAiAnalysisResult, String> {
    Err("在线 AI 分析接口待接入。".to_string())
}
