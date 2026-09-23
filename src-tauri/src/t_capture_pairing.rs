//! Short-lived pairing requests. Only the main desktop window can approve them.
use serde::Serialize;
use serde_json::{Value, json};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use uuid::Uuid;

const LIFETIME: Duration = Duration::from_secs(120);
static PENDING: Mutex<Option<Pairing>> = Mutex::new(None);

struct Pairing {
    id: String,
    secret: String,
    origin: String,
    code: String,
    created: Instant,
    decision: Option<bool>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingPrompt {
    request_id: String,
    code: String,
    extension_id: String,
    seconds_left: u64,
}

fn extension_id(origin: &str) -> Result<&str, String> {
    let id = origin
        .strip_prefix("chrome-extension://")
        .unwrap_or_default();
    if id.len() != 32 || !id.bytes().all(|c| (b'a'..=b'p').contains(&c)) {
        return Err("Pairing must be started from a Chromium extension settings page".into());
    }
    Ok(id)
}

impl Pairing {
    fn prompt(&self) -> PairingPrompt {
        PairingPrompt {
            request_id: self.id.clone(),
            code: self.code.clone(),
            extension_id: self.origin.trim_start_matches("chrome-extension://").into(),
            seconds_left: LIFETIME.saturating_sub(self.created.elapsed()).as_secs(),
        }
    }
}

pub fn begin(app: &tauri::AppHandle, origin: &str) -> Result<Value, String> {
    extension_id(origin)?;
    let mut pending = PENDING.lock().map_err(|_| "Pairing unavailable")?;
    if pending
        .as_ref()
        .is_some_and(|p| p.created.elapsed() < LIFETIME)
    {
        return Err("已有配对请求，请先完成该请求，或两分钟后重试。".into());
    }
    let pairing = Pairing {
        id: Uuid::new_v4().to_string(),
        secret: Uuid::new_v4().to_string(),
        origin: origin.into(),
        code: format!("{:06}", rand::random::<u32>() % 1_000_000),
        created: Instant::now(),
        decision: None,
    };
    let result = json!({"ok": true, "requestId": pairing.id, "secret": pairing.secret,
        "code": pairing.code, "expiresIn": LIFETIME.as_secs()});
    let prompt = pairing.prompt();
    *pending = Some(pairing);
    drop(pending);
    // The event contains the comparison code, never the credential or secret.
    let _ = app.emit_to("main", "lap-capture-pairing", prompt);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
    }
    Ok(result)
}

pub fn status(origin: &str, body: &Value, token: &str) -> Result<Value, String> {
    extension_id(origin)?;
    let pending = PENDING.lock().map_err(|_| "Pairing unavailable")?;
    let p = pending.as_ref().ok_or("配对请求已失效，请重新连接。")?;
    if p.created.elapsed() >= LIFETIME {
        return Err("配对请求已过期，请重新连接。".into());
    }
    if origin != p.origin
        || body["requestId"].as_str() != Some(&p.id)
        || body["secret"].as_str() != Some(&p.secret)
    {
        return Err("Invalid pairing request".into());
    }
    match p.decision {
        Some(true) => Ok(json!({"ok": true, "status": "approved", "token": token})),
        Some(false) => Ok(json!({"ok": true, "status": "rejected"})),
        None => Ok(json!({"ok": true, "status": "pending"})),
    }
}

#[tauri::command]
pub fn get_capture_pairing(window: tauri::WebviewWindow) -> Result<Option<PairingPrompt>, String> {
    if window.label() != "main" {
        return Err("Main window required".into());
    }
    let pending = PENDING.lock().map_err(|_| "Pairing unavailable")?;
    Ok(pending
        .as_ref()
        .filter(|p| p.decision.is_none() && p.created.elapsed() < LIFETIME)
        .map(Pairing::prompt))
}

#[tauri::command]
pub fn decide_capture_pairing(
    window: tauri::WebviewWindow,
    request_id: String,
    approve: bool,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Main window required".into());
    }
    let mut pending = PENDING.lock().map_err(|_| "Pairing unavailable")?;
    let p = pending.as_mut().ok_or("Pairing request expired")?;
    if p.id != request_id || p.created.elapsed() >= LIFETIME || p.decision.is_some() {
        return Err("Pairing request expired".into());
    }
    p.decision = Some(approve);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credential_requires_matching_request_secret_origin_and_desktop_approval() {
        let origin = "chrome-extension://abcdefghijklmnopabcdefghijklmnop";
        let body = json!({"requestId": "request", "secret": "nonce"});
        *PENDING.lock().unwrap() = Some(Pairing {
            id: "request".into(),
            secret: "nonce".into(),
            origin: origin.into(),
            code: "123456".into(),
            created: Instant::now(),
            decision: None,
        });
        assert!(
            status(origin, &body, "credential")
                .unwrap()
                .get("token")
                .is_none()
        );
        assert!(
            status(
                origin,
                &json!({"requestId":"request","secret":"wrong"}),
                "credential"
            )
            .is_err()
        );
        assert!(
            status(
                "chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                &body,
                "credential"
            )
            .is_err()
        );
        PENDING.lock().unwrap().as_mut().unwrap().decision = Some(false);
        assert_eq!(
            status(origin, &body, "credential").unwrap()["status"],
            "rejected"
        );
        PENDING.lock().unwrap().as_mut().unwrap().decision = Some(true);
        assert_eq!(
            status(origin, &body, "credential").unwrap()["token"],
            "credential"
        );
        PENDING.lock().unwrap().as_mut().unwrap().created = Instant::now() - LIFETIME;
        assert!(status(origin, &body, "credential").is_err());
        *PENDING.lock().unwrap() = None;
    }

    #[test]
    fn pairing_rejects_web_origins_and_malformed_extension_ids() {
        assert!(extension_id("chrome-extension://abcdefghijklmnopabcdefghijklmnop").is_ok());
        for origin in [
            "https://example.com",
            "null",
            "",
            "chrome-extension://abc",
            "chrome-extension://abcdefghijklmnopabcdefghijklmnop/",
            "chrome-extension://zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        ] {
            assert!(extension_id(origin).is_err());
        }
    }
}
