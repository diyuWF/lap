use crate::t_config;
use crate::t_dam;
use crate::t_sqlite::{self, AFile, AThumb};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const PROVIDER_STORE_VERSION: u32 = 1;
const MAX_INLINE_IMAGE_BYTES: usize = 12 * 1024 * 1024;
const DEFAULT_SYSTEM_PROMPT_ZH: &str = r#"你是专业的数字资产管理员。请分析输入的视觉素材，为设计师素材库生成准确、克制、可检索的结构化元数据。
只返回一个 JSON 对象，不要使用 Markdown，不要解释。JSON 格式：
{
  "description": "一句话中文描述",
  "assetType": "最具体的素材类型，例如 产品图、家具、人物、纹理、场景、UI、图标、视频参考",
  "styles": ["风格词"],
  "materials": ["材质词"],
  "colors": ["主要颜色"],
  "tags": [
    {"name": "检索标签", "confidence": 0.0}
  ]
}
要求：标签不要重复；confidence 必须在 0 到 1 之间；不要猜测无法从画面判断的品牌、作者、版权或具体型号。"#;
const DEFAULT_SYSTEM_PROMPT_EN: &str = r#"You are a professional digital asset librarian. Analyze the visual asset and produce accurate, restrained, searchable metadata for a designer's local asset library.
Return one JSON object only, without Markdown or explanation. Use this schema:
{
  "description": "one-sentence description",
  "assetType": "the most specific asset type",
  "styles": ["style"],
  "materials": ["material"],
  "colors": ["dominant color"],
  "tags": [
    {"name": "search tag", "confidence": 0.0}
  ]
}
Do not duplicate tags. Confidence must be between 0 and 1. Do not invent brands, authors, licensing, or model numbers that cannot be verified visually."#;

static PROVIDER_STORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub auto_apply_tags: bool,
    #[serde(default)]
    pub auto_mark_reviewed: bool,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
    #[serde(default = "default_max_tags")]
    pub max_tags: usize,
    #[serde(default = "default_language")]
    pub language: String,
    pub system_prompt: Option<String>,
    #[serde(default = "default_auth_header")]
    pub auth_header: String,
    #[serde(default = "default_auth_prefix")]
    pub auth_prefix: String,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnlineAiProviderConfig {
    id: String,
    name: String,
    kind: OnlineAiProviderKind,
    base_url: String,
    model: String,
    api_key: Option<String>,
    enabled: bool,
    auto_apply_tags: bool,
    auto_mark_reviewed: bool,
    min_confidence: f64,
    max_tags: usize,
    language: String,
    system_prompt: Option<String>,
    auth_header: String,
    auth_prefix: String,
    extra_headers: HashMap<String, String>,
    created_at: i64,
    updated_at: i64,
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
    pub auto_apply_tags: bool,
    pub auto_mark_reviewed: bool,
    pub min_confidence: f64,
    pub max_tags: usize,
    pub language: String,
    pub system_prompt: Option<String>,
    pub auth_header: String,
    pub auth_prefix: String,
    pub extra_headers: HashMap<String, String>,
}

