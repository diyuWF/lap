const DEFAULT_URL = 'http://127.0.0.1:47821';

function normalizeApiUrl(value) {
  return (value || DEFAULT_URL).trim().replace(/\/+$/, '');
}

async function getConfig() {
  return chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: '',
    tags: [],
    allowDuplicate: false,
    recentFolders: [],
  });
}

function friendlyApiError(data, status) {
  const message = String(data?.error || '');
  if (status === 401 || message.toLowerCase().includes('invalid pairing token')) {
    return '已找到本机 Lap，但配对 Token 不正确。请打开扩展设置重新检测并复制 Token。';
  }
  return message || `Lap 请求失败：HTTP ${status}`;
}

async function apiRequest(path, options = {}) {
  const config = await getConfig();
  if (!config.token && path !== '/health') {
    throw new Error('请先在扩展设置中填写 Lap 配对 Token');
  }

  const headers = new Headers(options.headers || {});
  if (config.token) headers.set('X-Lap-Token', config.token);
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  let response;
  try {
    response = await fetch(`${normalizeApiUrl(config.apiUrl)}${path}`, {
      ...options,
      headers,
    });
  } catch {
    throw new Error('无法连接本机 Lap。请确认 Lap 已启动并且采集服务可用。');
  }

  const data = await response.json().catch(() => ({}));
  if (!response.ok || data.ok === false) {
    throw new Error(friendlyApiError(data, response.status));
  }
  return data;
}

async function rememberFolder(folderPath) {
  if (!folderPath) return;
  const { recentFolders = [] } = await getConfig();
  const next = [folderPath, ...recentFolders.filter((item) => item !== folderPath)].slice(0, 8);
  await chrome.storage.local.set({ recentFolders: next });
}

async function capture(payload) {
  const config = await getConfig();
  const data = await apiRequest('/capture', {
    method: 'POST',
    body: JSON.stringify({
      ...payload,
      tags: Array.isArray(payload.tags) && payload.tags.length
        ? payload.tags
        : (Array.isArray(config.tags) ? config.tags : []),
      allowDuplicate: payload.allowDuplicate ?? Boolean(config.allowDuplicate),
    }),
  });
  if (!data.duplicate) await rememberFolder(payload.folderPath);
  return data;
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  const run = async () => {
    switch (message?.type) {
      case 'lap:get-state': {
        const config = await getConfig();
        const foldersResponse = await apiRequest('/folders');
        return {
          ok: true,
          folders: foldersResponse.folders || [],
          aiConfigured: Boolean(foldersResponse.aiConfigured),
          recentFolders: config.recentFolders || [],
          configured: Boolean(config.token),
        };
      }
      case 'lap:create-folder': {
        const result = await apiRequest('/folders', {
          method: 'POST',
          body: JSON.stringify({
            parentPath: message.parentPath || '',
            name: message.name || '',
          }),
        });
        return { ok: true, folder: result.folder };
      }
      case 'lap:capture':
        return { ok: true, result: await capture(message.payload || {}) };
      case 'lap:open-options':
        await chrome.runtime.openOptionsPage();
        return { ok: true };
      default:
        return { ok: false, error: '未知的扩展消息' };
    }
  };

  run()
    .then(sendResponse)
    .catch((error) => sendResponse({
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    }));
  return true;
});
