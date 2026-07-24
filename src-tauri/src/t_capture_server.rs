use crate::t_config;
use crate::t_dam::{self, CaptureSourceMetadata, DamFolder};
use crate::t_sqlite::AFile;
use crate::t_utils;
use chrono::Utc;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

const DEFAULT_CAPTURE_PORT: u16 = 47_821;
const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

static CAPTURE_SERVER_INFO: OnceLock<CaptureServerInfo> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureServerInfo {
    pub base_url: String,
    pub token: String,
    pub port: u16,
    pub api_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRequest {
    pub source_url: String,
    pub page_url: Option<String>,
    pub page_title: Option<String>,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub alt_text: Option<String>,
    pub folder_path: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub allow_duplicate: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub ok: bool,
    pub duplicate: bool,
    pub file_id: Option<i64>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub folder: Option<DamFolder>,
    pub applied_tag_ids: Vec<i64>,
    pub message: String,
}

pub fn get_capture_server_info() -> Option<CaptureServerInfo> {
    CAPTURE_SERVER_INFO.get().cloned()
}

pub fn init_capture_server() {
    if CAPTURE_SERVER_INFO.get().is_some() {
        return;
    }

    let token = match load_or_create_token() {
        Ok(token) => token,
        Err(error) => {
            eprintln!("Failed to initialize browser capture token: {}", error);
            return;
        }
    };
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let server_token = token.clone();

    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", DEFAULT_CAPTURE_PORT)).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("Failed to bind browser capture server: {}", error);
                let _ = sender.send(None);
                return;
            }
        };
        let port = listener
            .local_addr()
            .map(|address| address.port())
            .unwrap_or(DEFAULT_CAPTURE_PORT);
        let info = CaptureServerInfo {
            base_url: format!("http://127.0.0.1:{}", port),
            token: server_token.clone(),
            port,
            api_version: 1,
        };
        let _ = sender.send(Some(info));

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            let request_token = server_token.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = handle_connection(stream, &request_token).await {
                    eprintln!("Browser capture request failed: {}", error);
                }
            });
        }
    });

    match receiver.recv_timeout(Duration::from_secs(3)) {
        Ok(Some(info)) => {
            let _ = persist_server_info(&info);
            let _ = CAPTURE_SERVER_INFO.set(info);
        }
        Ok(None) => eprintln!("Browser capture server could not start."),
        Err(error) => eprintln!("Timed out starting browser capture server: {}", error),
    }
}

