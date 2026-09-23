import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createProviderForm, providerInput } from '../../src-vite/src/common/online-ai-form.mjs';
import { BOARD_STORAGE_KEY, readBoardState, writeBoardState } from '../../src-vite/src/common/reference-board-state.mjs';
import { createReferenceBoardDragSession } from '../../src-vite/src/common/reference-board-drag.mjs';
const prompt = readFileSync(new URL('../../src-vite/src/common/online-ai-system-prompt.txt', import.meta.url), 'utf8');
const presets = JSON.parse(readFileSync(new URL('../../src-vite/src/common/online-ai-presets.json', import.meta.url), 'utf8'));

test('preset switch creates a separate draft without the old credential or identity', () => {
  const form = { ...createProviderForm(prompt, presets[0]), id: 'saved', apiKey: 'old-key', hasApiKey: true, extraHeaders: { 'X-Key': 'old' } };
  Object.assign(form, createProviderForm(prompt, presets.find(p => p.id === 'gemini')));
  assert.equal(form.kind, 'gemini'); assert.equal(form.id, null);
  assert.equal(form.apiKey, ''); assert.equal(form.hasApiKey, false);
  assert.deepEqual(form.extraHeaders, {});
  assert.ok(form.systemPrompt.includes('targetFolderId'));
});
test('test/save payload uses current edits and resolves an empty prompt', () => {
  const form = createProviderForm(prompt, presets[0]);
  form.model = ' edited-model '; form.apiKey = ' new-key '; form.systemPrompt = '   ';
  const input = providerInput(form, prompt);
  assert.equal(input.model, 'edited-model'); assert.equal(input.apiKey, 'new-key');
  assert.equal(input.systemPrompt, prompt.trim()); assert.equal(input.id, null);
  form.systemPrompt = '自定义当前提示词';
  assert.equal(providerInput(form, prompt).systemPrompt, form.systemPrompt);
});
test('all provider presets include the intended protocol and HTTPS endpoint', () => {
  assert.equal(new Set(presets.map(p => p.id)).size, 9);
  for (const p of presets) {
    assert.ok(['openai_compatible', 'gemini', 'anthropic'].includes(p.kind));
    assert.equal(new URL(p.baseUrl).protocol, 'https:');
    assert.ok(p.model && p.docsUrl);
  }
  assert.equal(presets.find(p => p.id === 'deepseek').model, 'deepseek-flash');
});
test('board round trip preserves positions, dimensions, camera and pin state', () => {
  const values = new Map();
  const storage = { setItem: (k, v) => values.set(k, v) };
  const items = [{ id: 'one', path: 'C:/board.png', x: -140.5, y: 89, width: 420, height: 300, sized: true, failed: false }];
  const camera = { x: 257, y: -78, scale: 1.35 };
  writeBoardState(storage, items, camera, false);
  const state = readBoardState(values.get(BOARD_STORAGE_KEY));
  assert.deepEqual(state.items, items); assert.deepEqual(state.camera, camera); assert.equal(state.alwaysOnTop, false);
  items[0].x = 999;
  writeBoardState(storage, items, camera, true);
  assert.equal(readBoardState(values.get(BOARD_STORAGE_KEY)).items[0].x, 999);
});
test('board restores valid old entries, retries image loading and rejects invalid geometry', () => {
  const state = readBoardState(JSON.stringify({ items: [
    { id: 'valid', path: 'C:/image.png', x: 1, y: 2, width: 100, height: 200, failed: true },
    { id: 'bad', path: 'C:/bad.png', x: null, y: 0, width: 100, height: 200 },
  ], camera: { x: null, y: 6, scale: 999 } }));
  assert.equal(state.items.length, 1); assert.equal(state.items[0].failed, false);
  assert.deepEqual(state.camera, { x: 0, y: 6, scale: 32 });
});
test('storage failure propagates so the board can remain open instead of losing layout', () => {
  assert.throws(() => writeBoardState({ setItem() { throw new Error('quota'); } }, [], {}, true), /quota/);
});
test('closing a hidden reference board requires a new choice before it can reopen', async () => {
  const session = createReferenceBoardDragSession();
  let visible = false;
  const savedWindow = { isVisible: async () => visible };
  assert.equal(await session.nextDrag(null), 'prompt');
  visible = true;
  assert.equal(await session.nextDrag(savedWindow), 'native');
  visible = false;
  session.boardClosed();
  assert.equal(await session.nextDrag(savedWindow), 'prompt');
  session.chooseExternal();
  assert.equal(await session.nextDrag(savedWindow), 'native');
  assert.equal(await session.nextDrag(savedWindow), 'prompt');
  session.boardClosed();
  assert.equal(await session.nextDrag(savedWindow), 'prompt');
});
