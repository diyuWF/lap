use crate::t_config;
use crate::t_dam;
use crate::t_sqlite;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use reqwest::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

const CONFIG_VERSION: u32 = 1;
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a visual digital-asset librarian. Analyse the supplied visual asset and return strict JSON only. Create concise, reusable taxonomy tags. Prefer namespace:value tags such as subject:vehicle, style:minimal, composition:centered, lighting:studio, color:blue, material:metal, usage:ui-reference. Never invent people, brands, projects, or copyrighted titles. Return exactly this shape: {\"title\":\"\",\"description\":\"\",\"tags\":[\"namespace:value\"],\"dominantColors\":[\"#RRGGBB\"],\"suggestedFolder\":\"\",\"confidence\":0.0}."#;

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
struct StoredOnlineAiProvider {
    id: String,
    name: String,
    #[serde(default)]
    kind: OnlineAiProviderKind,
    base_url: String,
    model: String,
    #[serde(default)]
    api_key: String,
    #[serde(default = "default_auth_header")]
    auth_header: String,
    #[serde(default = "default_auth_prefix")]
    auth_prefix: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    auto_apply_tags: bool,
    #[serde(default)]
    auto_mark_reviewed: bool,
    #[serde(default = "default_min_confidence")]
    min_confidence: f64,
    #[serde(default = "default_max_tags")]
    max_tags: usize,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_system_prompt")]
    system_prompt: String,
    #[serde(default)]
    extra_headers: HashMap<String, String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredOnlineAiConfig {
    #[serde(default = "default_config_version")]
    version: u32,
    #[serde(default)]
    providers: Vec<StoredOnlineAiProvider>,
}

impl Default for StoredOnlineAiConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            providers: Vec::new(),
        }
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
    pub auth_header: Option<String>,
    pub auth_prefix: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_apply_tags: bool,
    #[serde(default)]
    pub auto_mark_reviewed: bool,
    pub min_confidence: Option<f64>,
    pub max_tags: Option<usize>,
    pub language: Option<String>,
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiProviderSummary {
    pub id: String,
    pub name: String,
    pub kind: OnlineAiProviderKind,
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
    pub has_api_key: bool,
    pub api_key_preview: String,
    pub auth_header: String,
    pub auth_prefix: String,
    pub auto_apply_tags: bool,
    pub auto_mark_reviewed: bool,
    pub min_confidence: f64,
    pub max_tags: usize,
    pub language: String,
    pub system_prompt: String,
    pub extra_headers: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiAnalysis {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dominant_colors: Vec<String>,
    #[serde(default)]
    pub suggested_folder: String,
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiAnalysisResult {
    pub file_id: i64,
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub analysis: OnlineAiAnalysis,
    pub applied_tag_ids: Vec<i64>,
    pub tags_applied: bool,
    pub workflow_updated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiProviderTestResult {
    pub ok: bool,
    pub provider_id: String,
    pub model: String,
    pub elapsed_ms: i64,
    pub message: String,
}

fn default_true() -> bool {
    true
}

fn default_config_version() -> u32 {
    CONFIG_VERSION
}

fn default_auth_header() -> String {
    "Authorization".to_string()
}

fn default_auth_prefix() -> String {
    "Bearer ".to_string()
}

fn default_min_confidence() -> f64 {
    0.78
}

fn default_max_tags() -> usize {
    12
}

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_system_prompt() -> String {
    DEFAULT_SYSTEM_PROMPT.to_string()
}

fn config_path() -> Result<PathBuf, String> {
    let directory = t_config::get_app_data_dir()?.join("ai");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("online-providers.json"))
}

fn load_config() -> Result<StoredOnlineAiConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(StoredOnlineAiConfig::default());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("在线 AI 配置文件损坏：{}", error))
}

fn save_config(config: &StoredOnlineAiConfig) -> Result<(), String> {
    let path = config_path()?;
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }

    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn validate_base_url(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    if value.starts_with("https://") || value.starts_with("http://") {
        Ok(value.to_string())
    } else {
        Err("API 地址必须以 http:// 或 https:// 开头".to_string())
    }
}

fn provider_summary(provider: &StoredOnlineAiProvider) -> OnlineAiProviderSummary {
    let api_key_preview = if provider.api_key.is_empty() {
        String::new()
    } else {
        let suffix = provider
            .api_key
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        format!("********{}", suffix)
    };

    OnlineAiProviderSummary {
        id: provider.id.clone(),
        name: provider.name.clone(),
        kind: provider.kind.clone(),
        base_url: provider.base_url.clone(),
        model: provider.model.clone(),
        enabled: provider.enabled,
        has_api_key: !provider.api_key.is_empty(),
        api_key_preview,
        auth_header: provider.auth_header.clone(),
        auth_prefix: provider.auth_prefix.clone(),
        auto_apply_tags: provider.auto_apply_tags,
        auto_mark_reviewed: provider.auto_mark_reviewed,
        min_confidence: provider.min_confidence,
        max_tags: provider.max_tags,
        language: provider.language.clone(),
        system_prompt: provider.system_prompt.clone(),
        extra_headers: provider.extra_headers.clone(),
        created_at: provider.created_at,
        updated_at: provider.updated_at,
    }
}

fn ensure_ai_schema() -> Result<(), String> {
    t_dam::ensure_schema()?;
    let conn = t_sqlite::open_conn()?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS dam_ai_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            provider_id TEXT,
            source TEXT NOT NULL DEFAULT 'manual',
            status TEXT NOT NULL DEFAULT 'queued',
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            result_json TEXT,
            created_at INTEGER NOT NULL,
            started_at INTEGER,
            completed_at INTEGER,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dam_ai_jobs_status_created
            ON dam_ai_jobs(status, created_at, id);
        CREATE INDEX IF NOT EXISTS idx_dam_ai_jobs_file
            ON dam_ai_jobs(file_id, status, id DESC);
        ",
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_online_ai_providers() -> Result<Vec<OnlineAiProviderSummary>, String> {
    ensure_ai_schema()?;
    Ok(load_config()?
        .providers
        .iter()
        .map(provider_summary)
        .collect())
}

#[tauri::command]
pub fn save_online_ai_provider(
    input: OnlineAiProviderInput,
) -> Result<OnlineAiProviderSummary, String> {
    ensure_ai_schema()?;
    let mut config = load_config()?;
    let provider_id = input
        .id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing = config
        .providers
        .iter()
        .find(|provider| provider.id == provider_id)
        .cloned();
    let name = input.name.trim();
    let model = input.model.trim();
    if name.is_empty() {
        return Err("服务名称不能为空".to_string());
    }
    if model.is_empty() {
        return Err("模型名称不能为空".to_string());
    }

    let supplied_key = input.api_key.unwrap_or_default().trim().to_string();
    let api_key = if supplied_key.is_empty() {
        existing
            .as_ref()
            .map(|provider| provider.api_key.clone())
            .unwrap_or_default()
    } else {
        supplied_key
    };
    let now = Utc::now().timestamp_millis();
    let provider = StoredOnlineAiProvider {
        id: provider_id.clone(),
        name: name.to_string(),
        kind: input.kind,
        base_url: validate_base_url(&input.base_url)?,
        model: model.to_string(),
        api_key,
        auth_header: input
            .auth_header
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(default_auth_header),
        auth_prefix: input.auth_prefix.unwrap_or_else(default_auth_prefix),
        enabled: input.enabled,
        auto_apply_tags: input.auto_apply_tags,
        auto_mark_reviewed: input.auto_mark_reviewed,
        min_confidence: input
            .min_confidence
            .unwrap_or_else(default_min_confidence)
            .clamp(0.0, 1.0),
        max_tags: input.max_tags.unwrap_or_else(default_max_tags).clamp(1, 50),
        language: input
            .language
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(default_language),
        system_prompt: input
            .system_prompt
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(default_system_prompt),
        extra_headers: input.extra_headers,
        created_at: existing
            .as_ref()
            .map(|provider| provider.created_at)
            .unwrap_or(now),
        updated_at: now,
    };

    if let Some(index) = config
        .providers
        .iter()
        .position(|provider| provider.id == provider_id)
    {
        config.providers[index] = provider.clone();
    } else {
        config.providers.push(provider.clone());
    }
    config.version = CONFIG_VERSION;
    save_config(&config)?;
    Ok(provider_summary(&provider))
}

#[tauri::command]
pub fn delete_online_ai_provider(provider_id: String) -> Result<bool, String> {
    let mut config = load_config()?;
    let before = config.providers.len();
    config
        .providers
        .retain(|provider| provider.id != provider_id.trim());
    let changed = config.providers.len() != before;
    if changed {
        save_config(&config)?;
    }
    Ok(changed)
}

fn select_provider(provider_id: Option<&str>) -> Result<StoredOnlineAiProvider, String> {
    let config = load_config()?;
    let requested = provider_id.map(str::trim).filter(|value| !value.is_empty());
    let provider = match requested {
        Some(id) => config.providers.into_iter().find(|provider| provider.id == id),
        None => config.providers.into_iter().find(|provider| provider.enabled),
    }
    .ok_or_else(|| "没有可用的在线 AI 服务，请先在设置中添加并启用一个服务".to_string())?;

    if !provider.enabled {
        return Err("该在线 AI 服务已停用".to_string());
    }
    if matches!(provider.kind, OnlineAiProviderKind::Gemini | OnlineAiProviderKind::Anthropic)
        && provider.api_key.trim().is_empty()
    {
        return Err("该在线 AI 服务尚未配置 API Key".to_string());
    }
    Ok(provider)
}

struct ImagePayload {
    mime: String,
    base64: String,
}

fn direct_image_mime(extension: &str) -> Option<&'static str> {
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

fn load_image_payload(file_id: i64) -> Result<ImagePayload, String> {
    let conn = t_sqlite::open_conn()?;
    let file: Option<(String, String)> = conn
        .query_row(
            "SELECT b.path, a.name FROM afiles a JOIN afolders b ON b.id = a.folder_id WHERE a.id = ?1",
            params![file_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let (folder_path, file_name) = file.ok_or_else(|| format!("未找到文件：{}", file_id))?;
    let file_path = Path::new(&folder_path).join(&file_name);
    let extension = file_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if let Some(mime) = direct_image_mime(extension) {
        let bytes = fs::read(&file_path).map_err(|error| error.to_string())?;
        if bytes.len() <= MAX_IMAGE_BYTES {
            return Ok(ImagePayload {
                mime: mime.to_string(),
                base64: STANDARD.encode(bytes),
            });
        }
    }

    let thumbnail: Option<Vec<u8>> = conn
        .query_row(
            "SELECT thumb_data FROM athumbs WHERE file_id = ?1",
            params![file_id],
            |row| row.get::<_, Option<Vec<u8>>>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .flatten();
    let thumbnail = thumbnail.ok_or_else(|| "该素材没有可用于在线 AI 的预览图".to_string())?;
    if thumbnail.len() > MAX_IMAGE_BYTES {
        return Err("预览图超过 20 MB，无法发送到在线 AI".to_string());
    }
    let mime = if thumbnail.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else {
        "image/jpeg"
    };
    Ok(ImagePayload {
        mime: mime.to_string(),
        base64: STANDARD.encode(thumbnail),
    })
}

fn endpoint(base_url: &str, suffix: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let suffix = suffix.trim_start_matches('/');
    if base.ends_with(suffix) {
        base.to_string()
    } else {
        format!("{}/{}", base, suffix)
    }
}

fn analysis_prompt(provider: &StoredOnlineAiProvider, connection_test: bool) -> String {
    if connection_test {
        return format!(
            "这是连接测试。请使用 {} 只返回以下结构的有效 JSON：{{\"title\":\"连接测试\",\"description\":\"ok\",\"tags\":[\"status:connected\"],\"dominantColors\":[],\"suggestedFolder\":\"\",\"confidence\":1.0}}",
            provider.language
        );
    }
    format!(
        "请使用 {} 输出。最多生成 {} 个标签。标签必须稳定、可复用，并优先使用 namespace:value 格式。suggestedFolder 只给出简短的语义目录建议，不要编造磁盘路径。只输出 JSON，不要 Markdown。",
        provider.language, provider.max_tags
    )
}

enum AuthMode {
    Generic,
    Gemini,
    Anthropic,
}

fn apply_headers(
    mut request: reqwest::RequestBuilder,
    provider: &StoredOnlineAiProvider,
    auth_mode: AuthMode,
) -> Result<reqwest::RequestBuilder, String> {
    match auth_mode {
        AuthMode::Generic => {
            if !provider.api_key.is_empty() && !provider.auth_header.trim().is_empty() {
                let name = HeaderName::from_bytes(provider.auth_header.trim().as_bytes())
                    .map_err(|error| format!("认证 Header 无效：{}", error))?;
                let value = HeaderValue::from_str(&format!(
                    "{}{}",
                    provider.auth_prefix, provider.api_key
                ))
                .map_err(|error| format!("认证值无效：{}", error))?;
                request = request.header(name, value);
            }
        }
        AuthMode::Gemini => {
            request = request.header("x-goog-api-key", provider.api_key.as_str());
        }
        AuthMode::Anthropic => {
            request = request
                .header("x-api-key", provider.api_key.as_str())
                .header("anthropic-version", "2023-06-01");
        }
    }

    for (name, value) in &provider.extra_headers {
        let name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|error| format!("自定义 Header 名无效：{}", error))?;
        let value = HeaderValue::from_str(value)
            .map_err(|error| format!("自定义 Header 值无效：{}", error))?;
        request = request.header(name, value);
    }
    Ok(request)
}

async fn post_json(
    provider: &StoredOnlineAiProvider,
    url: &str,
    body: JsonValue,
    auth_mode: AuthMode,
) -> Result<JsonValue, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())?;
    let request = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&body).map_err(|error| error.to_string())?);
    let response = apply_headers(request, provider, auth_mode)?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let value: JsonValue = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "AI 服务返回的内容不是 JSON（HTTP {}）：{}；{}",
            status,
            error,
            String::from_utf8_lossy(&bytes)
                .chars()
                .take(500)
                .collect::<String>()
        )
    })?;
    if !status.is_success() {
        return Err(format!("AI 服务请求失败（HTTP {}）：{}", status, value));
    }
    Ok(value)
}

