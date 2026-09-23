import {
  DEFAULT_URL,
  getConfig,
  request,
  normalizeApiUrl,
  normalizeTags,
} from "./api.js";
const $ = (id) => document.getElementById(id);
let stored = null;
let foldersLoaded = false;
let busy = false;
function status(state, message) {
  $("connectionBadge").className = `badge ${state}`;
  $("connectionBadge").textContent = {
    online: "已连接",
    detected: "需要配对",
    checking: "连接中",
    offline: "未连接",
  }[state];
  $("connectionMessage").textContent = message;
  $("connectionMessage").className =
    `message ${state === "online" ? "success" : state === "offline" ? "error" : ""}`;
}
function setBusy(value) {
  busy = value;
  for (const id of [
    "detectButton",
    "testButton",
    "saveButton",
    "token",
    "showToken",
    "folderPath",
    "tags",
    "allowDuplicate",
  ])
    $(id).disabled = value;
}
function readConnection() {
  return {
    apiUrl: normalizeApiUrl($("apiUrl").value),
    token: $("token").value.trim(),
  };
}
function renderFolders(data) {
  const selected = $("folderPath").value || stored.folderPath || "";
  $("folderPath").replaceChildren(
    new Option(data.aiConfigured ? "AI 收件箱" : "第一个可用目录", ""),
  );
  for (const folder of data.folders || [])
    $("folderPath").add(
      new Option(`${folder.name} — ${folder.path}`, folder.path),
    );
  if (
    selected &&
    ![...$("folderPath").options].some((x) => x.value === selected)
  )
    $("folderPath").add(new Option(`原目录暂不可用 — ${selected}`, selected));
  $("folderPath").value = selected;
  foldersLoaded = true;
}
async function verify({ manual = false, saveRules = false } = {}) {
  if (busy) return;
  setBusy(true);
  status("checking", "正在连接本机 Lap…");
  try {
    const config = readConnection();
    if (!config.token) {
      const health = await request("/health", { config, timeout: 8000 });
      $("advancedSettings").open = !health.pairingSupported;
      status(
        "detected",
        health.pairingSupported
          ? "点击“连接并记住”，在 Lap 中核对确认码并允许连接。"
          : "当前桌面版本需手动配对一次，请粘贴 Token 后验证。",
      );
      return;
    }
    const data = await request("/folders", { config, timeout: 8000 });
    if (!Array.isArray(data.folders))
      throw new Error("服务返回的目录数据不正确。");
    const rules = saveRules
      ? {
          folderPath: foldersLoaded ? $("folderPath").value : stored.folderPath,
          tags: normalizeTags($("tags").value),
          allowDuplicate: $("allowDuplicate").checked,
        }
      : {};
    // Connection verification and persistence are one operation.
    await chrome.storage.local.set({ ...config, ...rules });
    stored = { ...stored, ...config, ...rules };
    renderFolders(data);
    $("advancedSettings").open = false;
    status(
      "online",
      saveRules
        ? "设置已保存，后续自动使用此配对。"
        : "已连接，配对已记住。下次启动无需重复填写。",
    );
  } catch (error) {
    status(error.status === 401 ? "detected" : "offline", error.message);
    if (error.status === 401 || manual) $("advancedSettings").open = true;
  } finally {
    setBusy(false);
  }
}
async function initialize() {
  try {
    stored = await getConfig();
    $("apiUrl").value = stored.apiUrl;
    $("token").value = stored.token;
    $("tags").value = normalizeTags(stored.tags).join("\n");
    $("allowDuplicate").checked = Boolean(stored.allowDuplicate);
    if (stored.folderPath) {
      $("folderPath").add(new Option(stored.folderPath, stored.folderPath));
      $("folderPath").value = stored.folderPath;
    }
    await verify();
  } catch (error) {
    status("offline", error.message);
  }
}
async function connect() {
  if (busy) return;
  if ($("token").value.trim()) {
    await verify({ manual: true });
    if ($("connectionBadge").classList.contains("online")) return;
    // An unavailable service must not discard an existing credential.
    if (!$("connectionBadge").classList.contains("detected")) return;
  }
  setBusy(true);
  try {
    const config = { apiUrl: DEFAULT_URL, token: "" };
    const health = await request("/health", { config, timeout: 8000 });
    if (!health.pairingSupported) {
      $("advancedSettings").open = true;
      status("detected", "此桌面版本请手动粘贴 Token 并验证一次。");
      return;
    }
    const pairing = await request("/pair/request", {
      config,
      method: "POST",
      body: "{}",
      timeout: 8000,
    });
    $("pairingCode").textContent = pairing.code;
    $("pairingPanel").hidden = false;
    status("checking", "请切换到 Lap，确认两处数字一致，然后允许连接。");
    const deadline = Date.now() + pairing.expiresIn * 1000;
    while (Date.now() < deadline) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const result = await request("/pair/status", {
        config,
        method: "POST",
        body: JSON.stringify({
          requestId: pairing.requestId,
          secret: pairing.secret,
        }),
        timeout: 8000,
      });
      if (result.status === "rejected")
        throw new Error("你已在 Lap 中拒绝连接。两分钟后可重试。");
      if (result.status === "approved") {
        if (typeof result.token !== "string" || result.token.length < 32)
          throw new Error("配对响应格式不正确。");
        config.token = result.token;
        const data = await request("/folders", { config, timeout: 8000 });
        if (!Array.isArray(data.folders))
          throw new Error("服务返回的目录数据不正确。");
        await chrome.storage.local.set(config);
        stored = { ...stored, ...config };
        $("token").value = config.token;
        $("apiUrl").value = config.apiUrl;
        renderFolders(data);
        $("advancedSettings").open = false;
        status("online", "已连接并记住。以后启动 Lap 即可采集，无需重复授权。");
        return;
      }
    }
    throw new Error("配对已超时，请重新连接。");
  } catch (error) {
    status("offline", error.message);
  } finally {
    $("pairingPanel").hidden = true;
    setBusy(false);
  }
}
$("detectButton").addEventListener("click", connect);
$("testButton").addEventListener("click", () => verify({ manual: true }));
$("saveButton").addEventListener("click", () =>
  verify({ manual: true, saveRules: true }),
);
$("showToken").addEventListener("click", () => {
  const hidden = $("token").type === "password";
  $("token").type = hidden ? "text" : "password";
  $("showToken").textContent = hidden ? "隐藏" : "显示";
  $("showToken").setAttribute("aria-pressed", String(hidden));
});
void initialize();
