const DEFAULT_URL = "http://127.0.0.1:47821";

async function getConfig() {
  const result = await chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: ""
  });
  return result;
}

async function captureImage(info, tab) {
  const config = await getConfig();

  if (!config.token) {
    throw new Error("请先在扩展设置中填写 Lap 配对 Token");
  }

  const payload = {
    sourceUrl: info.srcUrl,
    pageUrl: tab?.url || "",
    pageTitle: tab?.title || "",
    metadata: {
      browser: "chrome",
      capturedBy: "lap-extension"
    },
    tags: []
  };

  const response = await fetch(`${config.apiUrl}/capture`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Lap-Token": config.token
    },
    body: JSON.stringify(payload)
  });

  const data = await response.json();
  if (!response.ok || !data.ok) {
    throw new Error(data.error || "保存失败");
  }

  return data;
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "save-image-to-lap",
    title: "保存图片到 Lap",
    contexts: ["image"]
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== "save-image-to-lap") {
    return;
  }

  try {
    await captureImage(info, tab);
  } catch (error) {
    console.error(error);
  }
});