impl From<&OnlineAiProviderConfig> for OnlineAiProviderSummary {
    fn from(provider: &OnlineAiProviderConfig) -> Self {
        Self {
            id: provider.id.clone(),
            name: provider.name.clone(),
            kind: provider.kind,
            base_url: provider.base_url.clone(),
            model: provider.model.clone(),
            enabled: provider.enabled,
            has_api_key: provider
                .api_key
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            auto_apply_tags: provider.auto_apply_tags,
            auto_mark_reviewed: provider.auto_mark_reviewed,
            min_confidence: provider.min_confidence,
            max_tags: provider.max_tags,
            language: provider.language.clone(),
            system_prompt: provider.system_prompt.clone(),
            auth_header: provider.auth_header.clone(),
            auth_prefix: provider.auth_prefix.clone(),
            extra_headers: provider.extra_headers.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnlineAiProviderStore {
    version: u32,
    providers: Vec<OnlineAiProviderConfig>,
}

impl Default for OnlineAiProviderStore {
    fn default() -> Self {
        Self {
            version: PROVIDER_STORE_VERSION,
            providers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiTestResult {
    pub message: String,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiTagResult {
    pub name: String,
    pub confidence: f64,
    pub kind: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineAiAnalysisResult {
    pub file_id: i64,
    pub provider_id: String,
    pub model: String,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub tags: Vec<String>,
    pub tag_details: Vec<OnlineAiTagResult>,
    pub applied_tags: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ModelAnalysisPayload {
    description: Option<String>,
    #[serde(alias = "asset_type")]
    asset_type: Option<String>,
    #[serde(default, alias = "style")]
    styles: Vec<String>,
    #[serde(default)]
    materials: Vec<String>,
    #[serde(default)]
    colors: Vec<String>,
    #[serde(default)]
    tags: Vec<ModelTag>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ModelTag {
    Text(String),
    Detail(ModelTagDetail),
}

#[derive(Debug, Deserialize)]
struct ModelTagDetail {
    name: String,
    #[serde(default = "default_tag_confidence")]
    confidence: f64,
}

#[derive(Debug, Clone)]
struct TagSuggestion {
    name: String,
    confidence: f64,
    kind: String,
}

struct VisualInput {
    mime_type: String,
    data_base64: String,
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

fn default_auth_header() -> String {
    "Authorization".to_string()
}

fn default_auth_prefix() -> String {
    "Bearer ".to_string()
}

fn default_tag_confidence() -> f64 {
    0.75
}

fn provider_store_lock() -> &'static Mutex<()> {
    PROVIDER_STORE_LOCK.get_or_init(|| Mutex::new(()))
}

fn provider_store_path() -> Result<PathBuf, String> {
    let app_dir = t_config::get_app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|error| format!("Failed to create application data directory: {error}"))?;
    Ok(app_dir.join("online-ai-providers.json"))
}

fn load_provider_store_locked() -> Result<OnlineAiProviderStore, String> {
    let path = provider_store_path()?;
    if !path.exists() {
        return Ok(OnlineAiProviderStore::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read online AI configuration: {error}"))?;
    let mut store: OnlineAiProviderStore = serde_json::from_str(&content)
        .map_err(|error| format!("Failed to parse online AI configuration: {error}"))?;
    store.version = PROVIDER_STORE_VERSION;
    Ok(store)
}

fn save_provider_store_locked(store: &OnlineAiProviderStore) -> Result<(), String> {
    let path = provider_store_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "Online AI configuration path has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create online AI configuration directory: {error}"))?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_path = parent.join(format!(
        ".online-ai-providers.{}.{}.tmp",
        std::process::id(),
        stamp
    ));

    let content = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("Failed to serialize online AI configuration: {error}"))?;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp_path)
        .map_err(|error| format!("Failed to create online AI configuration: {error}"))?;
    file.write_all(&content)
        .map_err(|error| format!("Failed to write online AI configuration: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("Failed to sync online AI configuration: {error}"))?;
    drop(file);

    if let Err(first_error) = fs::rename(&temp_path, &path) {
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
        fs::rename(&temp_path, &path).map_err(|_| {
            let _ = fs::remove_file(&temp_path);
            format!("Failed to replace online AI configuration: {first_error}")
        })?;
    }
    Ok(())
}

fn normalize_provider_input(
    input: OnlineAiProviderInput,
    existing: Option<&OnlineAiProviderConfig>,
) -> Result<OnlineAiProviderConfig, String> {
    let name = input.name.trim().to_string();
    let base_url = input.base_url.trim().trim_end_matches('/').to_string();
    let model = input.model.trim().to_string();
    if name.is_empty() {
        return Err("AI 服务名称不能为空。".to_string());
    }
    if model.is_empty() {
        return Err("模型标识不能为空。".to_string());
    }
    if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
        return Err("API 地址必须以 http:// 或 https:// 开头。".to_string());
    }

    let api_key = input
        .api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| existing.and_then(|provider| provider.api_key.clone()));
    if matches!(
        input.kind,
        OnlineAiProviderKind::Gemini | OnlineAiProviderKind::Anthropic
    ) && api_key.is_none()
    {
        return Err("该接口类型必须填写 API 密钥。".to_string());
    }

    let now = Utc::now().timestamp_millis();
    let id = input
        .id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| existing.map(|provider| provider.id.clone()))
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let system_prompt = input
        .system_prompt
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let auth_header = if input.auth_header.trim().is_empty() {
        default_auth_header()
    } else {
        input.auth_header.trim().to_string()
    };

    Ok(OnlineAiProviderConfig {
        id,
        name,
        kind: input.kind,
        base_url,
        model,
        api_key,
        enabled: input.enabled,
        auto_apply_tags: input.auto_apply_tags,
        auto_mark_reviewed: input.auto_mark_reviewed,
        min_confidence: input.min_confidence.clamp(0.0, 1.0),
        max_tags: input.max_tags.clamp(1, 50),
        language: if input.language.trim().is_empty() {
            default_language()
        } else {
            input.language.trim().to_string()
        },
        system_prompt,
        auth_header,
        auth_prefix: input.auth_prefix,
        extra_headers: input
            .extra_headers
            .into_iter()
            .filter_map(|(name, value)| {
                let name = name.trim().to_string();
                let value = value.trim().to_string();
                (!name.is_empty() && !value.is_empty()).then_some((name, value))
            })
            .collect(),
        created_at: existing.map(|provider| provider.created_at).unwrap_or(now),
        updated_at: now,
    })
}

fn get_provider(provider_id: &str) -> Result<OnlineAiProviderConfig, String> {
    let _guard = provider_store_lock()
        .lock()
        .map_err(|_| "Online AI configuration lock is unavailable".to_string())?;
    let store = load_provider_store_locked()?;
    store
        .providers
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .ok_or_else(|| "未找到指定的在线 AI 服务。".to_string())
}

fn build_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .user_agent("Lap-DAM/0.4")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn build_headers(provider: &OnlineAiProviderConfig) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    match provider.kind {
        OnlineAiProviderKind::OpenaiCompatible => {
            if let Some(api_key) = provider.api_key.as_deref() {
                let name = HeaderName::from_bytes(provider.auth_header.as_bytes())
                    .map_err(|_| "自定义鉴权请求头名称无效。".to_string())?;
                let value = HeaderValue::from_str(&format!("{}{}", provider.auth_prefix, api_key))
                    .map_err(|_| "自定义鉴权请求头内容无效。".to_string())?;
                headers.insert(name, value);
            }
        }
        OnlineAiProviderKind::Gemini => {
            let api_key = provider
                .api_key
                .as_deref()
                .ok_or_else(|| "Gemini API 密钥为空。".to_string())?;
            headers.insert(
                HeaderName::from_static("x-goog-api-key"),
                HeaderValue::from_str(api_key)
                    .map_err(|_| "Gemini API 密钥格式无效。".to_string())?,
            );
        }
        OnlineAiProviderKind::Anthropic => {
            let api_key = provider
                .api_key
                .as_deref()
                .ok_or_else(|| "Anthropic API 密钥为空。".to_string())?;
            headers.insert(
                HeaderName::from_static("x-api-key"),
                HeaderValue::from_str(api_key)
                    .map_err(|_| "Anthropic API 密钥格式无效。".to_string())?,
            );
            headers.insert(
                HeaderName::from_static("anthropic-version"),
                HeaderValue::from_static("2023-06-01"),
            );
        }
    }

    for (name, value) in &provider.extra_headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| format!("额外请求头名称无效：{name}"))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|_| format!("额外请求头内容无效：{name}"))?;
        headers.insert(header_name, header_value);
    }
    Ok(headers)
}

fn openai_endpoint(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    }
}

fn gemini_endpoint(base_url: &str, model: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with(":generateContent") {
        base.to_string()
    } else {
        format!("{base}/models/{model}:generateContent")
    }
}

fn anthropic_endpoint(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/messages") {
        base.to_string()
    } else {
        format!("{base}/messages")
    }
}

