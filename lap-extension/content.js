(() => {
  const HOVER_EXPAND_MS = 360;
  const CLOSE_AFTER_DRAG_MS = 160;
  const DRAG_DISTANCE_VIEWPORT_RATIO = 1 / 3;
  const RADIAL_FOLDER_LIMIT = 8;
  const RADIAL_NAVIGATION_RELEASE_PX = 28;
  const folderCoverViews = new WeakMap();

  let overlay = null;
  let mode = 'idle';
  let currentAsset = null;
  let folders = [];
  let folderCoverUrls = new Map();
  let recentFolderPaths = [];
  let expanded = new Set();
  let hoverTimer = null;
  let radialDwellTimer = null;
  let radialDwellTarget = null;
  let radialPointer = null;
  let radialNavigationLock = null;
  let thresholdReached = false;
  let foldersReady = false;
  let aiConfigured = false;
  let folderLoadError = null;
  let dragLastPoint = null;
  let dragTravelDistance = 0;
  let selectedFolder = null;
  let dragGhost = null;
  let armedImage = null;
  let armedDraggableValue = null;
  let captureSession = 0;
  let radialParentKey = null;
  let keepRadialAfterDragEnd = false;

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

  function clearRadialDwell() {
    if (radialDwellTimer) clearTimeout(radialDwellTimer);
    radialDwellTimer = null;
    radialDwellTarget?.classList.remove('is-target');
    radialDwellTarget = null;
  }

  function activateRadialNavigation(button) {
    if (!button || mode !== 'radial') return;
    const action = button.dataset.radialNavigate;
    if (action === 'folder') showRadialLevel(button.dataset.radialTarget || null);
    if (action === 'back') showRadialLevel(button.dataset.radialTarget || null);
  }

  function scheduleRadialDwell(button) {
    if (!button || mode !== 'radial' || radialNavigationLock) {
      clearRadialDwell();
      return;
    }
    if (radialDwellTarget === button && radialDwellTimer) return;
    clearRadialDwell();
    radialDwellTarget = button;
    button.classList.add('is-target');
    radialDwellTimer = setTimeout(() => {
      if (mode !== 'radial' || radialDwellTarget !== button || !button.isConnected) return;
      radialDwellTimer = null;
      radialDwellTarget = null;
      activateRadialNavigation(button);
    }, HOVER_EXPAND_MS);
  }

  function updateRadialDwell(clientX, clientY) {
    if (!overlay || mode !== 'radial' || typeof document.elementsFromPoint !== 'function') {
      clearRadialDwell();
      return;
    }
    radialPointer = { x: clientX, y: clientY };
    if (radialNavigationLock) {
      const moved = Math.hypot(
        clientX - radialNavigationLock.x,
        clientY - radialNavigationLock.y,
      );
      if (moved < RADIAL_NAVIGATION_RELEASE_PX) {
        clearRadialDwell();
        return;
      }
      radialNavigationLock = null;
    }
    const button = document
      .elementsFromPoint(clientX, clientY)
      .map((element) => element.closest?.('[data-radial-navigate]'))
      .find((element) => element instanceof HTMLButtonElement && overlay.contains(element));
    scheduleRadialDwell(button || null);
  }

  function removeDragGhost() {
    dragGhost?.remove();
    dragGhost = null;
  }

  function closeOverlay() {
    captureSession += 1;
    clearHoverTimer();
    clearRadialDwell();
    removeDragGhost();
    overlay?.remove();
    overlay = null;
    mode = 'idle';
    currentAsset = null;
    selectedFolder = null;
    folders = [];
    folderCoverUrls = new Map();
    recentFolderPaths = [];
    expanded = new Set();
    thresholdReached = false;
    foldersReady = false;
    aiConfigured = false;
    folderLoadError = null;
    dragLastPoint = null;
    dragTravelDistance = 0;
    radialParentKey = null;
    radialPointer = null;
    radialNavigationLock = null;
    keepRadialAfterDragEnd = false;
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
    overlay.className = 'lap-capture-overlay is-radial-open';
    const extensionIconUrl = chrome.runtime.getURL('icon.png');
    const aiClassifyIconUrl = chrome.runtime.getURL('ai-classify.png');
    const radialGaugeUrl = chrome.runtime.getURL('radial-gauge.png');
    overlay.innerHTML = `
      <div class="lap-capture-radial" role="menu" aria-label="保存到文件夹">
        <img class="lap-capture-radial-gauge" src="${escapeHtml(radialGaugeUrl)}" alt="" />
        <button class="lap-capture-radial-center" type="button" role="menuitem">
          <img src="${escapeHtml(aiClassifyIconUrl)}" alt="" />
          <span>AI 分类</span>
        </button>
        <div class="lap-capture-radial-items"></div>
      </div>
      <section class="lap-capture-panel" role="dialog" aria-label="选择保存目录" hidden>
        <header class="lap-capture-header">
          <div class="lap-capture-brand">
            <img class="lap-capture-logo" src="${escapeHtml(extensionIconUrl)}" alt="" />
            <div>
              <strong>选择保存目录</strong>
              <span>松手或点击后立即保存</span>
            </div>
          </div>
          <button class="lap-capture-close" type="button" aria-label="关闭">关闭</button>
        </header>
        <div class="lap-capture-body">
          <aside class="lap-capture-preview">
            <div class="lap-capture-preview-frame">
              <img class="lap-capture-preview-image" alt="待保存素材预览" />
              <div class="lap-capture-preview-fallback" hidden>
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
            <form class="lap-capture-create-folder" hidden>
              <div class="lap-capture-create-heading">
                <strong>创建目录</strong>
                <span>新目录会建立在已有 Lap 目录下，完成后立即保存当前图片。</span>
              </div>
              <label class="lap-capture-field">
                <span>上级目录</span>
                <select name="parentPath"></select>
              </label>
              <label class="lap-capture-field">
                <span>目录名称</span>
                <input name="folderName" maxlength="100" autocomplete="off" placeholder="例如：中国风" required />
              </label>
              <div class="lap-capture-inline-error" hidden></div>
              <div class="lap-capture-actions">
                <button class="lap-capture-button is-secondary" data-action="cancel-create" type="button">取消</button>
                <button class="lap-capture-button is-primary" data-action="create" type="submit">创建并保存</button>
              </div>
            </form>
          </main>
        </div>
      </section>`;

    overlay.querySelector('.lap-capture-close').addEventListener('click', closeOverlay);
    overlay.querySelector('[data-action="cancel-create"]').addEventListener('click', showFolderBrowser);
    overlay.querySelector('.lap-capture-create-folder').addEventListener('submit', createFolderAndSave);
    document.documentElement.appendChild(overlay);

    const preview = overlay.querySelector('.lap-capture-preview-image');
    const previewFallback = overlay.querySelector('.lap-capture-preview-fallback');
    preview.addEventListener(
      'error',
      () => {
        preview.hidden = true;
        previewFallback.hidden = false;
      },
      { once: true },
    );
    preview.src = currentAsset.sourceUrl;
    const dimensions =
      currentAsset.naturalWidth && currentAsset.naturalHeight
        ? `${currentAsset.naturalWidth} × ${currentAsset.naturalHeight}`
        : '尺寸未知';
    overlay.querySelector('.lap-capture-preview-meta').innerHTML = `
      <strong>${escapeHtml(sourceFileName(currentAsset.sourceUrl))}</strong>
      <span>${escapeHtml(dimensions)}</span>
      <span>${escapeHtml(location.hostname)}</span>`;
  }

  function hideIntentUi() {
    if (!overlay) return;
    overlay.classList.remove('is-radial-open');
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
        parentKey: null,
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
      if (parent) {
        folder.parentKey = parent.key;
        parent.children.push(folder);
      } else roots.push(folder);
    }

    const sortNodes = (nodes) => {
      nodes.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN', { numeric: true }));
      nodes.forEach((node) => sortNodes(node.children));
    };
    sortNodes(roots);
    return { roots, byKey };
  }

  function radialFolderCandidates(parentKey = radialParentKey) {
    const { roots, byKey } = buildFolderTree();
    const parent = parentKey ? byKey.get(normalizePath(parentKey).toLowerCase()) || null : null;
    const siblings = [...(parent ? parent.children : roots)];
    const recentOrder = new Map(
      recentFolderPaths.map((path, index) => [normalizePath(path).toLowerCase(), index]),
    );
    siblings.sort((left, right) => {
      const leftRecent = recentOrder.get(left.key.toLowerCase()) ?? Number.MAX_SAFE_INTEGER;
      const rightRecent = recentOrder.get(right.key.toLowerCase()) ?? Number.MAX_SAFE_INTEGER;
      return leftRecent - rightRecent || left.name.localeCompare(right.name, 'zh-CN', { numeric: true });
    });
    return {
      parent,
      siblings,
      candidates: siblings.slice(0, RADIAL_FOLDER_LIMIT),
    };
  }

  function radialOrbitPosition(index, total) {
    const stageSize = Math.min(820, window.innerWidth - 48, window.innerHeight - 80);
    const scale = Math.max(0.42, Math.min(1, stageSize / 820));
    const radius = (total <= 4 ? 248 : total <= 7 ? 266 : 278) * scale;
    const startAngle = total === 1 ? 90 : -90;
    const angle = startAngle + (360 / Math.max(1, total)) * index;
    const radians = (angle * Math.PI) / 180;
    return {
      x: Math.cos(radians) * radius,
      y: Math.sin(radians) * radius,
      angle,
    };
  }

  function applyRadialFolderCover(button, folder) {
    const cover = button.querySelector('.lap-capture-radial-cover');
    const icon = button.querySelector('.lap-capture-radial-folder-icon');
    const dataUrl = folderCoverUrls.get(Number(folder.id));
    if (!cover || !icon || !dataUrl) return;
    let view = folderCoverViews.get(cover);
    if (!view) {
      const shadow = cover.attachShadow({ mode: 'closed' });
      const style = document.createElement('style');
      style.textContent =
        ':host{display:block;width:100%;height:100%}img{display:block;width:100%;height:100%;object-fit:cover;object-position:center}';
      const image = document.createElement('img');
      image.alt = '';
      shadow.append(style, image);
      view = { image };
      folderCoverViews.set(cover, view);
    }
    view.image.onload = () => {
      icon.hidden = true;
      button.classList.add('has-cover');
    };
    view.image.onerror = () => {
      icon.hidden = false;
      button.classList.remove('has-cover');
    };
    view.image.src = dataUrl;
    if (view.image.complete && view.image.naturalWidth > 0) view.image.onload();
  }

  async function hydrateRadialFolderCovers(candidates, session) {
    const folderIds = candidates
      .map((folder) => Number(folder.id))
      .filter((value) => Number.isInteger(value) && value > 0 && !folderCoverUrls.has(value));
    if (!folderIds.length) return;
    try {
      const response = await sendMessage({ type: 'lap:get-folder-covers', folderIds });
      if (session !== captureSession || !response?.ok || !overlay) return;
      for (const cover of response.covers || []) {
        const folderId = Number(cover.folderId);
        if (!Number.isInteger(folderId) || !cover.dataUrl) continue;
        folderCoverUrls.set(folderId, cover.dataUrl);
        const folder = candidates.find((item) => Number(item.id) === folderId);
        const button = overlay.querySelector(`.lap-capture-radial-item[data-folder-id="${folderId}"]`);
        if (folder && button) applyRadialFolderCover(button, folder);
      }
    } catch {
      // Covers are optional visual metadata; empty folders keep the folder icon.
    }
  }

  function showRadialLevel(parentKey) {
    radialParentKey = parentKey || null;
    radialNavigationLock = radialPointer ? { ...radialPointer } : null;
    clearHoverTimer();
    clearRadialDwell();
    showRadialMenu();
  }

  function createRadialFolder(folder, index, total, options = {}) {
    const saveDirectly = Boolean(options.saveDirectly);
    const navigates = folder.children.length > 0 && !saveDirectly;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = `lap-capture-radial-item${navigates ? ' has-children' : ''}${saveDirectly ? ' is-current-folder' : ''}`;
    button.dataset.path = folder.path;
    button.dataset.folderId = String(folder.id || '');
    if (navigates) {
      button.dataset.radialNavigate = 'folder';
      button.dataset.radialTarget = folder.key;
    }
    button.setAttribute('role', 'menuitem');
    button.title = folder.path;
    button.innerHTML = `
      <span class="lap-capture-radial-visual">
        <span class="lap-capture-radial-cover" aria-hidden="true"></span>
        <img class="lap-capture-radial-folder-icon" src="${escapeHtml(chrome.runtime.getURL('folder.svg'))}" alt="" />
      </span>
      <strong>${escapeHtml(saveDirectly ? `保存到 ${folder.name}` : folder.name)}</strong>`;
    applyRadialFolderCover(button, folder);

    const position = radialOrbitPosition(index, total);
    button.style.setProperty('--lap-radial-x', `${position.x}px`);
    button.style.setProperty('--lap-radial-y', `${position.y}px`);
    button.style.setProperty('--lap-radial-order', index);

    button.addEventListener('dragover', (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    });
    if (navigates) {
      button.addEventListener('mouseenter', () => scheduleRadialDwell(button));
      button.addEventListener('mouseleave', () => {
        if (radialDwellTarget === button) clearRadialDwell();
      });
    }
    button.addEventListener('drop', (event) => {
      if (mode !== 'radial') return;
      event.preventDefault();
      event.stopPropagation();
      clearHoverTimer();
      if (navigates) {
        keepRadialAfterDragEnd = true;
        showRadialLevel(folder.key);
        return;
      }
      selectedFolder = folder;
      saveSelectedAsset();
    });
    button.addEventListener('click', () => {
      if (mode !== 'radial') return;
      if (navigates) {
        showRadialLevel(folder.key);
        return;
      }
      selectedFolder = folder;
      saveSelectedAsset();
    });
    return button;
  }

  function showFolderBrowser() {
    if (!overlay) return;
    mode = 'browsing';
    openPanel();
    setPanelBrand('选择保存目录', '点击目录后立即保存');
    overlay.querySelector('.lap-capture-status').hidden = true;
    overlay.querySelector('.lap-capture-create-folder').hidden = true;
    overlay.querySelector('.lap-capture-folder-area').hidden = false;
    renderFolders();
  }

  function showRadialMenu() {
    if (!overlay || !foldersReady || folderLoadError) return;
    clearRadialDwell();
    mode = 'radial';
    const radial = overlay.querySelector('.lap-capture-radial');
    const items = overlay.querySelector('.lap-capture-radial-items');
    const level = radialFolderCandidates();
    const candidates = level.candidates;
    const showBack = Boolean(level.parent);
    const showCurrentFolder = Boolean(level.parent);
    const showMore = level.siblings.length > candidates.length;
    const showCreate = folders.length > 0;
    const total = candidates.length
      + (showBack ? 1 : 0)
      + (showCurrentFolder ? 1 : 0)
      + (showCreate ? 1 : 0)
      + (showMore ? 1 : 0);
    const hasAiTarget = true;
    const session = captureSession;

    overlay.classList.add('is-radial-open');
    radial.hidden = false;
    radial.classList.toggle('has-ai-target', hasAiTarget);
    items.textContent = '';
    const inboxTarget = radial.querySelector('.lap-capture-radial-center');
    const selectAiClassification = () => {
      selectedFolder = {
        id: null,
        name: 'AI 分类',
        path: '',
        isInbox: true,
      };
      saveSelectedAsset();
    };
    inboxTarget.hidden = false;
    inboxTarget.disabled = false;
    inboxTarget.classList.remove('is-loading');
    inboxTarget.title = aiConfigured
      ? '静默保存到 AI 分类，稍后在 Lap 中一键整理'
      : '静默保存到 AI 分类；配置 AI 服务后可一键整理';
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
      selectAiClassification();
    };
    inboxTarget.onclick = selectAiClassification;
    let nextIndex = 0;
    if (showCurrentFolder) {
      items.appendChild(createRadialFolder(level.parent, nextIndex, total, { saveDirectly: true }));
      nextIndex += 1;
    }
    if (showBack) {
      const back = document.createElement('button');
      back.type = 'button';
      back.className = 'lap-capture-radial-item is-back';
      back.dataset.radialNavigate = 'back';
      back.dataset.radialTarget = level.parent.parentKey || '';
      back.setAttribute('role', 'menuitem');
      back.innerHTML = `<span class="lap-capture-radial-visual"><img class="lap-capture-radial-action-icon" src="${escapeHtml(chrome.runtime.getURL('back.svg'))}" alt="" /></span><strong>返回上级</strong>`;
      const backPosition = radialOrbitPosition(nextIndex, total);
      back.style.setProperty('--lap-radial-x', `${backPosition.x}px`);
      back.style.setProperty('--lap-radial-y', `${backPosition.y}px`);
      back.style.setProperty('--lap-radial-order', nextIndex);
      const goBack = () => showRadialLevel(level.parent.parentKey);
      back.addEventListener('dragover', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
      });
      back.addEventListener('mouseenter', () => scheduleRadialDwell(back));
      back.addEventListener('mouseleave', () => {
        if (radialDwellTarget === back) clearRadialDwell();
      });
      back.addEventListener('drop', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        event.stopPropagation();
        keepRadialAfterDragEnd = true;
        goBack();
      });
      back.addEventListener('click', goBack);
      items.appendChild(back);
      nextIndex += 1;
    }

    candidates.forEach((folder) => {
      items.appendChild(createRadialFolder(folder, nextIndex, total));
      nextIndex += 1;
    });
    void hydrateRadialFolderCovers(
      showCurrentFolder ? [level.parent, ...candidates] : candidates,
      session,
    );

    if (showCreate) {
      const createFolder = document.createElement('button');
      createFolder.type = 'button';
      createFolder.className = 'lap-capture-radial-item is-create';
      createFolder.setAttribute('role', 'menuitem');
      createFolder.innerHTML = `<span class="lap-capture-radial-visual"><img class="lap-capture-radial-action-icon" src="${escapeHtml(chrome.runtime.getURL('plus.svg'))}" alt="" /></span><strong>创建目录</strong>`;
      const createIndex = nextIndex;
      const createPosition = radialOrbitPosition(createIndex, total);
      createFolder.style.setProperty('--lap-radial-x', `${createPosition.x}px`);
      createFolder.style.setProperty('--lap-radial-y', `${createPosition.y}px`);
      createFolder.style.setProperty('--lap-radial-order', createIndex);
      createFolder.addEventListener('dragenter', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        createFolder.classList.add('is-target');
      });
      createFolder.addEventListener('dragover', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
      });
      createFolder.addEventListener('dragleave', () => createFolder.classList.remove('is-target'));
      createFolder.addEventListener('drop', (event) => {
        if (mode !== 'radial') return;
        event.preventDefault();
        event.stopPropagation();
        removeDragGhost();
        showCreateFolderPanel();
      });
      createFolder.addEventListener('click', showCreateFolderPanel);
      items.appendChild(createFolder);
      nextIndex += 1;
    }

    if (showMore) {
      const more = document.createElement('button');
      more.type = 'button';
      more.className = 'lap-capture-radial-item is-more';
      more.setAttribute('role', 'menuitem');
      more.innerHTML = `<span class="lap-capture-radial-visual"><img class="lap-capture-radial-action-icon" src="${escapeHtml(chrome.runtime.getURL('more.svg'))}" alt="" /></span><strong>更多</strong>`;
      const moreIndex = nextIndex;
      const morePosition = radialOrbitPosition(moreIndex, total);
      more.style.setProperty('--lap-radial-x', `${morePosition.x}px`);
      more.style.setProperty('--lap-radial-y', `${morePosition.y}px`);
      more.style.setProperty('--lap-radial-order', moreIndex);
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
  }

  function resolveIntentState() {
    if (!overlay || !thresholdReached || mode !== 'intent') return;
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

  function getDragIntentDistance() {
    return Math.max(1, window.innerWidth * DRAG_DISTANCE_VIEWPORT_RATIO);
  }

  function updateDragTravel(clientX, clientY) {
    if (!['tracking', 'intent'].includes(mode)) return;
    if (!Number.isFinite(clientX) || !Number.isFinite(clientY) || (clientX === 0 && clientY === 0)) return;

    const point = { x: clientX, y: clientY };
    if (dragLastPoint) {
      dragTravelDistance += Math.hypot(point.x - dragLastPoint.x, point.y - dragLastPoint.y);
    }
    dragLastPoint = point;

    if (dragTravelDistance >= getDragIntentDistance() && !thresholdReached) {
      thresholdReached = true;
      if (!overlay) createOverlay();
      resolveIntentState();
    }
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
      <span class="lap-capture-chevron ${hasChildren ? '' : 'is-empty'}">${hasChildren ? (isExpanded ? '收起' : '展开') : ''}</span>
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
      saveSelectedAsset();
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
      saveSelectedAsset();
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
    if (session !== captureSession) return;
    if (!response?.ok) throw new Error(response?.error || '无法读取 Lap 文件夹');
    folders = response.folders || [];
    recentFolderPaths = response.recentFolders || [];
    aiConfigured = Boolean(response.aiConfigured);
    if (!folders.length && !aiConfigured) {
      throw new Error('Lap 中还没有可用文件夹，请先添加资料库文件夹或配置 AI 服务。');
    }

    foldersReady = true;
    resolveIntentState();
  }

  function setPanelBrand(title, subtitle) {
    const brand = overlay?.querySelector('.lap-capture-brand div');
    if (!brand) return;
    brand.querySelector('strong').textContent = title;
    brand.querySelector('span').textContent = subtitle;
  }

  function showCreateFolderPanel() {
    if (!overlay || !folders.length) return;
    mode = 'creating-folder';
    clearHoverTimer();
    removeDragGhost();
    openPanel();
    setPanelBrand('创建目录', '创建成功后立即保存当前图片');

    overlay.querySelector('.lap-capture-status').hidden = true;
    overlay.querySelector('.lap-capture-folder-area').hidden = true;
    const form = overlay.querySelector('.lap-capture-create-folder');
    const parentSelect = form.querySelector('select[name="parentPath"]');
    const nameInput = form.querySelector('input[name="folderName"]');
    const error = form.querySelector('.lap-capture-inline-error');
    const submitButton = form.querySelector('[data-action="create"]');
    const radialParent = radialParentKey
      ? folders.find((folder) => normalizePath(folder.path) === normalizePath(radialParentKey))
      : null;
    const preferredParent = radialParent?.path || recentFolderPaths.find((path) =>
      folders.some((folder) => normalizePath(folder.path) === normalizePath(path)),
    );

    parentSelect.textContent = '';
    [...folders]
      .sort((left, right) => left.path.localeCompare(right.path, 'zh-CN', { numeric: true }))
      .forEach((folder) => {
        const option = document.createElement('option');
        option.value = folder.path;
        option.textContent = `${folder.name} — ${folder.path}`;
        option.selected = normalizePath(folder.path) === normalizePath(preferredParent);
        parentSelect.appendChild(option);
      });
    nameInput.value = '';
    error.hidden = true;
    error.textContent = '';
    submitButton.disabled = false;
    submitButton.textContent = '创建并保存';
    form.hidden = false;
    requestAnimationFrame(() => nameInput.focus());
  }

  function showCreateError(message) {
    const form = overlay?.querySelector('.lap-capture-create-folder');
    if (!form) return;
    mode = 'creating-folder';
    const error = form.querySelector('.lap-capture-inline-error');
    const submitButton = form.querySelector('[data-action="create"]');
    error.textContent = message || '创建目录失败';
    error.hidden = false;
    submitButton.disabled = false;
    submitButton.textContent = '创建并保存';
  }

  async function createFolderAndSave(event) {
    event.preventDefault();
    if (!overlay || !currentAsset || mode !== 'creating-folder') return;
    const form = event.currentTarget;
    const parentPath = form.elements.parentPath.value.trim();
    const name = form.elements.folderName.value.trim();
    const submitButton = form.querySelector('[data-action="create"]');
    const error = form.querySelector('.lap-capture-inline-error');
    if (!parentPath || !name) {
      showCreateError(!parentPath ? '请选择上级目录' : '请输入目录名称');
      return;
    }

    mode = 'creating-folder-pending';
    error.hidden = true;
    submitButton.disabled = true;
    submitButton.textContent = '正在创建…';
    try {
      const response = await sendMessage({
        type: 'lap:create-folder',
        parentPath,
        name,
      });
      if (!response?.ok || !response.folder) {
        showCreateError(response?.error || '创建目录失败');
        return;
      }
      folders.push(response.folder);
      selectedFolder = response.folder;
      await saveSelectedAsset();
    } catch (errorValue) {
      showCreateError(errorValue instanceof Error ? errorValue.message : String(errorValue));
    }
  }

  function saveSelectedAsset() {
    if (!currentAsset || !selectedFolder || ['saving', 'complete'].includes(mode)) return;
    mode = 'saving';
    clearHoverTimer();
    clearRadialDwell();
    removeDragGhost();
    const isInbox = Boolean(selectedFolder.isInbox);
    const payload = {
      sourceUrl: currentAsset.sourceUrl,
      pageUrl: location.href,
      pageTitle: document.title,
      siteName: location.hostname,
      altText: currentAsset.altText,
      folderPath: selectedFolder.path || null,
      workflowStatus: isInbox ? 'inbox' : 'selected',
      autoClassify: false,
      tags: [],
      metadata: {
        capturedBy: 'lap-drag-capture',
        capturedAt: new Date().toISOString(),
        organizationQueue: isInbox ? 'inbox' : 'organized',
        naturalWidth: currentAsset.naturalWidth,
        naturalHeight: currentAsset.naturalHeight,
        displayWidth: currentAsset.displayWidth,
        displayHeight: currentAsset.displayHeight,
      },
    };

    closeOverlay();
    void sendMessage({ type: 'lap:capture', payload })
      .then((response) => {
        if (!response?.ok) {
          console.warn('Lap background capture failed:', response?.error || 'Unknown capture error');
        }
      })
      .catch((error) => {
        console.warn(
          'Lap background capture failed:',
          error instanceof Error ? error.message : String(error),
        );
      });
  }

  async function beginCapture(image, event) {
    closeOverlay();
    currentAsset = createAsset(image);
    dragLastPoint = {
      x: event.clientX || window.innerWidth / 2,
      y: event.clientY || window.innerHeight / 2,
    };
    dragTravelDistance = 0;
    mode = 'tracking';
    const session = captureSession;
    createDragGhost(image, event.dataTransfer);
    try {
      await loadFolderState(session);
    } catch (error) {
      if (session !== captureSession) return;
      folderLoadError = error instanceof Error ? error.message : String(error);
      resolveIntentState();
    }
  }

  document.addEventListener('pointerdown', armImageForNativeDrag, true);
  document.addEventListener('mousedown', armImageForNativeDrag, true);

  document.addEventListener(
    'pointerup',
    () => {
      if (!overlay) restoreArmedImage();
    },
    true,
  );

  document.addEventListener(
    'dragstart',
    (event) => {
      const image = findDraggedImage(event.target, event.clientX, event.clientY) || armedImage;
      if (!image) return;
      beginCapture(image, event);
    },
    true,
  );

  document.addEventListener(
    'dragover',
    (event) => {
      if (!['tracking', 'intent', 'radial'].includes(mode)) return;
      if (mode === 'tracking' || mode === 'intent') updateDragTravel(event.clientX, event.clientY);
      if (mode === 'radial') {
        event.preventDefault();
        updateRadialDwell(event.clientX, event.clientY);
      }
    },
    true,
  );

  document.addEventListener(
    'drop',
    (event) => {
      if (!overlay || mode !== 'radial') return;
      if (
        event.target instanceof Element &&
        event.target.closest('.lap-capture-radial-item, .lap-capture-radial-center')
      )
        return;
      event.preventDefault();
      closeOverlay();
    },
    true,
  );

  document.addEventListener(
    'dragend',
    () => {
      restoreArmedImage();
      removeDragGhost();
      if (!['tracking', 'intent', 'radial'].includes(mode)) return;
      if (mode === 'radial' && keepRadialAfterDragEnd) {
        keepRadialAfterDragEnd = false;
        return;
      }
      setTimeout(() => {
        if (['tracking', 'intent', 'radial'].includes(mode)) closeOverlay();
      }, CLOSE_AFTER_DRAG_MS);
    },
    true,
  );

  document.addEventListener(
    'keydown',
    (event) => {
      if (event.key === 'Escape' && overlay) closeOverlay();
    },
    true,
  );
})();
