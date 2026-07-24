import { invoke } from '@tauri-apps/api/core';

export async function initDamSchema() {
  return invoke('dam_init_schema');
}

export async function listDamFolders() {
  return invoke('dam_list_folders');
}

export async function findDuplicateSource(sourceUrl) {
  return invoke('dam_find_duplicate_source', { sourceUrl });
}

export async function saveSourceMetadata(fileId, source) {
  return invoke('dam_save_source_metadata', { fileId, source });
}

export async function setDamWorkflowStatus(fileId, status) {
  return invoke('dam_set_workflow_status', { fileId, status });
}

export async function applyDamTags(fileId, tags) {
  return invoke('dam_apply_tags', { fileId, tags });
}

export async function getCaptureServerInfo() {
  return invoke('get_capture_server_info');
}

export async function listOnlineAiProviders() {
  return invoke('list_online_ai_providers');
}

export async function saveOnlineAiProvider(input) {
  return invoke('save_online_ai_provider', { input });
}

export async function deleteOnlineAiProvider(providerId) {
  return invoke('delete_online_ai_provider', { providerId });
}

export async function testOnlineAiProvider(providerId) {
  return invoke('test_online_ai_provider', { providerId });
}

export async function analyzeFileWithOnlineAi(fileId, providerId, forceAutoApply = null) {
  return invoke('analyze_file_with_online_ai', {
    fileId,
    providerId,
    forceAutoApply,
  });
}