fn response_error(status: reqwest::StatusCode, body: &str) -> String {
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let preview = compact.chars().take(800).collect::<String>();
    format!("AI 接口返回 HTTP {}：{}", status.as_u16(), preview)
}

async fn post_json(
    provider: &OnlineAiProviderConfig,
    url: String,
    body: JsonValue,
) -> Result<JsonValue, String> {
    let client = build_http_client()?;
    let response = client
        .post(url)
        .headers(build_headers(provider)?)
        .body(
            serde_json::to_vec(&body)
                .map_err(|error| format!("Failed to serialize AI request: {error}"))?,
        )
        .send()
        .await
        .map_err(|error| format!("AI 接口请求失败：{error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("读取 AI 接口响应失败：{error}"))?;
    if !status.is_success() {
        return Err(response_error(status, &text));
    }
    serde_json::from_str(&text).map_err(|error| {
        let preview = text.chars().take(800).collect::<String>();
        format!("AI 接口未返回有效 JSON：{error}；响应：{preview}")
    })
}

fn extract_openai_text(value: &JsonValue) -> Option<String> {
    if let Some(content) = value
        .pointer("/choices/0/message/content")
        .and_then(JsonValue::as_str)
    {
        return Some(content.to_string());
    }
    if let Some(parts) = value
        .pointer("/choices/0/message/content")
        .and_then(JsonValue::as_array)
    {
        let text = parts
            .iter()
            .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            return Some(text);
        }
    }
    value
        .get("output_text")
        .and_then(JsonValue::as_str)
        .map(str::to_string)
}

fn extract_gemini_text(value: &JsonValue) -> Option<String> {
    value
        .pointer("/candidates/0/content/parts")
        .and_then(JsonValue::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|text| !text.is_empty())
}

fn extract_anthropic_text(value: &JsonValue) -> Option<String> {
    value
        .get("content")
        .and_then(JsonValue::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter(|part| part.get("type").and_then(JsonValue::as_str) == Some("text"))
                .filter_map(|part| part.get("text").and_then(JsonValue::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
}

async fn request_model(
    provider: &OnlineAiProviderConfig,
    system_prompt: &str,
    user_prompt: &str,
    visual: Option<&VisualInput>,
) -> Result<String, String> {
    let (url, body) = match provider.kind {
        OnlineAiProviderKind::OpenaiCompatible => {
            let user_content = if let Some(visual) = visual {
                json!([
                    {"type": "text", "text": user_prompt},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{};base64,{}", visual.mime_type, visual.data_base64),
                            "detail": "auto"
                        }
                    }
                ])
            } else {
                json!(user_prompt)
            };
            (
                openai_endpoint(&provider.base_url),
                json!({
                    "model": provider.model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": user_content}
                    ],
                    "temperature": 0.1,
                    "max_tokens": 1200
                }),
            )
        }
        OnlineAiProviderKind::Gemini => {
            let mut parts = Vec::new();
            if let Some(visual) = visual {
                parts.push(json!({
                    "inline_data": {
                        "mime_type": visual.mime_type,
                        "data": visual.data_base64
                    }
                }));
            }
            parts.push(json!({"text": user_prompt}));
            (
                gemini_endpoint(&provider.base_url, &provider.model),
                json!({
                    "system_instruction": {"parts": [{"text": system_prompt}]},
                    "contents": [{"role": "user", "parts": parts}],
                    "generation_config": {
                        "temperature": 0.1,
                        "max_output_tokens": 1200,
                        "response_mime_type": "application/json"
                    }
                }),
            )
        }
        OnlineAiProviderKind::Anthropic => {
            let content = if let Some(visual) = visual {
                json!([
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": visual.mime_type,
                            "data": visual.data_base64
                        }
                    },
                    {"type": "text", "text": user_prompt}
                ])
            } else {
                json!([{"type": "text", "text": user_prompt}])
            };
            (
                anthropic_endpoint(&provider.base_url),
                json!({
                    "model": provider.model,
                    "max_tokens": 1200,
                    "temperature": 0.1,
                    "system": system_prompt,
                    "messages": [{"role": "user", "content": content}]
                }),
            )
        }
    };

    let value = post_json(provider, url, body).await?;
    let text = match provider.kind {
        OnlineAiProviderKind::OpenaiCompatible => extract_openai_text(&value),
        OnlineAiProviderKind::Gemini => extract_gemini_text(&value),
        OnlineAiProviderKind::Anthropic => extract_anthropic_text(&value),
    };
    text.filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "AI 接口响应中没有可读取的文本结果。".to_string())
}

