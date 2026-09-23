import { getConfig, request, normalizeTags } from "./api.js";
let writes = Promise.resolve();
function serialize(action) {
  const next = writes.then(action);
  writes = next.catch(() => {});
  return next;
}
async function rememberFolder(path) {
  if (!path) return;
  await serialize(async () => {
    const { recentFolders } = await getConfig();
    await chrome.storage.local.set({
      recentFolders: [path, ...recentFolders.filter((x) => x !== path)].slice(
        0,
        8,
      ),
    });
  });
}
async function capture(payload) {
  const config = await getConfig();
  const source = new URL(payload.sourceUrl);
  if (!["http:", "https:"].includes(source.protocol))
    throw new Error("仅支持 HTTP 或 HTTPS 图片地址。");
  const result = await request("/capture", {
    method: "POST",
    body: JSON.stringify({
      ...payload,
      tags: normalizeTags(
        Array.isArray(payload.tags) ? payload.tags : config.tags,
      ),
      allowDuplicate: payload.allowDuplicate ?? Boolean(config.allowDuplicate),
    }),
  });
  await rememberFolder(payload.folderPath).catch(() => {});
  return result;
}
// Store each outcome before beginning another item. Never replay an uncertain POST.
let activeJob = null;
let starting = false;
async function saveJob(job) {
  await chrome.storage.local.set({ captureJob: structuredClone(job) });
}
function jobSummary(job) {
  if (!job) return null;
  return {
    id: job.id,
    status: job.status,
    total: job.items.length,
    completed: job.items.filter((x) =>
      ["saved", "duplicate", "failed", "unknown"].includes(x.status),
    ).length,
    saved: job.items.filter((x) => x.status === "saved").length,
    duplicate: job.items.filter((x) => x.status === "duplicate").length,
    failed: job.items.filter((x) => x.status === "failed").length,
    unknown: job.items.filter((x) => x.status === "unknown").length,
    errors: job.items
      .filter((x) => x.error)
      .map((x) => ({ sourceUrl: x.payload.sourceUrl, error: x.error }))
      .slice(0, 20),
  };
}
async function recoverJob() {
  if (activeJob) return activeJob;
  const { captureJob } = await chrome.storage.local.get("captureJob");
  if (activeJob) return activeJob;
  if (captureJob?.status === "running") {
    captureJob.status = "interrupted";
    for (const item of captureJob.items) {
      if (item.status === "saving") {
        item.status = "unknown";
        item.error = "浏览器任务中断，请先在 Lap 中确认是否已保存。";
      } else if (item.status === "pending") {
        item.status = "failed";
        item.error = "任务中断，尚未提交。";
      }
    }
    await saveJob(captureJob);
  }
  return captureJob || null;
}
async function runJob(job) {
  const keepAlive = setInterval(() => {
    void chrome.runtime.getPlatformInfo().catch(() => {});
  }, 20000);
  try {
    // Sequential writes avoid desktop URL-dedup races and deterministic progress is retained.
    for (const item of job.items) {
      item.status = "saving";
      await saveJob(job);
      try {
        const data = await capture(item.payload);
        item.status = data.duplicate ? "duplicate" : "saved";
      } catch (error) {
        item.status = /结果待确认/.test(error.message) ? "unknown" : "failed";
        item.error = error.message;
      }
      await saveJob(job);
    }
    job.status = "done";
    await saveJob(job);
  } catch (error) {
    job.status = "interrupted";
    for (const item of job.items) {
      if (item.status === "saving") {
        item.status = "unknown";
        item.error = "任务中断，请在 Lap 中确认保存结果。";
      }
      if (item.status === "pending") {
        item.status = "failed";
        item.error = "任务中断，尚未提交。";
      }
    }
    await saveJob(job).catch(() => {});
  } finally {
    clearInterval(keepAlive);
    activeJob = null;
  }
}
async function startBatch(payloads) {
  if (starting || activeJob)
    throw new Error("已有采集任务进行中，请等待完成。");
  starting = true;
  try {
    const items = (Array.isArray(payloads) ? payloads : []).slice(0, 300);
    if (!items.length) throw new Error("没有选择图片。");
    const config = await getConfig();
    await request("/folders", { config });
    const job = {
      id: crypto.randomUUID(),
      status: "running",
      items: items.map((payload) => ({ payload, status: "pending" })),
    };
    activeJob = job;
    try {
      await saveJob(job);
    } catch (error) {
      activeJob = null;
      throw error;
    }
    void runJob(job);
    return jobSummary(job);
  } finally {
    starting = false;
  }
}
chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === "install")
    void chrome.runtime.openOptionsPage().catch(() => {});
});
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (sender.id && sender.id !== chrome.runtime.id) return false;
  const run = async () => {
    switch (message?.type) {
      case "lap:get-state": {
        const config = await getConfig();
        const data = await request("/folders");
        if (!Array.isArray(data.folders))
          throw new Error("Lap 返回的目录格式不正确。");
        return {
          ok: true,
          folders: data.folders,
          aiConfigured: Boolean(data.aiConfigured),
          recentFolders: config.recentFolders,
          configured: true,
        };
      }
      case "lap:create-folder": {
        const name = String(message.name || "").trim();
        if (
          !name ||
          name.length > 100 ||
          /[<>:"/\\|?*\x00-\x1f]/.test(name) ||
          /[. ]$/.test(name) ||
          /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(name)
        )
          throw new Error("目录名称包含 Windows 不支持的字符或保留名称。");
        const data = await request("/folders", {
          method: "POST",
          body: JSON.stringify({ parentPath: message.parentPath || "", name }),
        });
        return { ok: true, folder: data.folder };
      }
      case "lap:get-folder-covers": {
        const folderIds = [
          ...new Set(
            (message.folderIds || [])
              .map(Number)
              .filter((x) => Number.isInteger(x) && x > 0),
          ),
        ].slice(0, 12);
        if (!folderIds.length) return { ok: true, covers: [] };
        const data = await request("/folder-covers", {
          method: "POST",
          body: JSON.stringify({ folderIds }),
        });
        return { ok: true, covers: data.covers || [] };
      }
      case "lap:capture":
        return { ok: true, result: await capture(message.payload || {}) };
      case "lap:start-batch":
        return { ok: true, job: await startBatch(message.payloads) };
      case "lap:get-job":
        return { ok: true, job: jobSummary(await recoverJob()) };
      case "lap:open-options":
        await chrome.runtime.openOptionsPage();
        return { ok: true };
      default:
        return { ok: false, error: "未知的扩展消息" };
    }
  };
  run()
    .then(sendResponse)
    .catch((error) =>
      sendResponse({ ok: false, error: error.message || String(error) }),
    );
  return true;
});