fn extract_openai_text(value: &JsonValue) -> Result<String, String> {
    let content = &value["choices"][0]["message"]["content"];
    if let Some(text) = content.as_str() {
        return Ok(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let text = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            return Ok(text);
        }
    }
    Err(format!("OpenAI-compatible 响应中没有文本：{}", value))
}

fn extract_gemini_text(value: &JsonValue) -> Result<String, String> {
    let text = value["candidates"][0]["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    if text.is_empty() {
        Err(format!("Gemini 响应中没有文本：{}", value))
    } else {
        Ok(text)
    }
}

fn extract_anthropic_text(value: &JsonValue) -> Result<String, String> {
    let text = value["content"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter(|part| part.get("type").and_then(JsonValue::as_str) == Some("text"))
                .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    if text.is_empty() {
        Err(format!("Anthropic 响应中没有文本：{}", value))
    } else {
        Ok(text)
    }
}

async fn call_provider(
    provider: &StoredOnlineAiProvider,
    image: Option<&ImagePayload>,
    connection_test: bool,
) -> Result<String, String> {
    let prompt = analysis_prompt(provider, connection_test);
    match provider.kind {
        OnlineAiProviderKind::OpenaiCompatible => {
            let url = endpoint(&provider.base_url, "chat/completions");
            let user_content = if let Some(image) = image {
                json!([
                    { "type": "text", "text": prompt },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{};base64,{}", image.mime, image.base64),
                            "detail": "auto"
                        }
                    }
                ])
            } else {
                json!(prompt)
            };
            let body = json!({
                "model": provider.model,
                "temperature": 0.1,
                "messages": [
                    { "role": "system", "content": provider.system_prompt },
                    { "role": "user", "content": user_content }
                ]
            });
            let value = post_json(provider, &url, body, AuthMode::Generic).await?;
            extract_openai_text(&value)
        }
        OnlineAiProviderKind::Gemini => {
            let url = if provider.base_url.contains(":generateContent") {
                provider.base_url.clone()
            } else {
                endpoint(
                    &provider.base_url,
                    &format!("models/{}:generateContent", provider.model),
                )
            };
            let mut parts = vec![json!({
                "text": format!("{}\n{}", provider.system_prompt, prompt)
            })];
            if let Some(image) = image {
                parts.push(json!({
                    "inline_data": {
                        "mime_type": image.mime,
                        "data": image.base64
                    }
                }));
            }
            let body = json!({
                "contents": [{ "role": "user", "parts": parts }],
                "generationConfig": {
                    "temperature": 0.1,
                    "responseMimeType": "application/json"
                }
            });
            let value = post_json(provider, &url, body, AuthMode::Gemini).await?;
            extract_gemini_text(&value)
        }
        OnlineAiProviderKind::Anthropic => {
            let url = endpoint(&provider.base_url, "messages");
            let mut content = Vec::new();
            if let Some(image) = image {
                content.push(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": image.mime,
                        "data": image.base64
                    }
                }));
            }
            content.push(json!({ "type": "text", "text": prompt }));
            let body = json!({
                "model": provider.model,
                "max_tokens": 1400,
                "temperature": 0.1,
                "system": provider.system_prompt,
                "messages": [{ "role": "user", "content": content }]
            });
            let value = post_json(provider, &url, body, AuthMode::Anthropic).await?;
            extract_anthropic_text(&value)
        }
    }
}

