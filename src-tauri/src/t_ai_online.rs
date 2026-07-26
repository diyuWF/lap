use crate::t_config;
use crate::t_dam;
use crate::t_sqlite::{self, AFile, AThumb};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderName, HeaderValue};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

const CONFIG_VERSION: u32 = 1;
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const DEFAULT_SYSTEM_PROMPT: &str = r##"你是专业的数字素材库管理员。分析图像并且只输出严格 JSON，不要输出 Markdown。标签必须简洁、稳定、可复用，优先使用 namespace:value 形式，例如 subject:car、style:minimal、composition:centered、lighting:studio、color:blue、usage:ui-reference。不要虚构人物、作者或项目名称。返回结构：{"title":"","description":"","tags":["namespace:value"],"dominantColors":["#RRGGBB"],"confidence":0.0}。"##;

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
    pub has_api_key: bool,
    pub api_key_preview: String,
    pub auth_header: String,
    pub auth_prefix: String,
    pub enabled: bool,
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
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiAnalysisResult {
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub file_id: i64,
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
    pub message: String,
    pub elapsed_ms: i64,
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
    0.75
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
    let data = fs::read(&path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&data)
        .map_err(|error| format!("在线 AI 设置文件解析失败：{}", error))
}

fn save_config(config: &StoredOnlineAiConfig) -> Result<(), String> {
    let path = config_path()?;
    let temporary = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&temporary, data).map_err(|error| error.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }

    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, &path).map_err(|error| error.to_string())
}

fn normalize_http_url(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    if value.starts_with("https://") || value.starts_with("http://") {
        Ok(value.to_string())
    } else {
        Err("API 地址必须以 http:// 或 https:// 开头".to_string())
    }
}

