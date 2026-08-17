const DEFAULT_URL = 'http://127.0.0.1:47821';

const elements = {
  apiUrl: document.querySelector('#apiUrl'),
  token: document.querySelector('#token'),
  folderPath: document.querySelector('#folderPath'),
  tags: document.querySelector('#tags'),
  allowDuplicate: document.querySelector('#allowDuplicate'),
  advanced: document.querySelector('#advancedSettings'),
  detectButton: document.querySelector('#detectButton'),
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

function setConnectionState(state, message) {
  const online = state === 'online';
  const detected = state === 'detected';
  const checking = state === 'checking';
  elements.badge.textContent = online
    ? '已连接'
    : detected
      ? '已找到 Lap'
      : checking
        ? '检测中'
        : '未连接';
  elements.badge.classList.toggle('online', online);
  elements.badge.classList.toggle('detected', detected);
  elements.badge.classList.toggle('checking', checking);
  elements.badge.classList.toggle('offline', !online && !detected && !checking);
  elements.message.textContent = message || '';
  elements.message.classList.toggle('success', online || detected);
  elements.message.classList.toggle('error', state === 'offline' && Boolean(message));
}

async function getStoredConfig() {
  return chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: '',
    folderPath: '',
    tags: [],
    allowDuplicate: false,
    pairingOnboarding: false,
  });
}

async function request(path, config) {
  let response;
  try {
    response = await fetch(`${normalizeApiUrl(config.apiUrl)}${path}`, {
      headers: config.token ? { 'X-Lap-Token': config.token } : {},
    });
  } catch {
    const error = new Error('未找到本机 Lap。请先启动 Lap 桌面应用，再重新检测。');
    error.status = 0;
    throw error;
  }
  const data = await response.json().catch(() => ({}));
  if (!response.ok || data.ok === false) {
    const error = new Error(data.error || `请求失败：HTTP ${response.status}`);
    error.status = response.status;
    error.data = data;
    throw error;
  }
  return data;
}

function isInvalidToken(error) {
  return error?.status === 401
    || String(error?.message || '').toLowerCase().includes('invalid pairing token');
}

async function loadFolders(config) {
  const data = await request('/folders', config);
  const selected = elements.folderPath.value || config.folderPath || '';
  elements.folderPath.innerHTML = '<option value="">自动选择第一个可用目录</option>';
  for (const folder of data.folders || []) {
    const option = document.createElement('option');
    option.value = folder.path;
    option.textContent = `${folder.name} — ${folder.path}`;
    elements.folderPath.append(option);
  }
  elements.folderPath.value = selected;
}

function setBusy(busy) {
  elements.detectButton.disabled = busy;
  elements.testButton.disabled = busy;
}

async function verifyConnection(config) {
  await request('/health', { ...config, token: '' });
  if (!config.token) {
    elements.advanced.open = true;
    setConnectionState('detected', '已找到本机 Lap。请在高级连接设置中粘贴配对 Token，然后验证配对。');
    return false;
  }
  try {
    await loadFolders(config);
  } catch (error) {
    if (isInvalidToken(error)) {
      elements.advanced.open = true;
      setConnectionState('detected', '已找到本机 Lap，但配对 Token 不正确。请从 Lap 设置中重新复制。');
      return false;
    }
    throw error;
  }
  setConnectionState('online', '连接成功，已读取 Lap 素材目录。');
  return true;
}

async function testConnection() {
  const config = {
    apiUrl: normalizeApiUrl(elements.apiUrl.value),
    token: elements.token.value.trim(),
  };
  setBusy(true);
  setConnectionState('checking', '正在检测本机 Lap…');
  try {
    await verifyConnection(config);
  } catch (error) {
    setConnectionState('offline', error instanceof Error ? error.message : String(error));
  } finally {
    setBusy(false);
  }
}

async function detectLocalLap() {
  elements.apiUrl.value = DEFAULT_URL;
  setBusy(true);
  setConnectionState('checking', '正在检测本机 Lap…');
  try {
    await verifyConnection({
      apiUrl: DEFAULT_URL,
      token: elements.token.value.trim(),
    });
  } catch (error) {
    setConnectionState('offline', error instanceof Error ? error.message : String(error));
  } finally {
    setBusy(false);
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

  if (config.pairingOnboarding) {
    elements.advanced.open = true;
    requestAnimationFrame(() => elements.token.focus());
    await chrome.storage.local.set({ pairingOnboarding: false });
  }

  await testConnection();
  elements.folderPath.value = config.folderPath || '';
}

elements.detectButton.addEventListener('click', detectLocalLap);
elements.testButton.addEventListener('click', testConnection);
elements.saveButton.addEventListener('click', saveConfig);
initialize();