fn strip_code_fence(value: &str) -> &str {
    let trimmed = value.trim();
    if !trimmed.starts_with("```") {
        return trimmed;
    }
    let without_prefix = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_start();
    without_prefix
        .strip_suffix("```")
        .unwrap_or(without_prefix)
        .trim()
}

fn normalize_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('#');
    if trimmed.is_empty() {
        return None;
    }
    let allowed_namespaces = [
        "subject",
        "style",
        "composition",
        "lighting",
        "color",
        "material",
        "usage",
        "mood",
        "scene",
        "camera",
        "technique",
        "keyword",
    ];
    let (namespace, value) = trimmed
        .split_once(':')
        .map(|(namespace, value)| (namespace.trim().to_lowercase(), value.trim()))
        .unwrap_or_else(|| ("keyword".to_string(), trimmed));
    if value.is_empty() {
        return None;
    }
    let namespace = if allowed_namespaces.contains(&namespace.as_str()) {
        namespace
    } else {
        "keyword".to_string()
    };
    let value = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_lowercase();
    if value.is_empty() {
        None
    } else {
        Some(format!("{}:{}", namespace, value))
    }
}

fn normalize_analysis(
    mut analysis: OnlineAiAnalysis,
    provider: &StoredOnlineAiProvider,
) -> OnlineAiAnalysis {
    let mut seen = HashSet::new();
    analysis.tags = analysis
        .tags
        .iter()
        .filter_map(|tag| normalize_tag(tag))
        .filter(|tag| seen.insert(tag.to_lowercase()))
        .take(provider.max_tags)
        .collect();
    analysis.dominant_colors = analysis
        .dominant_colors
        .iter()
        .map(|color| color.trim().to_uppercase())
        .filter(|color| {
            color.len() == 7
                && color.starts_with('#')
                && color.chars().skip(1).all(|character| character.is_ascii_hexdigit())
        })
        .take(8)
        .collect();
    analysis.title = analysis.title.trim().chars().take(180).collect();
    analysis.description = analysis.description.trim().chars().take(1200).collect();
    analysis.suggested_folder = analysis
        .suggested_folder
        .trim()
        .chars()
        .take(240)
        .collect();
    analysis.confidence = analysis.confidence.clamp(0.0, 1.0);
    analysis
}