fn provider_summary(provider: &StoredOnlineAiProvider) -> OnlineAiProviderSummary {
    let preview = if provider.api_key.is_empty() {
        String::new()
    } else if provider.api_key.chars().count() <= 8 {
        "********".to_string()
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
        has_api_key: !provider.api_key.is_empty(),
        api_key_preview: preview,
        auth_header: provider.auth_header.clone(),
        auth_prefix: provider.auth_prefix.clone(),
        enabled: provider.enabled,
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

#[tauri::command]
pub fn list_online_ai_providers() -> Result<Vec<OnlineAiProviderSummary>, String> {
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
    let mut config = load_config()?;
    let now = Utc::now().timestamp_millis();
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

    let supplied_key = input.api_key.unwrap_or_default().trim().to_string();
    let api_key = if supplied_key.is_empty() {
        existing
            .as_ref()
            .map(|provider| provider.api_key.clone())
            .unwrap_or_default()
    } else {
        supplied_key
    };
    let name = input.name.trim();
    let model = input.model.trim();
    if name.is_empty() {
        return Err("服务名称不能为空".to_string());
    }
    if model.is_empty() {
        return Err("模型名称不能为空".to_string());
    }

    let provider = StoredOnlineAiProvider {
        id: provider_id.clone(),
        name: name.to_string(),
        kind: input.kind,
        base_url: normalize_http_url(&input.base_url)?,
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
    let previous_len = config.providers.len();
    config.providers.retain(|provider| provider.id != provider_id);
    let changed = config.providers.len() != previous_len;
    if changed {
        save_config(&config)?;
    }
    Ok(changed)
}

fn find_provider(provider_id: &str) -> Result<StoredOnlineAiProvider, String> {
    let provider = load_config()?
        .providers
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .ok_or_else(|| "未找到指定的在线 AI 服务".to_string())?;
    if !provider.enabled {
        return Err("该在线 AI 服务已停用".to_string());
    }
    if provider.api_key.trim().is_empty() {
        return Err("该在线 AI 服务尚未配置 API 密钥".to_string());
    }
    Ok(provider)
}

struct ImagePayload {
    mime: String,
    base64: String,
}

fn mime_for_path(path: &str) -> Option<&'static str> {
    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

fn thumbnail_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else {
        "image/jpeg"
    }
}

fn load_image_payload(file_id: i64) -> Result<ImagePayload, String> {
    let file = AFile::get_file_info(file_id)?
        .ok_or_else(|| format!("未找到文件：{}", file_id))?;
    let file_path = file
        .file_path
        .clone()
        .ok_or_else(|| "文件路径不可用".to_string())?;

    if let Some(mime) = mime_for_path(&file_path) {
        let bytes = fs::read(&file_path).map_err(|error| error.to_string())?;
        if bytes.len() <= MAX_IMAGE_BYTES {
            return Ok(ImagePayload {
                mime: mime.to_string(),
                base64: STANDARD.encode(bytes),
            });
        }
    }

    let thumbnail = AThumb::get_or_create_thumb(
        file_id,
        &file_path,
        file.file_type.unwrap_or_default(),
        file.e_orientation.unwrap_or(1) as i32,
        1024,
        false,
        file.duration.map(|value| value as u64),
        None,
    )?
    .ok_or_else(|| "无法为该文件生成 AI 分析预览图".to_string())?;
    let bytes = thumbnail
        .thumb_data
        .ok_or_else(|| "AI 分析预览图为空".to_string())?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("AI 分析预览图超过 20 MB".to_string());
    }
    Ok(ImagePayload {
        mime: thumbnail_mime(&bytes).to_string(),
        base64: STANDARD.encode(bytes),
    })
}

fn analysis_prompt(provider: &StoredOnlineAiProvider) -> String {
    format!(
        "使用 {} 输出结果，最多生成 {} 个标签。标签优先使用 namespace:value 形式。只输出 JSON。",
        provider.language, provider.max_tags
    )
}

fn endpoint(base_url: &str, suffix: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    let suffix = suffix.trim().trim_start_matches('/');
    if base
        .to_ascii_lowercase()
        .ends_with(&suffix.to_ascii_lowercase())
    {
        base.to_string()
    } else {
        format!("{}/{}", base, suffix)
    }
}

fn is_openrouter(base_url: &str) -> bool {
    reqwest::Url::parse(base_url.trim())
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .map(|host| host.eq_ignore_ascii_case("openrouter.ai"))
        .unwrap_or(false)
}

fn openai_compatible_endpoint(base_url: &str) -> String {
    if is_openrouter(base_url) {
        "https://openrouter.ai/api/v1/chat/completions".to_string()
    } else {
        endpoint(base_url, "chat/completions")
    }
}

fn apply_extra_headers(
    mut request: reqwest::RequestBuilder,
    provider: &StoredOnlineAiProvider,
) -> Result<reqwest::RequestBuilder, String> {
    for (name, value) in &provider.extra_headers {
        let header_name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|error| format!("无效的 Header {}：{}", name, error))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|error| format!("无效的 Header 值 {}：{}", name, error))?;
        request = request.header(header_name, header_value);
    }
    Ok(request)
}

