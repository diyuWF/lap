(() => {
  let dragging = false;
  let currentImage = null;
  let overlay = null;
  let folders = [];
  let expanded = new Set();
  let hoverTimer = null;

  const API = 'http://127.0.0.1:47821';

  function getConfig() {
    return new Promise((resolve) => {
      chrome.storage.local.get({ apiUrl: API, token: '' }, resolve);
    });
  }

  function isImageElement(target) {
    return target instanceof HTMLImageElement && target.src;
  }

  function createOverlay() {
    overlay = document.createElement('div');
    overlay.className = 'lap-picker-overlay';
    overlay.innerHTML = `
      <div class="lap-picker-panel">
        <div class="lap-picker-header">
          <strong>保存到 Lap</strong>
          <span class="lap-picker-tip">拖入文件夹，停留自动展开</span>
        </div>
        <div class="lap-picker-tree"></div>
      </div>`;
    document.body.appendChild(overlay);
    return overlay;
  }

  function childrenOf(folderPath) {
    return folders.filter((folder) => {
      const parent = folder.path.split(/[\\/]/).slice(0, -1).join('/');
      return parent === folderPath;
    });
  }

  function renderTree() {
    const tree = overlay.querySelector('.lap-picker-tree');
    tree.innerHTML = '';
    const roots = folders.filter((folder) => {
      return !folders.some((parent) => {
        return folder.path.startsWith(parent.path + '/');
      });
    });

    function renderNode(folder, depth) {
      const row = document.createElement('div');
      row.className = 'lap-folder-row';
      row.style.paddingLeft = `${depth * 18 + 12}px`;
      row.dataset.path = folder.path;
      row.innerHTML = `📁 ${folder.name}`;

      const children = childrenOf(folder.path);
      if (children.length) {
        row.innerHTML = `${expanded.has(folder.path) ? '📂' : '📁'} ${folder.name}`;
      }

      row.addEventListener('dragover', (event) => {
        event.preventDefault();
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => {
          if (children.length) {
            expanded.add(folder.path);
            renderTree();
          }
        }, 500);
      });

      row.addEventListener('drop', async (event) => {
        event.preventDefault();
        await saveToFolder(folder.path);
      });

      tree.appendChild(row);

      if (expanded.has(folder.path)) {
        children.forEach((child) => renderNode(child, depth + 1));
      }
    }

    roots.forEach((folder) => renderNode(folder, 0));
  }

  async function loadFolders() {
    const config = await getConfig();
    const response = await fetch(`${config.apiUrl || API}/folders`, {
      headers: { 'X-Lap-Token': config.token }
    });
    const data = await response.json();
    folders = data.folders || [];
    renderTree();
  }

  async function saveToFolder(folderPath) {
    const config = await getConfig();
    await fetch(`${config.apiUrl || API}/capture`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Lap-Token': config.token
      },
      body: JSON.stringify({
        sourceUrl: currentImage,
        pageUrl: location.href,
        pageTitle: document.title,
        siteName: location.hostname,
        folderPath,
        metadata: {
          capturedBy: 'lap-drag-capture'
        },
        tags: []
      })
    });
    closeOverlay();
  }

  function closeOverlay() {
    if (overlay) overlay.remove();
    overlay = null;
    currentImage = null;
  }

  document.addEventListener('dragstart', async (event) => {
    const image = isImageElement(event.target);
    if (!image) return;
    dragging = true;
    currentImage = image.currentSrc || image.src;
    createOverlay();
    await loadFolders();
  }, true);

  document.addEventListener('dragend', () => {
    dragging = false;
    if (overlay) {
      setTimeout(() => {
        if (overlay) closeOverlay();
      }, 1500);
    }
  }, true);
})();