fn parse_analysis(
    raw: &str,
    provider: &StoredOnlineAiProvider,
) -> Result<OnlineAiAnalysis, String> {
    let clean = strip_code_fence(raw);
    let analysis: OnlineAiAnalysis = serde_json::from_str(clean).map_err(|error| {
        format!(
            "AI 返回的 JSON 不符合格式：{}；原始内容：{}",
            error,
            clean.chars().take(800).collect::<String>()
        )
    })?;
    Ok(normalize_analysis(analysis, provider))
}

fn save_suggestions(
    file_id: i64,
    provider: &StoredOnlineAiProvider,
    analysis: &OnlineAiAnalysis,
    tags_applied: bool,
) -> Result<(), String> {
    ensure_ai_schema()?;
    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM dam_ai_suggestions WHERE file_id = ?1 AND provider = ?2 AND model = ?3 AND status = 'pending'",
            params![file_id, provider.id, provider.model],
        )
        .map_err(|error| error.to_string())?;
    let now = Utc::now().timestamp_millis();

    let mut insert = |kind: &str, value: &str, status: &str| -> Result<(), String> {
        if value.trim().is_empty() {
            return Ok(());
        }
        transaction
            .execute(
                "INSERT INTO dam_ai_suggestions (file_id, kind, value, confidence, provider, model, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    file_id,
                    kind,
                    value,
                    analysis.confidence,
                    provider.id,
                    provider.model,
                    status,
                    now
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    };

    insert("title", &analysis.title, "pending")?;
    insert("description", &analysis.description, "pending")?;
    insert("folder", &analysis.suggested_folder, "pending")?;
    for tag in &analysis.tags {
        insert("tag", tag, if tags_applied { "applied" } else { "pending" })?;
    }
    for color in &analysis.dominant_colors {
        insert("color", color, "pending")?;
    }
    drop(insert);
    transaction.commit().map_err(|error| error.to_string())
}