fn compact_service_message(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn ai_http_error(
    status: reqwest::StatusCode,
    value: &JsonValue,
    provider: &StoredOnlineAiProvider,
    url: &str,
) -> String {
    let metadata = &value["error"]["metadata"];
    let provider_name = metadata["provider_name"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let raw = metadata["raw"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let message = value["error"]["message"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let upstream_limited = raw
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("temporarily rate-limited upstream");
        if upstream_limited {
            let upstream = provider_name
                .map(|name| format!("在 {} 上游", name))
                .unwrap_or_else(|| "在上游".to_string());
            return format!(
                "当前模型 {} {}暂时限流。请稍后重试，或在 OpenRouter 配置自有上游密钥/更换可用模型。",
                provider.model,
                upstream
            );
        }
        return format!(
            "AI 服务请求过于频繁（HTTP 429）。请稍后重试，或检查 {} 的额度与限流设置。",
            provider.name
        );
    }

    if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        return format!(
            "AI 服务认证失败（HTTP {}）。请检查 API 密钥、账户权限和服务商配置。",
            status.as_u16()
        );
    }

    let detail = raw
        .or(message)
        .map(compact_service_message)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "服务商没有返回可读的错误说明".to_string());
    format!(
        "AI 服务请求失败（HTTP {}）：{}。请求地址：{}",
        status.as_u16(),
        detail,
        url
    )
}

async fn send_request(
    provider: &StoredOnlineAiProvider,
    url: &str,
    body: JsonValue,
    native_headers: &[(&str, String)],
    use_custom_auth: bool,
) -> Result<JsonValue, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())?;
    let mut request = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(serde_json::to_vec(&body).map_err(|error| error.to_string())?);

    if use_custom_auth && !provider.auth_header.trim().is_empty() {
        let name = HeaderName::from_bytes(provider.auth_header.trim().as_bytes())
            .map_err(|error| format!("无效的认证 Header：{}", error))?;
        let value = HeaderValue::from_str(&format!(
            "{}{}",
            provider.auth_prefix, provider.api_key
        ))
        .map_err(|error| format!("无效的认证值：{}", error))?;
        request = request.header(name, value);
    }
    for (name, value) in native_headers {
        request = request.header(*name, value);
    }
    request = apply_extra_headers(request, provider)?;

    let response = request
        .send()
        .await
        .map_err(|error| format!("AI 服务连接失败：{}；请求地址：{}", error, url))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("未知")
        .to_string();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let response_start = String::from_utf8_lossy(&bytes)
        .trim_start()
        .chars()
        .take(32)
        .collect::<String>()
        .to_ascii_lowercase();
    let looks_like_html = content_type.to_ascii_lowercase().contains("text/html")
        || response_start.starts_with("<!doctype html")
        || response_start.starts_with("<html");

    if looks_like_html {
        return Err(format!(
            "AI 服务返回了网页 HTML 而不是 JSON（HTTP {}）。请求地址：{}。请检查 API 地址是否为服务商提供的 API 基础地址。",
            status, url
        ));
    }

    let value: JsonValue = serde_json::from_slice(&bytes).map_err(|error| {
        let preview = String::from_utf8_lossy(&bytes)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(240)
            .collect::<String>();
        format!(
            "AI 服务返回内容无法解析（HTTP {}，响应类型 {}）：{}。请求地址：{}。响应摘要：{}",
            status, content_type, error, url, preview
        )
    })?;
    if !status.is_success() {
        return Err(ai_http_error(status, &value, provider, url));
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
    Err(format!("OpenAI 兼容响应中没有文本：{}", value))
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
) -> Result<String, String> {
    let prompt = analysis_prompt(provider);
    match provider.kind {
        OnlineAiProviderKind::OpenaiCompatible => {
            let url = openai_compatible_endpoint(&provider.base_url);
            let mut content = vec![json!({ "type": "text", "text": prompt })];
            if let Some(image) = image {
                content.push(json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:{};base64,{}", image.mime, image.base64),
                        "detail": "auto"
                    }
                }));
            }
            let body = json!({
                "model": provider.model,
                "temperature": 0.1,
                "messages": [
                    { "role": "system", "content": provider.system_prompt },
                    { "role": "user", "content": content }
                ]
            });
            let headers = if is_openrouter(&provider.base_url) {
                vec![
                    ("HTTP-Referer", "https://github.com/diyuWF/lap".to_string()),
                    ("X-OpenRouter-Title", "Lap".to_string()),
                ]
            } else {
                Vec::new()
            };
            extract_openai_text(&send_request(provider, &url, body, &headers, true).await?)
        }
        OnlineAiProviderKind::Gemini => {
            let url = endpoint(
                &provider.base_url,
                &format!("models/{}:generateContent", provider.model),
            );
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
            let headers = vec![("x-goog-api-key", provider.api_key.clone())];
            extract_gemini_text(&send_request(provider, &url, body, &headers, false).await?)
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
            let headers = vec![
                ("x-api-key", provider.api_key.clone()),
                ("anthropic-version", "2023-06-01".to_string()),
            ];
            extract_anthropic_text(&send_request(provider, &url, body, &headers, false).await?)
        }
    }
}

