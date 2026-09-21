export const DEFAULT_URL = "http://127.0.0.1:47821";
export function normalizeApiUrl(value) {
  const url = new URL(String(value || DEFAULT_URL).trim());
  if (
    url.origin !== DEFAULT_URL ||
    !["", "/"].includes(url.pathname) ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    throw new Error("采集服务地址须为 http://127.0.0.1:47821");
  }
  return DEFAULT_URL;
}
export function normalizeTags(value) {
  const seen = new Set();
  return (Array.isArray(value) ? value : String(value || "").split(/[,，\n]/))
    .map((x) => String(x).trim())
    .filter((x) => x && !seen.has(x.toLowerCase()) && seen.add(x.toLowerCase()))
    .slice(0, 50);
}
export async function getConfig() {
  return chrome.storage.local.get({
    apiUrl: DEFAULT_URL,
    token: "",
    folderPath: "",
    tags: [],
    allowDuplicate: false,
    recentFolders: [],
  });
}
export async function request(
  path,
  { config, timeout = 25000, ...options } = {},
) {
  config ||= await getConfig();
  const url = normalizeApiUrl(config.apiUrl);
  if (!config.token && path !== "/health" && !path.startsWith("/pair/"))
    throw Object.assign(
      new Error("首次使用请打开设置连接 Lap，授权后会自动记住。"),
      { status: 401 },
    );
  const headers = new Headers(options.headers || {});
  if (config.token && path !== "/health")
    headers.set("X-Lap-Token", config.token);
  if (options.body) headers.set("Content-Type", "application/json");
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  try {
    const response = await fetch(url + path, {
      ...options,
      headers,
      signal: controller.signal,
      cache: "no-store",
      redirect: "error",
      credentials: "omit",
    });
    let data;
    try {
      data = await response.json();
    } catch {
      throw new Error(
        options.method === "POST"
          ? "本地服务返回了无法识别的数据，结果待确认。请先检查 Lap 中的保存记录。"
          : "本地服务返回了无法识别的数据，请确认运行的是 Lap。",
      );
    }
    if (!response.ok || data?.ok === false) {
      const message =
        response.status === 401
          ? "已找到 Lap，但配对凭据已失效。请在设置中重新连接一次。"
          : String(data?.error || `请求失败：HTTP ${response.status}`);
      throw Object.assign(new Error(message), { status: response.status });
    }
    if (!data || typeof data !== "object")
      throw new Error("本地服务响应格式不正确。");
    return data;
  } catch (error) {
    if (controller.signal.aborted)
      throw new Error(
        options.method === "POST"
          ? "请求超时，结果待确认。请先查看 Lap 是否已保存，避免重复提交。"
          : "Lap 响应超时，请确认桌面应用运行正常。",
      );
    if (error instanceof TypeError)
      throw new Error(
        options.method === "POST"
          ? "连接中断，结果待确认。请先查看 Lap 是否已保存。"
          : "未连接到 Lap，请启动桌面应用后重试。",
      );
    throw error;
  } finally {
    clearTimeout(timer);
  }
}
