const DEFAULT_URL = 'http://127.0.0.1:47821';
const MAX_IMAGES = 300;
const CONCURRENCY = 3;

const state = {
  config: null,
  tab: null,
  images: [],
  selected: new Set(),
  busy: false,
};

const elements = {
  connectionCard: document.querySelector('#connectionCard'),
  connectionTitle: document.querySelector('#connectionTitle'),
  connectionMessage: document.querySelector('#connectionMessage'),
  settingsButton: document.querySelector('#settingsButton'),
  folderSelect: document.querySelector('#folderSelect'),
  tagsInput: document.querySelector('#tagsInput'),
  allowDuplicate: document.querySelector('#allowDuplicate'),
  refreshButton: document.querySelector('#refreshButton'),
  selectAllButton: document.querySelector('#selectAllButton'),
  clearButton: document.querySelector('#clearButton'),
  selectionSummary: document.querySelector('#selectionSummary'),
  imageGrid: document.querySelector('#imageGrid'),
  emptyState: document.querySelector('#emptyState'),
  progressText: document.querySelector('#progressText'),
  captureButton: document.querySelector('#captureButton'),
};

function normalizeApiUrl(value) {
  return (value || DEFAULT_URL).trim().replace(/\/$/, '');
}

function normalizeTags(value) {
  return String(value || '')
    .split(/[,，\n]/)
    .map((tag) => tag.trim())
    .filter(Boolean)
    .filter((tag, index, list) => list.findIndex((item) => item.toLowerCase() === tag.toLowerCase()) === index)
    .slice(0, 50);
}

async function loadConfig() {
  state.config = await chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: '',
    folderPath: '',
    tags: [],
    allowDuplicate: false,
  });
  elements.tagsInput.value = Array.isArray(state.config.tags) ? state.config.tags.join(', ') : '';
  elements.allowDuplicate.checked = Boolean(state.config.allowDuplicate);
}

function friendlyApiError(data, status) {
  const message = String(data?.error || '');
  if (status === 401 || message.toLowerCase().includes('invalid pairing token')) {
    return '已找到本机 Lap，但配对 Token 不正确。请打开设置重新配对。';
  }
  return message || `请求失败：HTTP ${status}`;
}

async function api(path, options = {}) {
  const { omitToken = false, ...requestOptions } = options;
  const headers = {
    ...(requestOptions.body ? { 'Content-Type': 'application/json' } : {}),
    ...(omitToken ? {} : { 'X-Lap-Token': state.config.token }),
    ...(requestOptions.headers || {}),
  };
  let response;
  try {
    response = await fetch(`${normalizeApiUrl(state.config.apiUrl)}${path}`, {
      ...requestOptions,
      headers,
    });
  } catch {
    throw new Error('未找到本机 Lap。请先启动 Lap 桌面应用。');
  }
  const data = await response.json().catch(() => ({}));
  if (!response.ok || data.ok === false) {
    throw new Error(friendlyApiError(data, response.status));
  }
  return data;
}

function setConnection(type, title, message) {
  elements.connectionCard.classList.remove('connected', 'error');
  if (type) elements.connectionCard.classList.add(type);
  elements.connectionTitle.textContent = title;
  elements.connectionMessage.textContent = message;
}

async function connect() {
  try {
    await api('/health', { omitToken: true });
  } catch (error) {
    setConnection('error', '未找到本机 Lap', error.message || '请确认 Lap 已启动。');
    elements.captureButton.disabled = true;
    return false;
  }
  if (!state.config.token) {
    setConnection('error', '已找到 Lap，尚未配对', '请打开设置，自动检测后粘贴 Lap 配对 Token。');
    elements.captureButton.disabled = true;
    return false;
  }
  try {
    const result = await api('/folders');
    renderFolders(result.folders || []);
    setConnection('connected', '已连接 Lap', '本地采集服务运行正常。');
    return true;
  } catch (error) {
    setConnection('error', '无法连接 Lap', error.message || '请确认 Lap 已启动。');
    elements.captureButton.disabled = true;
    return false;
  }
}