fn capture_config_dir() -> Result<PathBuf, String> {
    let dir = t_config::get_app_data_dir()?.join("browser-capture");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn load_or_create_token() -> Result<String, String> {
    let token_path = capture_config_dir()?.join("token.txt");
    if let Ok(existing) = std::fs::read_to_string(&token_path) {
        let token = existing.trim();
        if token.len() >= 32 && token.chars().all(|character| character.is_ascii_alphanumeric()) {
            return Ok(token.to_string());
        }
    }

    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    std::fs::write(&token_path, &token).map_err(|error| error.to_string())?;
    Ok(token)
}

fn persist_server_info(info: &CaptureServerInfo) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(info).map_err(|error| error.to_string())?;
    std::fs::write(capture_config_dir()?.join("server.json"), data)
        .map_err(|error| error.to_string())
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    expected_token: &str,
) -> Result<(), String> {
    let request = read_request(&mut stream).await?;

    if request.method == "OPTIONS" {
        return write_json(&mut stream, "204 No Content", &json!({})).await;
    }

    if request.method == "GET" && request.path == "/health" {
        return write_json(
            &mut stream,
            "200 OK",
            &json!({ "ok": true, "apiVersion": 1, "app": "Lap" }),
        )
        .await;
    }

    let supplied_token = request
        .headers
        .get("x-lap-token")
        .map(String::as_str)
        .unwrap_or_default();
    if supplied_token != expected_token {
        return write_json(
            &mut stream,
            "401 Unauthorized",
            &json!({ "ok": false, "error": "Invalid pairing token" }),
        )
        .await;
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/folders") => match t_dam::list_folders() {
            Ok(folders) => write_json(
                &mut stream,
                "200 OK",
                &json!({ "ok": true, "folders": folders }),
            )
            .await,
            Err(error) => write_json(
                &mut stream,
                "500 Internal Server Error",
                &json!({ "ok": false, "error": error }),
            )
            .await,
        },
        ("POST", "/capture") => {
            let capture_request: CaptureRequest = match serde_json::from_slice(&request.body) {
                Ok(value) => value,
                Err(error) => {
                    return write_json(
                        &mut stream,
                        "400 Bad Request",
                        &json!({ "ok": false, "error": format!("Invalid JSON: {}", error) }),
                    )
                    .await;
                }
            };
            match capture_remote_asset(capture_request).await {
                Ok(result) => write_json(&mut stream, "200 OK", &result).await,
                Err(error) => write_json(
                    &mut stream,
                    "422 Unprocessable Entity",
                    &json!({ "ok": false, "error": error }),
                )
                .await,
            }
        }
        _ => {
            write_json(
                &mut stream,
                "404 Not Found",
                &json!({ "ok": false, "error": "Route not found" }),
            )
            .await
        }
    }
}

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Result<HttpRequest, String> {
    let mut data = Vec::new();
    let mut buffer = [0u8; 8192];
    let mut header_end = None;
    let mut content_length = 0usize;

    loop {
        let read = stream.read(&mut buffer).await.map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..read]);
        if data.len() > MAX_REQUEST_BYTES {
            return Err("Request body is too large".to_string());
        }

        if header_end.is_none() {
            header_end = find_header_end(&data);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&data[..end]);
                content_length = parse_content_length(&headers).unwrap_or(0);
            }
        }

        if let Some(end) = header_end {
            if data.len() >= end + 4 + content_length {
                break;
            }
        }
    }

    let header_end = header_end.ok_or_else(|| "Incomplete HTTP headers".to_string())?;
    let header_text = String::from_utf8_lossy(&data[..header_end]);
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| "Missing request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_uppercase();
    let path = request_parts
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_lowercase(), value.trim().to_string());
        }
    }
    let body_start = header_end + 4;
    let body_end = body_start.saturating_add(content_length).min(data.len());

    Ok(HttpRequest {
        method,
        path,
        headers,
        body: data[body_start..body_end].to_vec(),
    })
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

