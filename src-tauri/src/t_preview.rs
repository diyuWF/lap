use crate::t_sqlite::AFile;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewKind {
    Image,
    Video,
    Svg,
    Pdf,
    Model3d,
    DesignAsset,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewDescriptor {
    pub file_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub kind: PreviewKind,
    pub mime_type: Option<String>,
    pub capabilities: Vec<String>,
    pub can_use_original: bool,
    pub can_open_external: bool,
    pub message: Option<String>,
}

fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn mime_for_extension(extension: &str) -> Option<String> {
    let mime = match extension {
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        "obj" => "model/obj",
        "stl" => "model/stl",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        _ => return None,
    };
    Some(mime.to_string())
}

fn descriptor_kind(file_type: i64, extension: &str) -> PreviewKind {
    match extension {
        "svg" => PreviewKind::Svg,
        "pdf" => PreviewKind::Pdf,
        "glb" | "gltf" | "obj" | "stl" => PreviewKind::Model3d,
        "psd" | "psb" | "ai" | "eps" | "blend" | "fbx" | "usd" | "usda"
        | "usdc" | "usdz" | "abc" | "3ds" | "dae" | "c4d" | "dwg" | "dxf"
        | "aep" | "aepx" | "prproj" => PreviewKind::DesignAsset,
        _ => match file_type {
            1 | 3 => PreviewKind::Image,
            2 => PreviewKind::Video,
            _ => PreviewKind::Unsupported,
        },
    }
}

#[tauri::command]
pub fn get_preview_descriptor(file_id: i64) -> Result<PreviewDescriptor, String> {
    let file = AFile::get_file_info(file_id)?
        .ok_or_else(|| format!("未找到文件：{}", file_id))?;
    let file_path = file
        .file_path
        .clone()
        .ok_or_else(|| "文件路径不可用".to_string())?;
    if !Path::new(&file_path).exists() {
        return Err(format!("文件不存在：{}", file_path));
    }

    let extension = extension(&file_path);
    let kind = descriptor_kind(file.file_type.unwrap_or_default(), &extension);
    let (capabilities, can_use_original, message) = match kind {
        PreviewKind::Image => (
            vec!["zoom", "pan", "rotate"],
            true,
            None,
        ),
        PreviewKind::Video => (
            vec!["playback", "seek", "volume", "fullscreen"],
            true,
            None,
        ),
        PreviewKind::Svg => (
            vec!["zoom", "pan", "transparent_background"],
            true,
            None,
        ),
        PreviewKind::Pdf => (
            vec!["pages", "zoom", "print"],
            true,
            None,
        ),
        PreviewKind::Model3d => (
            vec!["orbit", "zoom", "wireframe", "reset_camera"],
            true,
            None,
        ),
        PreviewKind::DesignAsset => (
            vec!["thumbnail", "open_external", "metadata"],
            false,
            Some("该格式暂不支持完整交互预览，可查看缩略图或使用原始应用打开。".to_string()),
        ),
        PreviewKind::Unsupported => (
            vec!["open_external", "metadata"],
            false,
            Some("该文件暂不支持内置预览。".to_string()),
        ),
    };

    Ok(PreviewDescriptor {
        file_id,
        file_path,
        file_name: file.name,
        extension: extension.clone(),
        kind,
        mime_type: mime_for_extension(&extension),
        capabilities: capabilities.into_iter().map(str::to_string).collect(),
        can_use_original,
        can_open_external: true,
        message,
    })
}
