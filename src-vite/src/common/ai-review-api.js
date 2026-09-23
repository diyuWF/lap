import { invoke } from '@tauri-apps/api/core';

export async function listAiSuggestions(status = 'pending', limit = 200) {
  return invoke('list_ai_suggestions', { status, limit });
}

export async function reviewAiSuggestion(id, action) {
  return invoke('review_ai_suggestion', { suggestionId: id, action });
}

export async function clearReviewedAiSuggestions() {
  return invoke('clear_reviewed_ai_suggestions');
}