async fn write_json<T: Serialize>(
    stream: &mut tokio::net::TcpStream,
    status: &str,
    value: &T,
) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    let headers = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-Lap-Token\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        status,
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .await
        .map_err(|error| error.to_string())?;
    if status != "204 No Content" {
        stream
            .write_all(&body)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn capture_remote_asset(request: CaptureRequest) -> Result<CaptureResult, String> {
    let source_url = request.source_url.trim();
    if !(source_url.starts_with("https://") || source_url.starts_with("http://")) {
        return Err("Only HTTP and HTTPS URLs are supported".to_string());
    }

    if !request.allow_duplicate {
        if let Some(file_id) = t_dam::find_file_by_source_url(source_url)? {
            return Ok(CaptureResult {
                ok: true,
                duplicate: true,
                file_id: Some(file_id),
                file_path: None,
                file_name: None,
                folder: None,
                applied_tag_ids: Vec::new(),
                message: "This source URL is already in the library".to_string(),
            });
        }
    }

    let folder = t_dam::find_folder(request.folder_path.as_deref())?
        .ok_or_else(|| "No Lap album folder is available. Add an album or an Inbox folder first.".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|error| error.to_string())?;
    let mut builder = client
        .get(source_url)
        .header(USER_AGENT, "Lap Browser Capture/1.0");
    if let Some(page_url) = request.page_url.as_deref() {
        if page_url.starts_with("https://") || page_url.starts_with("http://") {
            builder = builder.header(REFERER, page_url);
        }
    }
    let response = builder.send().await.map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("The source returned HTTP {}", response.status()));
    }
    if response.content_length().is_some_and(|size| size > MAX_DOWNLOAD_BYTES) {
        return Err("The asset is larger than the 512 MB capture limit".to_string());
    }

    let mime = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or(value).trim().to_lowercase())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let disposition_name = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .and_then(filename_from_content_disposition);
    let resolved_url = response.url().as_str().to_string();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err("The asset is larger than the 512 MB capture limit".to_string());
    }

    let filename = disposition_name
        .or_else(|| filename_from_url(&resolved_url))
        .or_else(|| filename_from_url(source_url))
        .unwrap_or_else(|| format!("capture-{}", Utc::now().timestamp_millis()));
    let filename = ensure_extension(sanitize_filename(&filename), &mime);
    let target_path = unique_target_path(Path::new(&folder.path), &filename);
    std::fs::write(&target_path, &bytes).map_err(|error| error.to_string())?;

    let target_path_string = target_path.to_string_lossy().into_owned();
    let file_type = match t_utils::get_file_type(&target_path_string) {
        Some(file_type) => file_type,
        None => {
            let _ = std::fs::remove_file(&target_path);
            return Err(format!("Unsupported captured file format: {}", filename));
        }
    };
    let now = Utc::now().timestamp_millis();
    let (file, _) = match AFile::add_to_db(folder.id, &target_path_string, file_type, now) {
        Ok(result) => result,
        Err(error) => {
            let _ = std::fs::remove_file(&target_path);
            return Err(error);
        }
    };
    let file_id = file.id.ok_or_else(|| "Imported file has no database ID".to_string())?;
    let source = CaptureSourceMetadata {
        source_url: source_url.to_string(),
        page_url: request.page_url,
        page_title: request.page_title,
        author: request.author,
        site_name: request.site_name,
        alt_text: request.alt_text,
        metadata: request.metadata,
    };
    t_dam::upsert_source_metadata(file_id, &source)?;
    t_dam::set_workflow_status(file_id, "inbox")?;
    let applied_tag_ids = t_dam::apply_tags(file_id, &request.tags)?;

    Ok(CaptureResult {
        ok: true,
        duplicate: false,
        file_id: Some(file_id),
        file_path: Some(target_path_string),
        file_name: Some(file.name),
        folder: Some(folder),
        applied_tag_ids,
        message: "Captured successfully".to_string(),
    })
}

fn filename_from_content_disposition(value: &str) -> Option<String> {
    for part in value.split(';') {
        let (key, raw) = part.trim().split_once('=')?;
        if key.trim().eq_ignore_ascii_case("filename") {
            return Some(raw.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn filename_from_url(value: &str) -> Option<String> {
    let clean = value.split('?').next()?.split('#').next()?;
    let segment = clean.trim_end_matches('/').rsplit('/').next()?.trim();
    (!segment.is_empty()).then(|| segment.to_string())
}

fn sanitize_filename(value: &str) -> String {
    let mut result = value
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect::<String>();
    result = result.trim().trim_matches('.').to_string();
    if result.is_empty() {
        result = format!("capture-{}", Utc::now().timestamp_millis());
    }
    if result.chars().count() > 180 {
        result = result.chars().take(180).collect();
    }
    result
}

fn ensure_extension(filename: String, mime: &str) -> String {
    if Path::new(&filename).extension().is_some() {
        return filename;
    }
    let extension = match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/heic" | "image/heif" => "heic",
        "image/svg+xml" => "svg",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        _ => "bin",
    };
    format!("{}.{}", filename, extension)
}

fn unique_target_path(folder: &Path, filename: &str) -> PathBuf {
    let initial = folder.join(filename);
    if !initial.exists() {
        return initial;
    }
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("capture");
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 1..10_000 {
        let candidate = match extension {
            Some(extension) => folder.join(format!("{} ({}) .{}", stem, index, extension).replace(") .", ").")),
            None => folder.join(format!("{} ({})", stem, index)),
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    folder.join(format!("{}-{}", stem, Uuid::new_v4().simple()))
}
