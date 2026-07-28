(() => {
  const HOVER_EXPAND_MS = 520;
  const INTENT_CONFIRM_MS = 500;
  const CLOSE_AFTER_DRAG_MS = 160;
  const RADIAL_SIZE = 328;

  let overlay = null;
  let mode = 'idle';
  let currentAsset = null;
  let folders = [];
  let recentFolderPaths = [];
  let expanded = new Set();
  let hoverTimer = null;
  let intentTimer = null;
  let intentReady = false;
  let foldersReady = false;
  let folderLoadError = null;
  let lastDragPoint = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
  let radialAnchor = null;
  let selectedFolder = null;
  let dragGhost = null;
  let armedImage = null;
  let armedDraggableValue = null;
  let captureSession = 0;

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

  function normalizeHttpUrl(value) {
    if (!value) return '';
    try {
      const url = new URL(value, location.href);
      return ['http:', 'https:'].includes(url.protocol) ? url.href : '';
    } catch {
      return '';
    }
  }

  function resolveImageSource(image) {
    if (!(image instanceof HTMLImageElement)) return '';
    const candidates = [
      image.currentSrc,
      image.src,
      image.getAttribute('data-src'),
      image.getAttribute('data-original'),
      image.getAttribute('data-lazy-src'),
      image.getAttribute('data-pin-media'),
    ];
    const srcset = image.getAttribute('srcset') || image.getAttribute('data-srcset') || '';
    candidates.push(
      ...srcset
        .split(',')
        .map((entry) => entry.trim().split(/\s+/)[0])
        .reverse(),
    );
    const linkedSource = image.closest('a[href]')?.getAttribute('href');
    if (linkedSource && /\.(?:avif|gif|jpe?g|png|webp)(?:[?#]|$)/i.test(linkedSource)) {
      candidates.push(linkedSource);
    }
    for (const candidate of candidates) {
      const sourceUrl = normalizeHttpUrl(candidate);
      if (sourceUrl) return sourceUrl;
    }
    return '';
  }

  function imageFromElement(target) {
    if (target instanceof HTMLImageElement) return target;
    if (!(target instanceof Element)) return null;
    const direct = target.closest('img');
    if (direct instanceof HTMLImageElement) return direct;
    const container = target.closest('a, button, figure, [role="button"]');
    const nested = container?.querySelector('img');
    return nested instanceof HTMLImageElement ? nested : null;
  }

  function findDraggedImage(target, clientX = 0, clientY = 0) {
    const candidates = [imageFromElement(target)];
    if (typeof document.elementsFromPoint === 'function' && clientX > 0 && clientY > 0) {
      for (const element of document.elementsFromPoint(clientX, clientY)) {
        candidates.push(imageFromElement(element));
      }
    }
    for (const image of candidates) {
      if (image instanceof HTMLImageElement && resolveImageSource(image)) return image;
    }
    return null;
  }

  function restoreArmedImage() {
    if (!(armedImage instanceof HTMLImageElement)) return;
    if (armedDraggableValue === null) armedImage.removeAttribute('draggable');
    else armedImage.setAttribute('draggable', armedDraggableValue);
    armedImage = null;
    armedDraggableValue = null;
  }

  function armImageForNativeDrag(event) {
    const image = findDraggedImage(event.target, event.clientX, event.clientY);
    if (!image || image === armedImage) return;
    restoreArmedImage();
    armedImage = image;
    armedDraggableValue = image.getAttribute('draggable');
    image.setAttribute('draggable', 'true');
  }

  function createAsset(image) {
    return {
      sourceUrl: resolveImageSource(image),
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

  function clearIntentTimer() {
    if (intentTimer) clearTimeout(intentTimer);
    intentTimer = null;
  }

  function removeDragGhost() {
    dragGhost?.remove();
    dragGhost = null;
  }

  function closeOverlay() {
    captureSession += 1;
    clearHoverTimer();
    clearIntentTimer();
    removeDragGhost();
    overlay?.remove();
    overlay = null;
    mode = 'idle';
    currentAsset = null;
    selectedFolder = null;
    folders = [];
    recentFolderPaths = [];
    expanded = new Set();
    intentReady = false;
    foldersReady = false;
    folderLoadError = null;
    radialAnchor = null;
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

    dragGhost = image.cloneNode(false);
    dragGhost.className = 'lap-capture-drag-ghost';
    dragGhost.removeAttribute('id');
    dragGhost.removeAttribute('srcset');
    dragGhost.removeAttribute('sizes');
    dragGhost.removeAttribute('style');
    dragGhost.removeAttribute('width');
    dragGhost.removeAttribute('height');
    dragGhost.src = currentAsset.sourceUrl;
    dragGhost.alt = '';
    dragGhost.draggable = false;

    const sourceWidth = Math.max(currentAsset.displayWidth, currentAsset.naturalWidth, 16);
    const sourceHeight = Math.max(currentAsset.displayHeight, currentAsset.naturalHeight, 16);
    const scale = Math.min(180 / sourceWidth, 140 / sourceHeight, 1);
    const width = Math.max(12, Math.round(sourceWidth * scale));
    const height = Math.max(12, Math.round(sourceHeight * scale));
    dragGhost.style.width = `${width}px`;
    dragGhost.style.height = `${height}px`;
    document.documentElement.appendChild(dragGhost);

    try {
      dataTransfer.effectAllowed = 'copy';
      dataTransfer.setData('application/x-lap-capture', currentAsset.sourceUrl);
      dataTransfer.setDragImage(dragGhost, Math.round(width / 2), Math.round(height / 2));
    } catch {
      // Some pages restrict custom drag data. The overlay remains usable.
    }
  }

  function createOverlay() {
    mode = 'intent';
    overlay = document.createElement('div');
    overlay.className = 'lap-capture-overlay';
    const extensionIconUrl = chrome.runtime.getURL('icon.png');
    overlay.innerHTML = `
      <div class="lap-capture-radial" role="menu" aria-label="保存到文件夹" hidden>
        <button class="lap-capture-radial-center" type="button" role="menuitem">
          <strong>保存到待整理区域</strong>
          <span>松开后可交给 AI 分类</span>
        </button>
        <div class="lap-capture-radial-items"></div>
      </div>
      <section class="lap-capture-panel" role="dialog" aria-label="保存到待整理区域" hidden>
        <header class="lap-capture-header">
          <div class="lap-capture-brand">
            <img class="lap-capture-logo" src="${escapeHtml(extensionIconUrl)}" alt="" />
            <div>
              <strong>保存到待整理区域</strong>
              <span>先收集，再由 AI 生成分类方案</span>
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

    overlay.querySelector('.lap-capture-close').addEventListener('click', closeOverlay);
    document.documentElement.appendChild(overlay);
    positionCaptureUi(lastDragPoint.x, lastDragPoint.y);

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

  function positionCaptureUi(clientX, clientY) {
    if (!overlay) return;
    const x = Number.isFinite(clientX) && clientX > 0 ? clientX : window.innerWidth / 2;
    const y = Number.isFinite(clientY) && clientY > 0 ? clientY : window.innerHeight / 2;
    lastDragPoint = { x, y };

    const radial = overlay.querySelector('.lap-capture-radial');
    if (radial && !radial.hidden) {
      const anchor = radialAnchor || { x, y };
      const left = Math.min(
        Math.max(10, anchor.x - RADIAL_SIZE / 2),
        Math.max(10, window.innerWidth - RADIAL_SIZE - 10),
      );
      const top = Math.min(
        Math.max(10, anchor.y - RADIAL_SIZE / 2),
        Math.max(10, window.innerHeight - RADIAL_SIZE - 10),
      );
      radial.style.left = `${left}px`;
      radial.style.top = `${top}px`;
    }
  }

  function hideIntentUi() {
    if (!overlay) return;
    overlay.querySelector('.lap-capture-radial').hidden = true;
  }

  function openPanel() {
    if (!overlay) return;
    hideIntentUi();
    overlay.classList.add('is-panel-open');
    overlay.querySelector('.lap-capture-panel').hidden = false;
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

  function radialFolderCandidates() {
    const { roots, byKey } = buildFolderTree();
    const candidates = [];
    const seen = new Set();
    const add = (folder) => {
      if (!folder || seen.has(folder.key)) return;
      seen.add(folder.key);
      candidates.push(folder);
    };
    recentFolderPaths
      .map((path) => byKey.get(normalizePath(path).toLowerCase()))
      .forEach(add);
    roots.forEach(add);
    [...byKey.values()].forEach(add);
    return candidates.slice(0, 5);
  }

  function createRadialFolder(folder, index, total) {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'lap-capture-radial-item';
    button.dataset.path = folder.path;
    button.setAttribute('role', 'menuitem');
    button.title = folder.path;
    button.innerHTML = `
      <span class="lap-capture-radial-folder-mark" aria-hidden="true">目录</span>
      <span>${escapeHtml(folder.name)}</span>`;

    const angle = -Math.PI / 2 + (Math.PI * 2 * index) / total;
    button.style.setProperty('--lap-radial-x', `${Math.cos(angle) * 116}px`);
    button.style.setProperty('--lap-radial-y', `${Math.sin(angle) * 116}px`);

    button.addEventListener('dragenter', (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      button.classList.add('is-target');
    });
    button.addEventListener('dragover', (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    });
    button.addEventListener('dragleave', (event) => {
      if (button.contains(event.relatedTarget)) return;
      button.classList.remove('is-target');
    });
    button.addEventListener('drop', (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      event.stopPropagation();
      selectedFolder = folder;
      showConfirmation();
    });
    return button;
  }

  function showFolderBrowser() {
    if (!overlay) return;
    mode = 'browsing';
    openPanel();
    overlay.querySelector('.lap-capture-status').hidden = true;
    overlay.querySelector('.lap-capture-confirm').hidden = true;
    overlay.querySelector('.lap-capture-folder-area').hidden = false;
    renderFolders();
  }

  function showRadialMenu() {
    if (!overlay || !foldersReady || folderLoadError) return;
    mode = 'radial';
    const radial = overlay.querySelector('.lap-capture-radial');
    const items = overlay.querySelector('.lap-capture-radial-items');
    const candidates = radialFolderCandidates();
    const showMore = folders.length > candidates.length;
    const total = candidates.length + (showMore ? 1 : 0);

    radialAnchor = { ...lastDragPoint };
    radial.hidden = false;
    items.textContent = '';
    const inboxTarget = radial.querySelector('.lap-capture-radial-center');
    const selectInbox = () => {
      selectedFolder = {
        id: null,
        name: '待整理区域',
        path: '',
        isInbox: true,
      };
      showConfirmation();
    };
    inboxTarget.classList.remove('is-target');
    inboxTarget.ondragenter = (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      inboxTarget.classList.add('is-target');
    };
    inboxTarget.ondragover = (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    };
    inboxTarget.ondragleave = (event) => {
      if (inboxTarget.contains(event.relatedTarget)) return;
      inboxTarget.classList.remove('is-target');
    };
    inboxTarget.ondrop = (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      event.stopPropagation();
      selectInbox();
    };
    inboxTarget.onclick = selectInbox;
    candidates.forEach((folder, index) => {
      items.appendChild(createRadialFolder(folder, index, total));
    });

    if (showMore) {
      const more = document.createElement('button');
      more.type = 'button';
      more.className = 'lap-capture-radial-item is-more';
      more.setAttribute('role', 'menuitem');
      more.innerHTML = '<strong>更多</strong><span>全部文件夹</span>';
      const angle = -Math.PI / 2 + (Math.PI * 2 * candidates.length) / total;
      more.style.setProperty('--lap-radial-x', `${Math.cos(angle) * 116}px`);
      more.style.setProperty('--lap-radial-y', `${Math.sin(angle) * 116}px`);
      more.addEventListener('dragenter', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        more.classList.add('is-target');
      });
      more.addEventListener('dragover', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
      });
      more.addEventListener('dragleave', () => more.classList.remove('is-target'));
      more.addEventListener('drop', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        event.stopPropagation();
        removeDragGhost();
        showFolderBrowser();
      });
      more.addEventListener('click', showFolderBrowser);
      items.appendChild(more);
    }
    positionCaptureUi(lastDragPoint.x, lastDragPoint.y);
  }

  function resolveIntentState() {
    if (!overlay || !intentReady || mode !== 'intent') return;
    if (folderLoadError) {
      mode = 'error';
      openPanel();
      setStatus(folderLoadError, 'error', {
        label: '打开扩展设置',
        run: () => sendMessage({ type: 'lap:open-options' }),
      });
      return;
    }
    if (foldersReady) {
      showRadialMenu();
    }
  }

  function startIntentConfirmation() {
    clearIntentTimer();
    intentTimer = setTimeout(() => {
      intentTimer = null;
      intentReady = true;
      resolveIntentState();
    }, INTENT_CONFIRM_MS);
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
      if (mode !== 'browsing') return;
      event.preventDefault();
      row.classList.add('is-target');
      scheduleExpand(folder, row);
    });
    row.addEventListener('dragover', (event) => {
      if (mode !== 'browsing') return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    });
    row.addEventListener('dragleave', (event) => {
      if (row.contains(event.relatedTarget)) return;
      row.classList.remove('is-target');
      cancelExpand(row);
    });
    row.addEventListener('drop', (event) => {
      if (mode !== 'browsing') return;
      event.preventDefault();
      event.stopPropagation();
      cancelExpand(row);
      selectedFolder = folder;
      showConfirmation();
    });
    row.querySelector('.lap-capture-chevron').addEventListener('click', (event) => {
      event.stopPropagation();
      if (!hasChildren || mode !== 'browsing') return;
      if (expanded.has(folder.key)) expanded.delete(folder.key);
      else expanded.add(folder.key);
      renderFolders();
    });
    row.addEventListener('click', () => {
      if (mode !== 'browsing') return;
      selectedFolder = folder;
      showConfirmation();
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

  async function loadFolderState(session) {
    const response = await sendMessage({ type: 'lap:get-state' });
    if (!overlay || session !== captureSession) return;
    if (!response?.ok) throw new Error(response?.error || '无法读取 Lap 文件夹');
    folders = response.folders || [];
    recentFolderPaths = response.recentFolders || [];
    if (!folders.length) throw new Error('Lap 中还没有可用文件夹，请先添加资料库文件夹。');

    foldersReady = true;
    resolveIntentState();
  }

  function showConfirmation() {
    mode = 'confirming';
    clearHoverTimer();
    removeDragGhost();
    openPanel();
    overlay.querySelector('.lap-capture-folder-area').hidden = true;
    overlay.querySelector('.lap-capture-status').hidden = true;
    const confirm = overlay.querySelector('.lap-capture-confirm');
    const isInbox = Boolean(selectedFolder?.isInbox);
    const destinationLabel = isInbox ? '待整理区域' : selectedFolder.path;
    confirm.hidden = false;
    confirm.innerHTML = `
      <div class="lap-capture-confirm-heading">
        <span class="lap-capture-confirm-icon">✓</span>
        <div>
          <strong>${isInbox ? '确认进入待整理区域' : '确认保存位置'}</strong>
          <span>${escapeHtml(destinationLabel)}</span>
        </div>
      </div>
      <label class="lap-capture-field">
        <span>标签</span>
        <textarea rows="3" placeholder="每行一个，例如：style:minimal"></textarea>
      </label>
      <div class="lap-capture-check">
        <span>
          <strong>保存后进入待整理列表</strong>
          <small>之后可在 Lap 中选择整理范围，让 AI 生成标签和目标文件夹方案</small>
        </span>
      </div>
      <div class="lap-capture-actions">
        <button class="lap-capture-button is-secondary" data-action="back" type="button">返回选择</button>
        <button class="lap-capture-button is-primary" data-action="save" type="button">${isInbox ? '保存到待整理' : '保存到此文件夹'}</button>
      </div>`;

    confirm.querySelector('[data-action="back"]').addEventListener('click', () => {
      selectedFolder = null;
      showFolderBrowser();
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
          folderPath: selectedFolder.path || null,
          tags,
          metadata: {
            capturedBy: 'lap-drag-capture',
            capturedAt: new Date().toISOString(),
            organizationQueue: 'inbox',
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
          <strong>${response.result?.duplicate ? '素材已存在' : (selectedFolder.isInbox ? '已进入待整理区域' : '已保存到 Lap')}</strong>
          <span>${escapeHtml(response.result?.folder?.path || selectedFolder.path || '待整理区域')}</span>
        </div>`;
      setTimeout(closeOverlay, 1100);
    } catch (error) {
      showSaveError(confirm, saveButton, error instanceof Error ? error.message : String(error));
    }
  }

  async function beginCapture(image, event) {
    closeOverlay();
    currentAsset = createAsset(image);
    lastDragPoint = {
      x: event.clientX || window.innerWidth / 2,
      y: event.clientY || window.innerHeight / 2,
    };
    const session = captureSession;
    createOverlay();
    createDragGhost(image, event.dataTransfer);
    startIntentConfirmation();
    try {
      await loadFolderState(session);
    } catch (error) {
      if (!overlay || session !== captureSession) return;
      folderLoadError = error instanceof Error ? error.message : String(error);
      resolveIntentState();
    }
  }

  document.addEventListener('pointerdown', armImageForNativeDrag, true);
  document.addEventListener('mousedown', armImageForNativeDrag, true);

  document.addEventListener('pointerup', () => {
    if (!overlay) restoreArmedImage();
  }, true);

  document.addEventListener('dragstart', (event) => {
    const image = findDraggedImage(event.target, event.clientX, event.clientY) || armedImage;
    if (!image) return;
    beginCapture(image, event);
  }, true);

  document.addEventListener('dragover', (event) => {
    if (!overlay || !['intent', 'radial'].includes(mode)) return;
    positionCaptureUi(event.clientX, event.clientY);
    if (mode === 'radial') event.preventDefault();
  }, true);

  document.addEventListener('drop', (event) => {
    if (!overlay || mode !== 'radial') return;
    if (
      event.target instanceof Element
      && event.target.closest('.lap-capture-radial-item, .lap-capture-radial-center')
    ) return;
    event.preventDefault();
    closeOverlay();
  }, true);

  document.addEventListener('dragend', () => {
    restoreArmedImage();
    removeDragGhost();
    if (!overlay || !['intent', 'radial'].includes(mode)) return;
    setTimeout(() => {
      if (overlay && ['intent', 'radial'].includes(mode)) closeOverlay();
    }, CLOSE_AFTER_DRAG_MS);
  }, true);

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && overlay) closeOverlay();
  }, true);
})();