fn detect_image_mime(path: &Path, bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, 0x50, 0x4e, 0x47]) {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => Some("image/jpeg"),
        Some("png") => Some("image/png"),
        Some("gif") => Some("image/gif"),
        Some("webp") => Some("image/webp"),
        Some("heic") => Some("image/heic"),
        Some("heif" | "hif") => Some("image/heif"),
        _ => None,
    }
}

fn prepare_visual_input(file_id: i64, file_path: &str) -> Result<VisualInput, String> {
    if let Some(thumb) = AThumb::fetch(file_id)? {
        if thumb.error_code == 0 {
            if let Some(bytes) = thumb.thumb_data {
                if !bytes.is_empty() && bytes.len() <= MAX_INLINE_IMAGE_BYTES {
                    let mime_type = detect_image_mime(Path::new(file_path), &bytes)
                        .unwrap_or("image/jpeg")
                        .to_string();
                    return Ok(VisualInput {
                        mime_type,
                        data_base64: general_purpose::STANDARD.encode(bytes),
                    });
                }
            }
        }
    }

    let bytes =
        fs::read(file_path).map_err(|error| format!("读取待分析素材失败：{error}"))?;
    if bytes.len() > MAX_INLINE_IMAGE_BYTES {
        return Err(format!(
            "素材没有可用缩略图，且原文件超过 {} MB。请等待缩略图生成后重试。",
            MAX_INLINE_IMAGE_BYTES / 1024 / 1024
        ));
    }
    let mime_type = detect_image_mime(Path::new(file_path), &bytes).ok_or_else(|| {
        "当前在线 AI 分析需要 JPEG、PNG、GIF、WebP、HEIC/HEIF 文件或已生成的 Lap 缩略图。"
            .to_string()
    })?;
    Ok(VisualInput {
        mime_type: mime_type.to_string(),
        data_base64: general_purpose::STANDARD.encode(bytes),
    })
}

