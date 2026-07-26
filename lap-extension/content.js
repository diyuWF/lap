(() => {
  const HOVER_EXPAND_MS = 520;
  const CLOSE_AFTER_DRAG_MS = 1200;

  let overlay = null;
  let mode = 'idle';
  let currentAsset = null;
  let folders = [];
  let recentFolderPaths = [];
  let expanded = new Set();
  let hoverTimer = null;
  let selectedFolder = null;
  let dragGhost = null;

  function sendMessage(message) {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(message, (response) => {
        const error = chrome.runtime.lastError;
        if (error) {
          reject(new Error(error.message));
          return;
        }
        resolve(response);
      });
    });
  }

  function normalizePath(value) {
    return String(value || '')
      .replace(/\\/g, '/')
      .replace(/\/{2,}/g, '/')
      .replace(/\/$/, '');
  }

  function pathName(value) {
    const normalized = normalizePath(value);
    return normalized.split('/').filter(Boolean).at(-1) || normalized;
  }

  function sourceFileName(sourceUrl) {
    try {
      return pathName(new URL(sourceUrl, location.href).pathname) || '网页图片';
    } catch {
      return '网页图片';
    }
  }

  function findDraggedImage(target) {
    const image = target instanceof HTMLImageElement
      ? target
      : target instanceof Element
        ? target.closest('img')
        : null;
    if (!(image instanceof HTMLImageElement)) return null;
    const sourceUrl = image.currentSrc || image.src;
    if (!sourceUrl || sourceUrl.startsWith('data:') || sourceUrl.startsWith('blob:')) return null;
    return image;
  }

  function createAsset(image) {
    return {
      sourceUrl: image.currentSrc || image.src,
      altText: image.alt || image.getAttribute('aria-label') || '',
      naturalWidth: image.naturalWidth || 0,
      naturalHeight: image.naturalHeight || 0,
      displayWidth: Math.round(image.getBoundingClientRect().width),
      displayHeight: Math.round(image.getBoundingClientRect().height),
    };
  }

  function clearHoverTimer() {
    if (hoverTimer) clearTimeout(hoverTimer);
    hoverTimer = null;
  }

  function removeDragGhost() {
    dragGhost?.remove();
    dragGhost = null;
  }

  function closeOverlay() {
    clearHoverTimer();
    removeDragGhost();
    overlay?.remove();
    overlay = null;
    mode = 'idle';
    currentAsset = null;
    selectedFolder = null;
    folders = [];
    recentFolderPaths = [];
    expanded = new Set();
  }

  function escapeHtml(value) {
    return String(value ?? '')
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
      .replaceAll('"', '&quot;')
      .replaceAll("'", '&#039;');
  }

  function createDragGhost(image, dataTransfer) {
    if (!dataTransfer) return;
    removeDragGhost();

    dragGhost = document.createElement('div');
    dragGhost.className = 'lap-capture-drag-ghost';

    const thumbnail = image.cloneNode(false);
    thumbnail.className = 'lap-capture-drag-ghost-image';
    thumbnail.removeAttribute('id');
    thumbnail.removeAttribute('srcset');
    thumbnail.removeAttribute('sizes');
    thumbnail.src = image.currentSrc || image.src;
    thumbnail.alt = '';
    thumbnail.draggable = false;

    const copy = document.createElement('div');
    copy.className = 'lap-capture-drag-ghost-copy';
    copy.innerHTML = '<strong>保存到 Lap</strong><span>拖到文件夹，松开后确认</span>';

    const badge = document.createElement('span');
    badge.className = 'lap-capture-drag-ghost-badge';
    badge.textContent = '复制';

    dragGhost.append(thumbnail, copy, badge);
    document.documentElement.appendChild(dragGhost);

    try {
      dataTransfer.effectAllowed = 'copy';
      dataTransfer.setData('application/x-lap-capture', currentAsset.sourceUrl);
      dataTransfer.setDragImage(dragGhost, 30, 30);
    } catch {
      // Some pages restrict custom drag data. The overlay remains usable.
    }
  }

  function createOverlay() {
    mode = 'loading';
    overlay = document.createElement('div');
    overlay.className = 'lap-capture-overlay';
    overlay.innerHTML = `
      <section class="lap-capture-panel" role="dialog" aria-label="保存到 Lap">
        <header class="lap-capture-header">
          <div class="lap-capture-brand">
            <span class="lap-capture-logo">L</span>
            <div>
              <strong>保存到 Lap</strong>
              <span>拖到文件夹上，停留自动展开</span>
            </div>
          </div>
          <button class="lap-capture-close" type="button" aria-label="关闭">×</button>
        </header>
        <div class="lap-capture-body">
          <aside class="lap-capture-preview">
            <div class="lap-capture-preview-frame">
              <img class="lap-capture-preview-image" alt="待保存素材预览" />
              <div class="lap-capture-preview-fallback" hidden>
                <span class="lap-capture-preview-fallback-icon">图</span>
                <strong>网页预览不可用</strong>
                <span>仍可保存原始图片地址</span>
              </div>
            </div>
            <div class="lap-capture-preview-meta"></div>
          </aside>
          <main class="lap-capture-main">
            <div class="lap-capture-status">正在读取 Lap 文件夹…</div>
            <div class="lap-capture-folder-area" hidden>
              <section class="lap-capture-recent" hidden>
                <div class="lap-capture-section-title">最近使用</div>
                <div class="lap-capture-recent-list"></div>
              </section>
              <section>
                <div class="lap-capture-section-title">全部文件夹</div>
                <div class="lap-capture-tree"></div>
              </section>
            </div>
            <div class="lap-capture-confirm" hidden></div>
          </main>
        </div>
      </section>`;

    overlay.addEventListener('dragover', (event) => {
      if (mode === 'picking') event.preventDefault();
    });
    overlay.querySelector('.lap-capture-close').addEventListener('click', closeOverlay);
    document.documentElement.appendChild(overlay);

    const preview = overlay.querySelector('.lap-capture-preview-image');
    const previewFallback = overlay.querySelector('.lap-capture-preview-fallback');
    preview.addEventListener('error', () => {
      preview.hidden = true;
      previewFallback.hidden = false;
    }, { once: true });
    preview.src = currentAsset.sourceUrl;
    const dimensions = currentAsset.naturalWidth && currentAsset.naturalHeight
      ? `${currentAsset.naturalWidth} × ${currentAsset.naturalHeight}`
      : '尺寸未知';
    overlay.querySelector('.lap-capture-preview-meta').innerHTML = `
      <strong>${escapeHtml(sourceFileName(currentAsset.sourceUrl))}</strong>
      <span>${escapeHtml(dimensions)}</span>
      <span>${escapeHtml(location.hostname)}</span>`;
  }

  function setStatus(message, kind = 'normal', action = null) {
    const status = overlay?.querySelector('.lap-capture-status');
    if (!status) return;
    status.hidden = false;
    status.className = `lap-capture-status is-${kind}`;
    status.innerHTML = `<span>${escapeHtml(message)}</span>`;
    if (action) {
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = action.label;
      button.addEventListener('click', action.run);
      status.appendChild(button);
    }
  }

  function buildFolderTree() {
    const records = folders
      .map((folder) => ({
        ...folder,
        key: normalizePath(folder.path),
        children: [],
      }))
      .filter((folder) => folder.key);
    const byKey = new Map(records.map((folder) => [folder.key.toLowerCase(), folder]));
    const roots = [];

    for (const folder of records) {
      const parts = folder.key.split('/');
      parts.pop();
      let parent = null;
      while (parts.length) {
        const candidate = byKey.get(parts.join('/').toLowerCase());
        if (candidate) {
          parent = candidate;
          break;
        }
        parts.pop();
      }
      if (parent) parent.children.push(folder);
      else roots.push(folder);
    }

    const sortNodes = (nodes) => {
      nodes.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN', { numeric: true }));
      nodes.forEach((node) => sortNodes(node.children));
    };
    sortNodes(roots);
    return { roots, byKey };
  }

  function scheduleExpand(folder, row) {
    clearHoverTimer();
    if (!folder.children.length || expanded.has(folder.key)) return;
    row.classList.add('is-waiting');
    hoverTimer = setTimeout(() => {
      expanded.add(folder.key);
      renderFolders();
    }, HOVER_EXPAND_MS);
  }

  function cancelExpand(row) {
    clearHoverTimer();
    row?.classList.remove('is-waiting');
  }

  function createFolderRow(folder, depth = 0, recent = false) {
    const row = document.createElement('div');
    row.className = `lap-capture-folder-row${recent ? ' is-recent' : ''}`;
    row.dataset.path = folder.path;
    row.style.setProperty('--lap-depth', String(depth));

    const hasChildren = folder.children?.length > 0;
    const isExpanded = expanded.has(folder.key);
    row.innerHTML = `
      <span class="lap-capture-chevron ${hasChildren ? '' : 'is-empty'}">${hasChildren ? (isExpanded ? '⌄' : '›') : ''}</span>
      <span class="lap-capture-folder-icon">${isExpanded && hasChildren ? '▾' : '▪'}</span>
      <span class="lap-capture-folder-name">${escapeHtml(folder.name)}</span>
      ${hasChildren ? `<span class="lap-capture-child-count">${folder.children.length}</span>` : ''}`;

    row.addEventListener('dragenter', (event) => {
      if (mode !== 'picking') return;
      event.preventDefault();
      row.classList.add('is-target');
      scheduleExpand(folder, row);
    });
    row.addEventListener('dragover', (event) => {
      if (mode !== 'picking') return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    });
    row.addEventListener('dragleave', (event) => {
      if (row.contains(event.relatedTarget)) return;
      row.classList.remove('is-target');
      cancelExpand(row);
    });
    row.addEventListener('drop', (event) => {
      if (mode !== 'picking') return;
      event.preventDefault();
      event.stopPropagation();
      cancelExpand(row);
      selectedFolder = folder;
      showConfirmation();
    });
    row.addEventListener('click', () => {
      if (!hasChildren || mode !== 'picking') return;
      if (expanded.has(folder.key)) expanded.delete(folder.key);
      else expanded.add(folder.key);
      renderFolders();
    });
    return row;
  }

  function renderFolders() {
    if (!overlay) return;
    const { roots, byKey } = buildFolderTree();
    const tree = overlay.querySelector('.lap-capture-tree');
    tree.textContent = '';

    const renderNode = (folder, depth) => {
      tree.appendChild(createFolderRow(folder, depth));
      if (expanded.has(folder.key)) {
        folder.children.forEach((child) => renderNode(child, depth + 1));
      }
    };
    roots.forEach((folder) => renderNode(folder, 0));

    const recentSection = overlay.querySelector('.lap-capture-recent');
    const recentList = overlay.querySelector('.lap-capture-recent-list');
    recentList.textContent = '';
    const recent = recentFolderPaths
      .map((path) => byKey.get(normalizePath(path).toLowerCase()))
      .filter(Boolean)
      .slice(0, 5);
    recentSection.hidden = recent.length === 0;
    recent.forEach((folder) => recentList.appendChild(createFolderRow(folder, 0, true)));
  }

  async function loadFolderState() {
    const response = await sendMessage({ type: 'lap:get-state' });
    if (!response?.ok) throw new Error(response?.error || '无法读取 Lap 文件夹');
    folders = response.folders || [];
    recentFolderPaths = response.recentFolders || [];
    if (!folders.length) throw new Error('Lap 中还没有可用文件夹，请先添加资料库文件夹。');

    mode = 'picking';
    overlay.querySelector('.lap-capture-status').hidden = true;
    overlay.querySelector('.lap-capture-folder-area').hidden = false;
    renderFolders();
  }

  function showConfirmation() {
    mode = 'confirming';
    clearHoverTimer();
    removeDragGhost();
    overlay.querySelector('.lap-capture-folder-area').hidden = true;
    overlay.querySelector('.lap-capture-status').hidden = true;
    const confirm = overlay.querySelector('.lap-capture-confirm');
    confirm.hidden = false;
    confirm.innerHTML = `
      <div class="lap-capture-confirm-heading">
        <span class="lap-capture-confirm-icon">✓</span>
        <div>
          <strong>确认保存位置</strong>
          <span>${escapeHtml(selectedFolder.path)}</span>
        </div>
      </div>
      <label class="lap-capture-field">
        <span>标签</span>
        <textarea rows="3" placeholder="每行一个，例如：style:minimal"></textarea>
      </label>
      <label class="lap-capture-check">
        <input type="checkbox" checked />
        <span>
          <strong>保存后交给 AI 自动整理</strong>
          <small>在线 AI 配置可用时生成标题、描述、标签和分类建议</small>
        </span>
      </label>
      <div class="lap-capture-actions">
        <button class="lap-capture-button is-secondary" data-action="back" type="button">返回选择</button>
        <button class="lap-capture-button is-primary" data-action="save" type="button">保存到此文件夹</button>
      </div>`;

    confirm.querySelector('[data-action="back"]').addEventListener('click', () => {
      mode = 'picking';
      selectedFolder = null;
      confirm.hidden = true;
      overlay.querySelector('.lap-capture-folder-area').hidden = false;
    });
    confirm.querySelector('[data-action="save"]').addEventListener('click', saveConfirmedAsset);
  }

  function parseTags(value) {
    return String(value || '')
      .split(/[\n,]/)
      .map((tag) => tag.trim())
      .filter(Boolean)
      .filter((tag, index, all) => all.findIndex((item) => item.toLowerCase() === tag.toLowerCase()) === index)
      .slice(0, 50);
  }

  function showSaveError(confirm, saveButton, message) {
    mode = 'confirming';
    saveButton.disabled = false;
    saveButton.textContent = '重试保存';
    confirm.querySelector('.lap-capture-inline-error')?.remove();
    const error = document.createElement('div');
    error.className = 'lap-capture-inline-error';
    error.textContent = message || '保存失败';
    confirm.insertBefore(error, confirm.querySelector('.lap-capture-actions'));
  }

  async function saveConfirmedAsset() {
    if (!currentAsset || !selectedFolder || mode === 'saving') return;
    mode = 'saving';
    const confirm = overlay.querySelector('.lap-capture-confirm');
    const saveButton = confirm.querySelector('[data-action="save"]');
    const tags = parseTags(confirm.querySelector('textarea').value);
    const aiAutoOrganize = confirm.querySelector('input[type="checkbox"]').checked;
    saveButton.disabled = true;
    saveButton.textContent = '正在保存…';

    try {
      const response = await sendMessage({
        type: 'lap:capture',
        payload: {
          sourceUrl: currentAsset.sourceUrl,
          pageUrl: location.href,
          pageTitle: document.title,
          siteName: location.hostname,
          altText: currentAsset.altText,
          folderPath: selectedFolder.path,
          tags,
          metadata: {
            capturedBy: 'lap-drag-capture',
            capturedAt: new Date().toISOString(),
            aiAutoOrganize,
            naturalWidth: currentAsset.naturalWidth,
            naturalHeight: currentAsset.naturalHeight,
            displayWidth: currentAsset.displayWidth,
            displayHeight: currentAsset.displayHeight,
          },
        },
      });

      if (!response?.ok) {
        showSaveError(confirm, saveButton, response?.error || '保存失败');
        return;
      }

      mode = 'complete';
      confirm.innerHTML = `
        <div class="lap-capture-complete">
          <span class="lap-capture-complete-icon">✓</span>
          <strong>${response.result?.duplicate ? '素材已存在' : '已保存到 Lap'}</strong>
          <span>${escapeHtml(selectedFolder.path)}</span>
        </div>`;
      setTimeout(closeOverlay, 1100);
    } catch (error) {
      showSaveError(confirm, saveButton, error instanceof Error ? error.message : String(error));
    }
  }

  async function beginCapture(image, event) {
    closeOverlay();
    currentAsset = createAsset(image);
    createOverlay();
    createDragGhost(image, event.dataTransfer);
    try {
      await loadFolderState();
    } catch (error) {
      mode = 'error';
      setStatus(
        error instanceof Error ? error.message : String(error),
        'error',
        {
          label: '打开扩展设置',
          run: () => sendMessage({ type: 'lap:open-options' }),
        },
      );
    }
  }

  document.addEventListener('dragstart', (event) => {
    const image = findDraggedImage(event.target);
    if (!image) return;
    beginCapture(image, event);
  }, true);

  document.addEventListener('dragend', () => {
    removeDragGhost();
    if (!overlay || !['loading', 'picking'].includes(mode)) return;
    setTimeout(() => {
      if (overlay && ['loading', 'picking'].includes(mode)) closeOverlay();
    }, CLOSE_AFTER_DRAG_MS);
  }, true);

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && overlay) closeOverlay();
  }, true);
})();