async fn analyze_internal(
    file_id: i64,
    provider: StoredOnlineAiProvider,
    force_auto_apply: Option<bool>,
) -> Result<OnlineAiAnalysisResult, String> {
    let image = load_image_payload(file_id)?;
    let raw = call_provider(&provider, Some(&image), false).await?;
    let analysis = parse_analysis(&raw, &provider)?;
    let should_apply = force_auto_apply.unwrap_or(provider.auto_apply_tags)
        && analysis.confidence >= provider.min_confidence
        && !analysis.tags.is_empty();
    let applied_tag_ids = if should_apply {
        t_dam::apply_tags(file_id, &analysis.tags)?
    } else {
        Vec::new()
    };
    let workflow_updated = if should_apply && provider.auto_mark_reviewed {
        t_dam::set_workflow_status(file_id, "reviewed")?;
        true
    } else {
        false
    };
    save_suggestions(file_id, &provider, &analysis, should_apply)?;

    Ok(OnlineAiAnalysisResult {
        file_id,
        provider_id: provider.id,
        provider_name: provider.name,
        model: provider.model,
        analysis,
        applied_tag_ids,
        tags_applied: should_apply,
        workflow_updated,
    })
}

fn update_job(
    job_id: i64,
    status: &str,
    error: Option<&str>,
    result: Option<&OnlineAiAnalysisResult>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();
    let result_json = result
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| error.to_string())?;
    let conn = t_sqlite::open_conn()?;
    conn.execute(
        "UPDATE dam_ai_jobs SET status = ?2, last_error = ?3, result_json = ?4, started_at = CASE WHEN ?2 = 'running' THEN COALESCE(started_at, ?5) ELSE started_at END, completed_at = CASE WHEN ?2 IN ('completed', 'failed') THEN ?5 ELSE completed_at END, attempts = CASE WHEN ?2 = 'running' THEN attempts + 1 ELSE attempts END WHERE id = ?1",
        params![job_id, status, error, result_json, now],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn process_job(
    job_id: i64,
    file_id: i64,
    provider: StoredOnlineAiProvider,
) -> Result<(), String> {
    update_job(job_id, "running", None, None)?;
    match analyze_internal(file_id, provider, None).await {
        Ok(result) => {
            update_job(job_id, "completed", None, Some(&result))?;
            Ok(())
        }
        Err(error) => {
            update_job(job_id, "failed", Some(&error), None)?;
            Err(error)
        }
    }
}

pub fn enqueue_capture_job(file_id: i64) -> Result<i64, String> {
    ensure_ai_schema()?;
    let conn = t_sqlite::open_conn()?;
    if let Some(existing_id) = conn
        .query_row(
            "SELECT id FROM dam_ai_jobs WHERE file_id = ?1 AND status IN ('queued', 'running', 'waiting_provider') ORDER BY id DESC LIMIT 1",
            params![file_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
    {
        return Ok(existing_id);
    }

    let provider = select_provider(None).ok();
    let status = if provider.is_some() {
        "queued"
    } else {
        "waiting_provider"
    };
    let provider_id = provider.as_ref().map(|provider| provider.id.clone());
    let now = Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO dam_ai_jobs (file_id, provider_id, source, status, created_at) VALUES (?1, ?2, 'browser_capture', ?3, ?4)",
        params![file_id, provider_id, status, now],
    )
    .map_err(|error| error.to_string())?;
    let job_id = conn.last_insert_rowid();
    drop(conn);

    if let Some(provider) = provider {
        tauri::async_runtime::spawn(async move {
            if let Err(error) = process_job(job_id, file_id, provider).await {
                eprintln!("Online AI organization job {} failed: {}", job_id, error);
            }
        });
    }
    Ok(job_id)
}

#[tauri::command]
pub async fn test_online_ai_provider(
    provider_id: String,
) -> Result<OnlineAiProviderTestResult, String> {
    let provider = select_provider(Some(&provider_id))?;
    let started = std::time::Instant::now();
    let raw = call_provider(&provider, None, true).await?;
    let _ = parse_analysis(&raw, &provider)?;
    Ok(OnlineAiProviderTestResult {
        ok: true,
        provider_id: provider.id,
        model: provider.model,
        elapsed_ms: started.elapsed().as_millis() as i64,
        message: "连接成功，模型返回了有效的结构化 JSON".to_string(),
    })
}

#[tauri::command]
pub async fn analyze_file_with_online_ai(
    file_id: i64,
    provider_id: String,
    force_auto_apply: Option<bool>,
) -> Result<OnlineAiAnalysisResult, String> {
    ensure_ai_schema()?;
    let provider = select_provider(Some(&provider_id))?;
    analyze_internal(file_id, provider, force_auto_apply).await
}