fn strip_json_fence(raw: &str) -> &str {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return trimmed;
    }
    let without_open = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_start();
    without_open
        .strip_suffix("```")
        .unwrap_or(without_open)
        .trim_end()
}

fn parse_analysis_payload(raw: &str) -> Result<ModelAnalysisPayload, String> {
    let stripped = strip_json_fence(raw);
    if let Ok(payload) = serde_json::from_str::<ModelAnalysisPayload>(stripped) {
        return Ok(payload);
    }
    let start = stripped.find('{');
    let end = stripped.rfind('}');
    if let (Some(start), Some(end)) = (start, end) {
        if start < end {
            return serde_json::from_str::<ModelAnalysisPayload>(&stripped[start..=end])
                .map_err(|error| format!("AI 返回的素材元数据 JSON 无法解析：{error}"));
        }
    }
    Err("AI 没有返回可解析的素材元数据 JSON。".to_string())
}

fn clean_text(value: &str) -> Option<String> {
    let cleaned = value
        .trim()
        .trim_matches(|character: char| character == ',' || character == '，')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

fn suggestion_kind(name: &str) -> String {
    match name.split_once(':').map(|(prefix, _)| prefix) {
        Some("类型") | Some("type") => "asset_type".to_string(),
        Some("风格") | Some("style") => "style".to_string(),
        Some("材质") | Some("material") => "material".to_string(),
        Some("颜色") | Some("color") => "color".to_string(),
        _ => "tag".to_string(),
    }
}

fn build_suggestions(payload: &ModelAnalysisPayload, max_tags: usize) -> Vec<TagSuggestion> {
    let mut candidates = Vec::new();
    if let Some(asset_type) = payload.asset_type.as_deref().and_then(clean_text) {
        candidates.push(TagSuggestion {
            name: format!("类型:{asset_type}"),
            confidence: 0.95,
            kind: "asset_type".to_string(),
        });
    }
    for style in &payload.styles {
        if let Some(style) = clean_text(style) {
            candidates.push(TagSuggestion {
                name: format!("风格:{style}"),
                confidence: 0.9,
                kind: "style".to_string(),
            });
        }
    }
    for material in &payload.materials {
        if let Some(material) = clean_text(material) {
            candidates.push(TagSuggestion {
                name: format!("材质:{material}"),
                confidence: 0.9,
                kind: "material".to_string(),
            });
        }
    }
    for color in &payload.colors {
        if let Some(color) = clean_text(color) {
            candidates.push(TagSuggestion {
                name: format!("颜色:{color}"),
                confidence: 0.85,
                kind: "color".to_string(),
            });
        }
    }
    for tag in &payload.tags {
        let (name, confidence) = match tag {
            ModelTag::Text(name) => (name.as_str(), default_tag_confidence()),
            ModelTag::Detail(detail) => (detail.name.as_str(), detail.confidence),
        };
        if let Some(name) = clean_text(name) {
            candidates.push(TagSuggestion {
                kind: suggestion_kind(&name),
                name,
                confidence: confidence.clamp(0.0, 1.0),
            });
        }
    }

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|suggestion| seen.insert(suggestion.name.to_lowercase()))
        .take(max_tags.clamp(1, 50))
        .collect()
}