function renderFolders(folders) {
  elements.folderSelect.replaceChildren();
  const fallback = document.createElement('option');
  fallback.value = '';
  fallback.textContent = '自动保存到待整理区域';
  elements.folderSelect.append(fallback);
  for (const folder of folders) {
    const option = document.createElement('option');
    option.value = folder.path;
    option.textContent = `${folder.name} — ${folder.path}`;
    elements.folderSelect.append(option);
  }
  elements.folderSelect.value = folders.some((folder) => folder.path === state.config.folderPath)
    ? state.config.folderPath
    : '';
}

async function getActiveTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  state.tab = tab || null;
  return state.tab;
}

function pageCollector() {
  const makeAbsolute = (value) => {
    try { return new URL(value, document.baseURI).href; } catch { return ''; }
  };
  const parseSrcset = (value) => String(value || '')
    .split(',')
    .map((item) => item.trim().split(/\s+/)[0])
    .map(makeAbsolute)
    .filter(Boolean);
  const candidates = [];
  for (const image of document.images) {
    const urls = [image.currentSrc, image.src, ...parseSrcset(image.srcset)].map(makeAbsolute).filter(Boolean);
    const sourceUrl = urls.at(-1) || '';
    if (!sourceUrl || sourceUrl.startsWith('data:') || sourceUrl.startsWith('blob:')) continue;
    const rect = image.getBoundingClientRect();
    const width = image.naturalWidth || Math.round(rect.width) || 0;
    const height = image.naturalHeight || Math.round(rect.height) || 0;
    if (width < 80 || height < 80) continue;
    candidates.push({
      sourceUrl,
      previewUrl: image.currentSrc || image.src || sourceUrl,
      altText: image.alt || image.getAttribute('aria-label') || '',
      width,
      height,
    });
  }
  const unique = new Map();
  for (const item of candidates) {
    const existing = unique.get(item.sourceUrl);
    if (!existing || item.width * item.height > existing.width * existing.height) unique.set(item.sourceUrl, item);
  }
  const meta = (selector, attribute = 'content') => document.querySelector(selector)?.getAttribute(attribute) || '';
  return {
    images: [...unique.values()].sort((a, b) => b.width * b.height - a.width * a.height).slice(0, 300),
    page: {
      url: location.href,
      title: document.title,
      author: meta('meta[name="author"]') || meta('meta[property="article:author"]'),
      siteName: meta('meta[property="og:site_name"]') || location.hostname,
    },
  };
}

async function scanPage() {
  elements.progressText.textContent = '正在扫描当前页面…';
  state.images = [];
  state.selected.clear();
  renderImages();
  const tab = await getActiveTab();
  if (!tab?.id || !/^https?:/i.test(tab.url || '')) {
    elements.progressText.textContent = '当前页面不允许读取。';
    elements.emptyState.hidden = false;
    return;
  }
  try {
    const [execution] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: pageCollector,
    });
    const result = execution?.result || { images: [], page: {} };
    state.images = (result.images || []).slice(0, MAX_IMAGES).map((item, index) => ({ ...item, id: index }));
    state.page = result.page || {};
    state.images.slice(0, 24).forEach((item) => state.selected.add(item.id));
    elements.progressText.textContent = state.images.length
      ? `已发现 ${state.images.length} 张图片，默认选择前 24 张。`
      : '';
  } catch (error) {
    elements.progressText.textContent = `扫描失败：${error.message || error}`;
  }
  renderImages();
}