fn strip_json_fence(value: &str) -> &str {
    let trimmed = value.trim();
    if !trimmed.starts_with("```") {
        return trimmed;
    }
    let without_start = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_start();
    without_start
        .strip_suffix("```")
        .unwrap_or(without_start)
        .trim()
}

fn normalize_analysis(
    mut analysis: OnlineAiAnalysis,
    provider: &StoredOnlineAiProvider,
) -> OnlineAiAnalysis {
    let mut seen = HashSet::new();
    analysis.tags = analysis
        .tags
        .into_iter()
        .map(|tag| tag.trim().trim_matches('#').to_string())
        .filter(|tag| !tag.is_empty())
        .filter(|tag| seen.insert(tag.to_lowercase()))
        .take(provider.max_tags)
        .collect();
    analysis.dominant_colors = analysis
        .dominant_colors
        .into_iter()
        .map(|color| color.trim().to_uppercase())
        .filter(|color| {
            color.len() == 7
                && color.starts_with('#')
                && color
                    .chars()
                    .skip(1)
                    .all(|character| character.is_ascii_hexdigit())
        })
        .take(8)
        .collect();
    analysis.confidence = analysis.confidence.clamp(0.0, 1.0);
    analysis.title = analysis.title.trim().chars().take(180).collect();
    analysis.description = analysis.description.trim().chars().take(1200).collect();
    analysis
}

fn parse_analysis(
    raw: &str,
    provider: &StoredOnlineAiProvider,
) -> Result<OnlineAiAnalysis, String> {
    let clean = strip_json_fence(raw);
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
    t_dam::ensure_schema()?;
    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM dam_ai_suggestions WHERE file_id = ?1 AND provider = ?2 AND model = ?3 AND status = 'pending'",
            params![file_id, provider.id, provider.model],
        )
        .map_err(|error| error.to_string())?;
    let now = Utc::now().timestamp_millis();

    let insert = |kind: &str, value: &str, status: &str| -> Result<(), String> {
        if value.trim().is_empty() {
            return Ok(());
        }
        transaction
            .execute(
                "INSERT INTO dam_ai_suggestions (file_id, kind, value, confidence, provider, model, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![file_id, kind, value, analysis.confidence, provider.id, provider.model, status, now],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    };

    insert("title", &analysis.title, "pending")?;
    insert("description", &analysis.description, "pending")?;
    for tag in &analysis.tags {
        insert("tag", tag, if tags_applied { "applied" } else { "pending" })?;
    }
    for color in &analysis.dominant_colors {
        insert("color", color, "pending")?;
    }
    transaction.commit().map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn test_online_ai_provider(
    provider_id: String,
) -> Result<OnlineAiProviderTestResult, String> {
    let provider = find_provider(&provider_id)?;
    let started = Instant::now();
    let raw = call_provider(&provider, None).await?;
    let _ = parse_analysis(&raw, &provider)?;
    Ok(OnlineAiProviderTestResult {
        ok: true,
        provider_id: provider.id,
        model: provider.model,
        message: "连接成功，模型返回了有效的结构化 JSON".to_string(),
        elapsed_ms: started.elapsed().as_millis() as i64,
    })
}

#[tauri::command]
pub async fn analyze_file_with_online_ai(
    file_id: i64,
    provider_id: String,
    force_auto_apply: Option<bool>,
) -> Result<OnlineAiAnalysisResult, String> {
    let provider = find_provider(&provider_id)?;
    let image = load_image_payload(file_id)?;
    let raw = call_provider(&provider, Some(&image)).await?;
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
        provider_id: provider.id,
        provider_name: provider.name,
        model: provider.model,
        file_id,
        analysis,
        applied_tag_ids,
        tags_applied: should_apply,
        workflow_updated,
    })
}
