import { getConfig, normalizeTags } from "./api.js";
const $ = (id) => document.getElementById(id);
const state = {
  config: null,
  connected: false,
  scanning: false,
  busy: false,
  images: [],
  selected: new Set(),
  tab: null,
  page: {},
  scanId: 0,
  job: null,
};
async function send(message) {
  const result = await chrome.runtime.sendMessage(message);
  if (!result?.ok)
    throw new Error(result?.error || "插件后台未响应，请刷新扩展。");
  return result;
}
function connection(ok, title, message) {
  state.connected = ok;
  $("connectionCard").className = `status-card ${ok ? "connected" : "error"}`;
  $("connectionTitle").textContent = title;
  $("connectionMessage").textContent = message;
  update();
}
async function connect() {
  try {
    const result = await send({ type: "lap:get-state" });
    renderFolders(result.folders, result.aiConfigured);
    connection(true, "已连接 Lap", "配对已记住 · 可直接保存素材");
    return true;
  } catch (error) {
    connection(false, "暂未连接", error.message);
    return false;
  }
}
function renderFolders(folders, aiConfigured) {
  $("folderSelect").replaceChildren();
  if (aiConfigured) $("folderSelect").add(new Option("AI 收件箱", ""));
  for (const f of folders) $("folderSelect").add(new Option(f.name, f.path));
  const selected = folders.some((f) => f.path === state.config.folderPath)
    ? state.config.folderPath
    : aiConfigured
      ? ""
      : folders[0]?.path || "";
  $("folderSelect").value = selected;
  if (!folders.length && !aiConfigured)
    $("folderSelect").add(new Option("请先在 Lap 中添加目录", ""));
  state.hasDestination = folders.length > 0 || aiConfigured;
}
function pageCollector() {
  const makeAbsolute = (value) => {
    try {
      return new URL(value, document.baseURI).href;
    } catch {
      return "";
    }
  };
  const candidates = [];
  for (const image of document.images) {
    const sourceUrl =
      [
        image.currentSrc,
        image.getAttribute("data-original"),
        image.getAttribute("data-src"),
        image.getAttribute("data-lazy-src"),
        image.getAttribute("src"),
      ]
        .filter(Boolean)
        .map(makeAbsolute)
        .find((url) => /^https?:/i.test(url)) || "";
    if (!sourceUrl) continue;
    const rect = image.getBoundingClientRect();
    const width = image.naturalWidth || Math.round(rect.width) || 0;
    const height = image.naturalHeight || Math.round(rect.height) || 0;
    if (width < 80 || height < 80) continue;
    candidates.push({
      sourceUrl,
      previewUrl: image.currentSrc || image.src || sourceUrl,
      altText: image.alt || image.getAttribute("aria-label") || "",
      width,
      height,
    });
  }
  const unique = new Map();
  for (const item of candidates) {
    const existing = unique.get(item.sourceUrl);
    if (
      !existing ||
      item.width * item.height > existing.width * existing.height
    )
      unique.set(item.sourceUrl, item);
  }
  const meta = (selector, attribute = "content") =>
    document.querySelector(selector)?.getAttribute(attribute) || "";
  return {
    images: [...unique.values()]
      .sort((a, b) => b.width * b.height - a.width * a.height)
      .slice(0, 300),
    page: {
      url: location.href,
      title: document.title,
      author:
        meta('meta[name="author"]') || meta('meta[property="article:author"]'),
      siteName: meta('meta[property="og:site_name"]') || location.hostname,
    },
  };
}