function renderImages() {
  elements.imageGrid.replaceChildren();
  elements.emptyState.hidden = state.images.length > 0;
  for (const image of state.images) {
    const card = document.createElement('button');
    card.type = 'button';
    card.className = `asset-card${state.selected.has(image.id) ? ' selected' : ''}`;
    card.title = image.altText || image.sourceUrl;
    const preview = document.createElement('img');
    preview.src = image.previewUrl;
    preview.alt = '';
    preview.loading = 'lazy';
    preview.referrerPolicy = 'no-referrer';
    const check = document.createElement('span');
    check.className = 'check';
    check.textContent = '✓';
    const dimensions = document.createElement('span');
    dimensions.className = 'dimensions';
    dimensions.textContent = `${image.width}×${image.height}`;
    card.append(preview, check, dimensions);
    card.addEventListener('click', () => {
      if (state.selected.has(image.id)) state.selected.delete(image.id);
      else state.selected.add(image.id);
      renderImages();
    });
    elements.imageGrid.append(card);
  }
  updateSelectionSummary();
}

function updateSelectionSummary() {
  elements.selectionSummary.textContent = state.images.length
    ? `已选择 ${state.selected.size} / ${state.images.length}`
    : '没有可用图片';
  elements.captureButton.disabled = state.busy || state.selected.size === 0 || !state.config?.token;
  elements.captureButton.textContent = state.busy ? '正在保存…' : `保存所选素材（${state.selected.size}）`;
}

async function captureOne(image) {
  const page = state.page || {};
  return api('/capture', {
    method: 'POST',
    body: JSON.stringify({
      sourceUrl: image.sourceUrl,
      pageUrl: page.url || state.tab?.url || '',
      pageTitle: page.title || state.tab?.title || '',
      author: page.author || '',
      siteName: page.siteName || '',
      altText: image.altText || '',
      folderPath: elements.folderSelect.value || null,
      tags: normalizeTags(elements.tagsInput.value),
      allowDuplicate: elements.allowDuplicate.checked,
      metadata: {
        browser: 'chromium',
        capturedBy: 'lap-extension-popup',
        capturedAt: new Date().toISOString(),
        width: image.width,
        height: image.height,
      },
    }),
  });
}

async function runPool(items, worker, concurrency) {
  const results = new Array(items.length);
  let cursor = 0;
  async function runner() {
    while (cursor < items.length) {
      const index = cursor++;
      try { results[index] = { ok: true, value: await worker(items[index], index) }; }
      catch (error) { results[index] = { ok: false, error }; }
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, runner));
  return results;
}

async function captureSelected() {
  if (state.busy) return;
  const items = state.images.filter((image) => state.selected.has(image.id));
  if (!items.length) return;
  state.busy = true;
  updateSelectionSummary();
  await chrome.storage.local.set({
    folderPath: elements.folderSelect.value,
    tags: normalizeTags(elements.tagsInput.value),
    allowDuplicate: elements.allowDuplicate.checked,
  });
  let completed = 0;
  elements.progressText.textContent = `正在保存 0 / ${items.length}…`;
  const results = await runPool(items, async (item) => {
    const result = await captureOne(item);
    completed += 1;
    elements.progressText.textContent = `正在保存 ${completed} / ${items.length}…`;
    return result;
  }, CONCURRENCY);
  const success = results.filter((item) => item.ok && !item.value?.duplicate).length;
  const duplicate = results.filter((item) => item.ok && item.value?.duplicate).length;
  const failed = results.filter((item) => !item.ok).length;
  elements.progressText.textContent = `完成：新增 ${success}，重复 ${duplicate}，失败 ${failed}。`;
  state.busy = false;
  updateSelectionSummary();
}

elements.settingsButton.addEventListener('click', () => chrome.runtime.openOptionsPage());
elements.refreshButton.addEventListener('click', scanPage);
elements.selectAllButton.addEventListener('click', () => {
  state.images.forEach((image) => state.selected.add(image.id));
  renderImages();
});
elements.clearButton.addEventListener('click', () => {
  state.selected.clear();
  renderImages();
});
elements.captureButton.addEventListener('click', captureSelected);

await loadConfig();
const connected = await connect();
if (connected) await scanPage();
