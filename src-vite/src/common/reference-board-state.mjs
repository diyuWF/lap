export const BOARD_STORAGE_KEY = 'lap-reference-board-v1';

export function readBoardState(raw) {
  const state = JSON.parse(raw || '{}');
  const finite = value => typeof value === 'number' && Number.isFinite(value);
  const items = (Array.isArray(state.items) ? state.items : []).filter(item =>
    item && typeof item.id === 'string' && typeof item.path === 'string' && item.path
    && [item.x, item.y, item.width, item.height].every(finite)
    && item.width > 0 && item.height > 0,
  ).map(item => ({ ...item, failed: false }));
  const camera = state.camera || {};
  return { items, camera: {
    x: finite(camera.x) ? camera.x : 0,
    y: finite(camera.y) ? camera.y : 0,
    scale: finite(camera.scale) ? Math.max(0.05, Math.min(32, camera.scale)) : 1,
  }, alwaysOnTop: state.alwaysOnTop !== false };
}

export function writeBoardState(storage, items, camera, alwaysOnTop) {
  // Keep the same key for existing boards. Persist synchronously before hiding.
  storage.setItem(BOARD_STORAGE_KEY, JSON.stringify({ items, camera, alwaysOnTop }));
}
