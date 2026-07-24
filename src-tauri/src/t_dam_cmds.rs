use crate::t_capture_server::{self, CaptureServerInfo};
use crate::t_dam::{self, CaptureSourceMetadata, DamFolder};

#[tauri::command]
pub fn dam_init_schema() -> Result<(), String> {
    t_dam::ensure_schema()
}

#[tauri::command]
pub fn dam_list_folders() -> Result<Vec<DamFolder>, String> {
    t_dam::list_folders()
}

#[tauri::command]
pub fn dam_find_duplicate_source(source_url: String) -> Result<Option<i64>, String> {
    t_dam::find_file_by_source_url(source_url.trim())
}

#[tauri::command]
pub fn dam_save_source_metadata(
    file_id: i64,
    source: CaptureSourceMetadata,
) -> Result<(), String> {
    t_dam::upsert_source_metadata(file_id, &source)
}

#[tauri::command]
pub fn dam_set_workflow_status(file_id: i64, status: String) -> Result<(), String> {
    t_dam::set_workflow_status(file_id, &status)
}

#[tauri::command]
pub fn dam_apply_tags(file_id: i64, tags: Vec<String>) -> Result<Vec<i64>, String> {
    t_dam::apply_tags(file_id, &tags)
}

#[tauri::command]
pub fn get_capture_server_info() -> Result<CaptureServerInfo, String> {
    t_capture_server::get_capture_server_info()
        .ok_or_else(|| "Browser capture service is not running".to_string())
}
