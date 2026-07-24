const DEFAULT_URL = 'http://127.0.0.1:47821';

const elements = {
  apiUrl: document.querySelector('#apiUrl'),
  token: document.querySelector('#token'),
  folderPath: document.querySelector('#folderPath'),
  tags: document.querySelector('#tags'),
  allowDuplicate: document.querySelector('#allowDuplicate'),
  testButton: document.querySelector('#testButton'),
  saveButton: document.querySelector('#saveButton'),
  badge: document.querySelector('#connectionBadge'),
  message: document.querySelector('#connectionMessage'),
};

function normalizeApiUrl(value) {
  return (value || DEFAULT_URL).trim().replace(/\/$/, '');
}

function parseTags(value) {
  return value
    .split(/[\n,]/)
    .map((item) => item.trim())
    .filter(Boolean)
    .filter((item, index, array) => array.indexOf(item) === index);
}

function setConnectionState(ok, message) {
  elements.badge.textContent = ok ? '已连接' : '未连接';
  elements.badge.classList.toggle('online', ok);
  elements.badge.classList.toggle('offline', !ok);
  elements.message.textContent = message || '';
  elements.message.classList.toggle('success', ok);
  elements.message.classList.toggle('error', !ok && Boolean(message));
}

async function getStoredConfig() {
  return chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: '',
    folderPath: '',
    tags: [],
    allowDuplicate: false,
  });
}

async function request(path, config) {
  const response = await fetch(`${normalizeApiUrl(config.apiUrl)}${path}`, {
    headers: config.token ? { 'X-Lap-Token': config.token } : {},
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok || data.ok === false) {
    throw new Error(data.error || `请求失败：HTTP ${response.status}`);
  }
  return data;
}

async function loadFolders(config) {
  const data = await request('/folders', config);
  const selected = elements.folderPath.value || config.folderPath || '';
  elements.folderPath.innerHTML = '<option value="">自动选择 Inbox 或第一个可用目录</option>';
  for (const folder of data.folders || []) {
    const option = document.createElement('option');
    option.value = folder.path;
    option.textContent = `${folder.name} — ${folder.path}`;
    elements.folderPath.append(option);
  }
  elements.folderPath.value = selected;
}

async function testConnection() {
  const config = {
    apiUrl: normalizeApiUrl(elements.apiUrl.value),
    token: elements.token.value.trim(),
  };
  elements.testButton.disabled = true;
  setConnectionState(false, '正在连接 Lap…');
  try {
    await request('/health', { ...config, token: '' });
    await loadFolders(config);
    setConnectionState(true, '连接成功，已读取 Lap 素材目录。');
  } catch (error) {
    setConnectionState(false, error instanceof Error ? error.message : String(error));
  } finally {
    elements.testButton.disabled = false;
  }
}

async function saveConfig() {
  const config = {
    apiUrl: normalizeApiUrl(elements.apiUrl.value),
    token: elements.token.value.trim(),
    folderPath: elements.folderPath.value,
    tags: parseTags(elements.tags.value),
    allowDuplicate: elements.allowDuplicate.checked,
  };
  await chrome.storage.local.set(config);
  elements.saveButton.textContent = '已保存';
  setTimeout(() => {
    elements.saveButton.textContent = '保存设置';
  }, 1200);
}

async function initialize() {
  const config = await getStoredConfig();
  elements.apiUrl.value = config.apiUrl;
  elements.token.value = config.token;
  elements.tags.value = (config.tags || []).join('\n');
  elements.allowDuplicate.checked = Boolean(config.allowDuplicate);
  elements.folderPath.value = config.folderPath || '';

  if (config.token) {
    await testConnection();
    elements.folderPath.value = config.folderPath || '';
  }
}

elements.testButton.addEventListener('click', testConnection);
elements.saveButton.addEventListener('click', saveConfig);
initialize();