fn persist_ai_suggestions(
    file_id: i64,
    provider: &OnlineAiProviderConfig,
    suggestions: &[TagSuggestion],
    applied_names: &HashSet<String>,
) -> Result<(), String> {
    t_dam::ensure_schema()?;
    let mut conn = t_sqlite::open_conn()?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM dam_ai_suggestions WHERE file_id = ?1 AND provider = ?2 AND model = ?3 AND status = 'pending'",
            rusqlite::params![file_id, provider.id, provider.model],
        )
        .map_err(|error| error.to_string())?;
    let created_at = Utc::now().timestamp_millis();
    {
        let mut statement = transaction
            .prepare_cached(
                "INSERT INTO dam_ai_suggestions (file_id, kind, value, confidence, provider, model, status, created_at, reviewed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .map_err(|error| error.to_string())?;
        for suggestion in suggestions {
            let is_applied = applied_names.contains(&suggestion.name.to_lowercase());
            statement
                .execute(rusqlite::params![
                    file_id,
                    suggestion.kind,
                    suggestion.name,
                    suggestion.confidence,
                    provider.id,
                    provider.model,
                    if is_applied { "applied" } else { "pending" },
                    created_at,
                    if is_applied { Some(created_at) } else { None },
                ])
                .map_err(|error| error.to_string())?;
        }
    }
    transaction
        .commit()
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn analysis_system_prompt(provider: &OnlineAiProviderConfig) -> String {
    provider.system_prompt.clone().unwrap_or_else(|| {
        if provider.language.to_ascii_lowercase().starts_with("zh") {
            DEFAULT_SYSTEM_PROMPT_ZH.to_string()
        } else {
            DEFAULT_SYSTEM_PROMPT_EN.to_string()
        }
    })
}

#[tauri::command]
pub fn list_online_ai_providers() -> Result<Vec<OnlineAiProviderSummary>, String> {
    let _guard = provider_store_lock()
        .lock()
        .map_err(|_| "Online AI configuration lock is unavailable".to_string())?;
    let mut providers = load_provider_store_locked()?
        .providers
        .iter()
        .map(OnlineAiProviderSummary::from)
        .collect::<Vec<_>>();
    providers.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(providers)
}

#[tauri::command]
pub fn save_online_ai_provider(
    input: OnlineAiProviderInput,
) -> Result<OnlineAiProviderSummary, String> {
    let _guard = provider_store_lock()
        .lock()
        .map_err(|_| "Online AI configuration lock is unavailable".to_string())?;
    let mut store = load_provider_store_locked()?;
    let existing_index = input
        .id
        .as_deref()
        .and_then(|id| store.providers.iter().position(|provider| provider.id == id));
    let existing = existing_index.and_then(|index| store.providers.get(index));
    let provider = normalize_provider_input(input, existing)?;
    if let Some(index) = existing_index {
        store.providers[index] = provider.clone();
    } else {
        store.providers.push(provider.clone());
    }
    save_provider_store_locked(&store)?;
    Ok(OnlineAiProviderSummary::from(&provider))
}

#[tauri::command]
pub fn delete_online_ai_provider(provider_id: String) -> Result<bool, String> {
    let _guard = provider_store_lock()
        .lock()
        .map_err(|_| "Online AI configuration lock is unavailable".to_string())?;
    let mut store = load_provider_store_locked()?;
    let previous_len = store.providers.len();
    store.providers.retain(|provider| provider.id != provider_id);
    let removed = store.providers.len() != previous_len;
    if removed {
        save_provider_store_locked(&store)?;
    }
    Ok(removed)
}

#[tauri::command]
pub async fn test_online_ai_provider(
    provider_id: String,
) -> Result<OnlineAiTestResult, String> {
    let provider = get_provider(&provider_id)?;
    let started = Instant::now();
    let response = request_model(
        &provider,
        "You are a connection test endpoint.",
        "Reply with exactly OK.",
        None,
    )
    .await?;
    if response.trim().is_empty() {
        return Err("AI 接口连接成功，但没有返回文本。".to_string());
    }
    Ok(OnlineAiTestResult {
        message: "连接成功".to_string(),
        elapsed_ms: started.elapsed().as_millis(),
    })
}

#[tauri::command]
pub async fn analyze_file_with_online_ai(
    file_id: i64,
    provider_id: String,
    force_auto_apply: Option<bool>,
) -> Result<OnlineAiAnalysisResult, String> {
    let provider = get_provider(&provider_id)?;
    if !provider.enabled {
        return Err("该在线 AI 服务尚未启用。".to_string());
    }
    let file =
        AFile::get_file_info(file_id)?.ok_or_else(|| "未找到待分析素材。".to_string())?;
    let file_path = file
        .file_path
        .as_deref()
        .ok_or_else(|| "待分析素材没有可读取的本地路径。".to_string())?;
    let visual = prepare_visual_input(file_id, file_path)?;
    let user_prompt = format!(
        "请分析这个素材。文件名：{}。优先生成对设计、3D、UI、摄影和参考图检索有价值的标签。",
        file.name
    );
    let raw = request_model(
        &provider,
        &analysis_system_prompt(&provider),
        &user_prompt,
        Some(&visual),
    )
    .await?;
    let payload = parse_analysis_payload(&raw)?;
    let suggestions = build_suggestions(&payload, provider.max_tags);
    if suggestions.is_empty() {
        return Err("AI 已返回分析结果，但没有生成有效标签。".to_string());
    }

    let should_apply = force_auto_apply.unwrap_or(provider.auto_apply_tags);
    let applied_tags = if should_apply {
        let tags = suggestions
            .iter()
            .filter(|suggestion| suggestion.confidence >= provider.min_confidence)
            .map(|suggestion| suggestion.name.clone())
            .collect::<Vec<_>>();
        if tags.is_empty() {
            Vec::new()
        } else {
            t_dam::apply_tags(file_id, &tags)?;
            if provider.auto_mark_reviewed {
                t_dam::set_workflow_status(file_id, "reviewed")?;
            }
            tags
        }
    } else {
        Vec::new()
    };
    let applied_names = applied_tags
        .iter()
        .map(|name| name.to_lowercase())
        .collect::<HashSet<_>>();
    persist_ai_suggestions(file_id, &provider, &suggestions, &applied_names)?;

    let tag_details = suggestions
        .iter()
        .map(|suggestion| OnlineAiTagResult {
            name: suggestion.name.clone(),
            confidence: suggestion.confidence,
            kind: suggestion.kind.clone(),
            applied: applied_names.contains(&suggestion.name.to_lowercase()),
        })
        .collect::<Vec<_>>();
    Ok(OnlineAiAnalysisResult {
        file_id,
        provider_id: provider.id,
        model: provider.model,
        description: payload.description.and_then(|value| clean_text(&value)),
        asset_type: payload.asset_type.and_then(|value| clean_text(&value)),
        tags: suggestions
            .iter()
            .map(|suggestion| suggestion.name.clone())
            .collect(),
        tag_details,
        applied_tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_json_fence() {
        let raw = "```json\n{\"assetType\":\"chair\",\"tags\":[]}\n```";
        let parsed = parse_analysis_payload(raw).expect("payload should parse");
        assert_eq!(parsed.asset_type.as_deref(), Some("chair"));
    }

    #[test]
    fn builds_deduplicated_taxonomy_tags() {
        let payload = ModelAnalysisPayload {
            asset_type: Some("椅子".to_string()),
            styles: vec!["现代".to_string()],
            materials: vec!["木材".to_string()],
            colors: vec!["黑色".to_string()],
            tags: vec![
                ModelTag::Text("家具".to_string()),
                ModelTag::Detail(ModelTagDetail {
                    name: "家具".to_string(),
                    confidence: 0.9,
                }),
            ],
            ..Default::default()
        };
        let suggestions = build_suggestions(&payload, 20);
        assert_eq!(
            suggestions
                .iter()
                .filter(|item| item.name == "家具")
                .count(),
            1
        );
        assert!(suggestions.iter().any(|item| item.name == "类型:椅子"));
        assert!(suggestions.iter().any(|item| item.name == "风格:现代"));
    }

    #[test]
    fn appends_provider_endpoints_once() {
        assert_eq!(
            openai_endpoint("https://example.com/v1"),
            "https://example.com/v1/chat/completions"
        );
        assert_eq!(
            openai_endpoint("https://example.com/v1/chat/completions"),
            "https://example.com/v1/chat/completions"
        );
        assert_eq!(
            gemini_endpoint("https://example.com/v1beta", "gemini-test"),
            "https://example.com/v1beta/models/gemini-test:generateContent"
        );
        assert_eq!(
            anthropic_endpoint("https://example.com/v1"),
            "https://example.com/v1/messages"
        );
    }
}
