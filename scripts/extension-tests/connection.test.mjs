import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { Window } from 'happy-dom';

function harness({ token = '', health = { ok: true, pairingSupported: true }, decision = 'approved' } = {}) {
  const window = new Window({ url: 'chrome-extension://abcdefghijklmnopabcdefghijklmnop/options.html' });
  window.Option = function(text, value) { const node = window.document.createElement('option'); node.textContent = text; node.value = value; return node; };
  const stored = { token };
  const calls = [];
  const read = name => fs.readFileSync(new URL('../../lap-extension/' + name, import.meta.url), 'utf8');
  window.document.write(read('options.html'));
  window.chrome = { storage: { local: {
    get: async defaults => ({ ...defaults, ...stored }),
    set: async value => Object.assign(stored, value),
  } } };
  window.fetch = async (url, options) => {
    calls.push({ url, options });
    const data = url.endsWith('/health') ? health
      : url.endsWith('/pair/request') ? { ok: true, requestId: 'request', secret: 'nonce', code: '123456', expiresIn: 120 }
      : url.endsWith('/pair/status') ? { ok: true, status: decision, token: 'a'.repeat(64) }
      : { ok: true, folders: [{ name: 'Assets', path: 'D:/assets' }] };
    return { ok: true, status: 200, json: async () => data };
  };
  window.eval(read('api.js').replaceAll('export ', '') + '\n' + read('options.js').replace(/^import[\s\S]*?from ['"]\.\/api\.js['"];?\s*/, ''));
  return { window, stored, calls, element: id => window.document.getElementById(id) };
}
const settle = (ms = 30) => new Promise(resolve => setTimeout(resolve, ms));

test('opening unpaired settings never requests authorization automatically', async () => {
  const h = harness(); await settle();
  assert.equal(h.element('advancedSettings').open, false);
  assert.equal(h.calls.filter(c => c.url.includes('/pair/')).length, 0);
  assert.equal(h.element('connectionBadge').textContent, '需要配对');
  h.window.close();
});

test('desktop approval persists and reopening reuses the credential', async () => {
  const h = harness(); await settle(); h.element('detectButton').click(); await settle(1150);
  assert.equal(h.stored.token, 'a'.repeat(64));
  assert.equal(h.element('connectionBadge').textContent, '已连接', h.element('connectionMessage').textContent);
  assert.equal(h.element('pairingPanel').hidden, true);
  const reopened = harness({ token: h.stored.token }); await settle();
  assert.equal(reopened.element('connectionBadge').textContent, '已连接');
  assert.equal(reopened.calls.filter(c => c.url.includes('/pair/')).length, 0);
  h.window.close(); reopened.window.close();
});

test('rejected pairing never stores a credential', async () => {
  const h = harness({ decision: 'rejected' }); await settle();
  h.element('detectButton').click(); await settle(1150);
  assert.equal(h.stored.token, '');
  assert.match(h.element('connectionMessage').textContent, /拒绝/);
  h.window.close();
});

test('legacy manual verification persists without an extra save', async () => {
  const h = harness({ health: { ok: true } }); await settle();
  assert.equal(h.element('advancedSettings').open, true);
  h.element('token').value = 'b'.repeat(64); h.element('testButton').click(); await settle();
  assert.equal(h.stored.token, 'b'.repeat(64));
  assert.equal(h.element('connectionBadge').textContent, '已连接', h.element('connectionMessage').textContent);
  h.window.close();
});
