const DEFAULT_URL = 'http://127.0.0.1:47821';

async function getConfig() {
  return chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: '',
    folderPath: '',
    tags: [],
    allowDuplicate: false,
  });
}

function normalizeApiUrl(value) {
  return (value || DEFAULT_URL).trim().replace(/\/$/, '');
}

async function captureImage(info, tab) {
  const config = await getConfig();

  if (!config.token) {
    await chrome.runtime.openOptionsPage();
    throw new Error('请先在扩展设置中填写 Lap 配对 Token');
  }

  const pageUrl = tab?.url || info.pageUrl || '';
  const payload = {
    sourceUrl: info.srcUrl,
    pageUrl,
    pageTitle: tab?.title || '',
    siteName: (() => {
      try {
        return new URL(pageUrl).hostname;
      } catch {
        return '';
      }
    })(),
    folderPath: config.folderPath || null,
    allowDuplicate: Boolean(config.allowDuplicate),
    metadata: {
      browser: 'chromium',
      capturedBy: 'lap-extension',
      capturedAt: new Date().toISOString(),
      frameUrl: info.frameUrl || null,
    },
    tags: Array.isArray(config.tags) ? config.tags : [],
  };

  const response = await fetch(`${normalizeApiUrl(config.apiUrl)}/capture`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Lap-Token': config.token,
    },
    body: JSON.stringify(payload),
  });

  const data = await response.json().catch(() => ({}));
  if (!response.ok || !data.ok) {
    throw new Error(data.error || `保存失败：HTTP ${response.status}`);
  }

  await chrome.action.setBadgeBackgroundColor({ color: data.duplicate ? '#8B5CF6' : '#16A34A' });
  await chrome.action.setBadgeText({ text: data.duplicate ? '重复' : '✓' });
  setTimeout(() => chrome.action.setBadgeText({ text: '' }), 1800);
  return data;
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: 'save-image-to-lap',
      title: '保存图片到 Lap',
      contexts: ['image'],
    });
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== 'save-image-to-lap') {
    return;
  }

  try {
    await captureImage(info, tab);
  } catch (error) {
    console.error(error);
    await chrome.action.setBadgeBackgroundColor({ color: '#DC2626' });
    await chrome.action.setBadgeText({ text: '!' });
    setTimeout(() => chrome.action.setBadgeText({ text: '' }), 2200);
  }
});

chrome.action.onClicked.addListener(() => chrome.runtime.openOptionsPage());