async function scanPage() {
  if (state.busy || state.scanning) return;
  state.scanning = true;
  const scanId = ++state.scanId;
  update();
  $("progressText").textContent = "正在扫描当前页面…";
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    state.tab = tab;
    if (!tab?.id || !/^https?:/i.test(tab.url || ""))
      throw new Error("此页面不支持采集，请切换到普通网页。");
    const [execution] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: pageCollector,
    });
    if (scanId !== state.scanId) return;
    const result = execution?.result || {};
    state.images = (result.images || [])
      .slice(0, 300)
      .map((image, id) => ({ ...image, id }));
    state.page = result.page || {};
    state.selected = new Set(state.images.slice(0, 24).map((x) => x.id));
    if (!state.job)
      $("progressText").textContent = state.images.length
        ? `找到 ${state.images.length} 张，已选择前 ${Math.min(24, state.images.length)} 张。`
        : "未找到尺寸大于 80 × 80 的图片。";
  } catch (error) {
    state.images = [];
    state.selected.clear();
    $("progressText").textContent = error.message;
  } finally {
    state.scanning = false;
    render();
  }
}
function render() {
  $("imageGrid").replaceChildren();
  $("emptyState").hidden = state.images.length > 0;
  for (const image of state.images) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = `asset-card${state.selected.has(image.id) ? " selected" : ""}`;
    card.setAttribute("aria-pressed", String(state.selected.has(image.id)));
    card.setAttribute(
      "aria-label",
      image.altText || `图片 ${image.id + 1}，${image.width} × ${image.height}`,
    );
    card.disabled = state.busy;
    const preview = document.createElement("img");
    preview.src = image.previewUrl;
    preview.alt = "";
    preview.loading = "lazy";
    preview.referrerPolicy = "no-referrer";
    const check = document.createElement("span");
    check.className = "check";
    check.textContent = "✓";
    check.setAttribute("aria-hidden", "true");
    const dims = document.createElement("span");
    dims.className = "dimensions";
    dims.textContent = `${image.width}×${image.height}`;
    card.append(preview, check, dims);
    card.addEventListener("click", () => {
      if (state.busy) return;
      if (state.selected.has(image.id)) state.selected.delete(image.id);
      else state.selected.add(image.id);
      card.classList.toggle("selected", state.selected.has(image.id));
      card.setAttribute("aria-pressed", String(state.selected.has(image.id)));
      update();
    });
    $("imageGrid").append(card);
  }
  update();
}
function update() {
  $("selectionSummary").textContent =
    `已选择 ${state.selected.size} / ${state.images.length}`;
  $("captureButton").disabled =
    state.busy ||
    state.scanning ||
    !state.connected ||
    !state.hasDestination ||
    !state.selected.size;
  $("captureButton").textContent = state.busy
    ? "后台保存中…"
    : `保存 ${state.selected.size} 张素材`;
  for (const id of [
    "refreshButton",
    "selectAllButton",
    "clearButton",
    "folderSelect",
    "tagsInput",
    "allowDuplicate",
  ])
    $(id).disabled = state.busy || state.scanning;
}
function payload(image) {
  return {
    sourceUrl: image.sourceUrl,
    pageUrl: state.page.url || state.tab?.url || "",
    pageTitle: state.page.title || "",
    author: state.page.author || "",
    siteName: state.page.siteName || "",
    altText: image.altText || "",
    folderPath: $("folderSelect").value || null,
    workflowStatus: $("folderSelect").value ? "selected" : "inbox",
    tags: normalizeTags($("tagsInput").value),
    allowDuplicate: $("allowDuplicate").checked,
    metadata: {
      browser: "chromium",
      capturedBy: "lap-extension-popup",
      capturedAt: new Date().toISOString(),
      width: image.width,
      height: image.height,
    },
  };
}
function showJob(job) {
  if (!job) return;
  state.job = job;
  state.busy = job.status === "running";
  $("progressText").textContent = state.busy
    ? `后台保存 ${job.completed} / ${job.total} · 关闭弹窗仍会继续`
    : `${job.status === "interrupted" ? "任务中断" : "采集完成"}：新增 ${job.saved}，重复 ${job.duplicate}，失败 ${job.failed}，待确认 ${job.unknown}。`;
  $("jobErrors").hidden = !job.errors?.length;
  $("jobErrors").textContent = (job.errors || [])
    .map((x) => x.error)
    .filter((x, i, a) => a.indexOf(x) === i)
    .join("；");
  update();
}
async function refreshJob() {
  try {
    const { job } = await send({ type: "lap:get-job" });
    const wasBusy = state.busy;
    showJob(job);
    if (wasBusy && !state.busy) {
      state.selected.clear();
      render();
    }
  } catch (error) {
    $("progressText").textContent = error.message;
  }
}
async function save() {
  if ($("captureButton").disabled) return;
  const items = state.images
    .filter((x) => state.selected.has(x.id))
    .map(payload);
  state.busy = true;
  update();
  try {
    await chrome.storage.local.set({
      folderPath: $("folderSelect").value,
      tags: normalizeTags($("tagsInput").value),
      allowDuplicate: $("allowDuplicate").checked,
    });
    const result = await send({ type: "lap:start-batch", payloads: items });
    showJob(result.job);
  } catch (error) {
    state.busy = false;
    $("progressText").textContent = error.message;
    update();
  }
}
$("settingsButton").addEventListener("click", () =>
  chrome.runtime.openOptionsPage(),
);
$("refreshButton").addEventListener("click", async () => {
  state.job = null;
  await connect();
  await scanPage();
});
$("selectAllButton").addEventListener("click", () => {
  if (state.busy) return;
  state.selected = new Set(state.images.map((x) => x.id));
  render();
});
$("clearButton").addEventListener("click", () => {
  if (state.busy) return;
  state.selected.clear();
  render();
});
$("captureButton").addEventListener("click", save);
try {
  state.config = await getConfig();
  $("tagsInput").value = normalizeTags(state.config.tags).join(", ");
  $("allowDuplicate").checked = state.config.allowDuplicate;
  await refreshJob();
  await connect();
  if (!state.busy) await scanPage();
} catch (error) {
  $("progressText").textContent = error.message;
}
setInterval(() => {
  if (state.busy) void refreshJob();
}, 800);
chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.captureJob) void refreshJob();
});
