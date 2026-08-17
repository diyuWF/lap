import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';
import { Window } from 'happy-dom';

const contentSource = fs.readFileSync(
  new URL('../../lap-extension/content.js', import.meta.url),
  'utf8',
);

const folders = [
  { id: 1, name: 'lap资源', path: 'D:/Lap/lap资源' },
  { id: 2, name: '1', path: 'D:/Lap/lap资源/1' },
  { id: 3, name: '三渲二', path: 'D:/Lap/lap资源/1/三渲二' },
  { id: 4, name: '写实', path: 'D:/Lap/lap资源/1/写实' },
  { id: 5, name: '风景摄影', path: 'D:/Lap/风景摄影' },
];

function dragEvent(window, type, x, y, dataTransfer) {
  const event = new window.Event(type, { bubbles: true, cancelable: true });
  Object.defineProperties(event, {
    clientX: { value: x },
    clientY: { value: y },
    dataTransfer: { value: dataTransfer },
  });
  return event;
}

async function createContentHarness({ width = 1200, height = 900, folderFixtures = folders } = {}) {
  const window = new Window({ url: 'https://example.test/inspiration' });
  const messages = [];
  Object.defineProperties(window, {
    innerWidth: { configurable: true, value: width },
    innerHeight: { configurable: true, value: height },
  });
  window.document.body.innerHTML =
    '<img id="source" src="https://images.example.test/source.jpg" alt="测试素材">';
  const image = window.document.querySelector('#source');
  image.getBoundingClientRect = () => ({
    x: 900,
    y: 200,
    left: 900,
    top: 200,
    width: 240,
    height: 360,
    right: 1140,
    bottom: 560,
  });
  Object.defineProperties(image, {
    naturalWidth: { configurable: true, value: 1200 },
    naturalHeight: { configurable: true, value: 1800 },
  });

  window.chrome = {
    runtime: {
      lastError: null,
      getURL: (path) => `chrome-extension://lap/${path}`,
      sendMessage(message, callback) {
        messages.push(message);
        if (message.type === 'lap:get-state') {
          queueMicrotask(() =>
            callback({
              ok: true,
              folders: folderFixtures,
              recentFolders: [],
              aiConfigured: false,
            }),
          );
          return;
        }
        if (message.type === 'lap:get-folder-covers') {
          queueMicrotask(() => callback({ ok: true, covers: [] }));
          return;
        }
        if (message.type === 'lap:capture') {
          queueMicrotask(() =>
            callback({
              ok: true,
              result: {
                duplicate: false,
                aiClassificationStarted: false,
                folder: { path: message.payload.folderPath || '' },
              },
            }),
          );
          return;
        }
        queueMicrotask(() => callback({ ok: true }));
      },
    },
  };
  window.eval(contentSource);

  const dataTransfer = {
    effectAllowed: '',
    dropEffect: '',
    setData() {},
    setDragImage() {},
  };
  image.dispatchEvent(dragEvent(window, 'dragstart', 1020, 380, dataTransfer));
  await new Promise((resolve) => setTimeout(resolve, 0));
  window.document.dispatchEvent(dragEvent(window, 'dragover', 800, 380, dataTransfer));
  window.document.dispatchEvent(dragEvent(window, 'dragover', 560, 380, dataTransfer));
  await new Promise((resolve) => setTimeout(resolve, 20));

  return { window, messages, dataTransfer };
}

function radialLabels(window) {
  return [...window.document.querySelectorAll('.lap-capture-radial-item strong')].map((element) =>
    element.textContent.trim(),
  );
}

function radialPoint(window, button) {
  return {
    x: window.innerWidth / 2 + Number(button.dataset.radialX),
    y: window.innerHeight / 2 + Number(button.dataset.radialY),
  };
}

test('AI classification remains visible and silently queues an inbox item', async () => {
  const { window, messages } = await createContentHarness();
  const center = window.document.querySelector('.lap-capture-radial-center');
  assert.ok(center, 'AI center should render');
  assert.equal(center.hidden, false);
  assert.equal(center.disabled, false);
  assert.match(center.querySelector('.lap-capture-ai-wordmark').src, /ai-wordmark\.png$/);
  assert.equal(center.querySelectorAll('.lap-capture-ai-orbit').length, 2);

  center.click();
  assert.equal(window.document.querySelector('.lap-capture-overlay'), null);
  await new Promise((resolve) => setTimeout(resolve, 20));
  const capture = messages.find((message) => message.type === 'lap:capture');
  assert.equal(capture?.payload.workflowStatus, 'inbox');
  assert.equal(capture?.payload.autoClassify, false);
  assert.equal(window.document.querySelector('.lap-capture-toast'), null);
  window.close();
});

test('document-level dwell opens only the hovered folder direct children', async () => {
  const { window, dataTransfer } = await createContentHarness();
  assert.deepEqual(radialLabels(window).sort(), ['lap资源', '创建目录', '风景摄影'].sort());
  assert.equal(radialLabels(window).includes('1'), false);

  const root = window.document.querySelector('[data-path="D:/Lap/lap资源"]');
  window.document.elementsFromPoint = () => [];
  const rootPoint = radialPoint(window, root);
  window.document.dispatchEvent(
    dragEvent(window, 'dragover', rootPoint.x, rootPoint.y, dataTransfer),
  );
  await new Promise((resolve) => setTimeout(resolve, 410));
  assert.deepEqual(radialLabels(window).sort(), ['1', '保存到 lap资源', '创建目录', '返回上级'].sort());
  assert.equal(radialLabels(window).includes('lap资源'), false);

  window.document.dispatchEvent(
    dragEvent(window, 'dragover', rootPoint.x, rootPoint.y, dataTransfer),
  );
  await new Promise((resolve) => setTimeout(resolve, 410));
  assert.deepEqual(radialLabels(window).sort(), ['1', '保存到 lap资源', '创建目录', '返回上级'].sort());

  const child = window.document.querySelector('[data-path="D:/Lap/lap资源/1"]');
  const childPoint = radialPoint(window, child);
  window.document.dispatchEvent(
    dragEvent(window, 'dragover', childPoint.x, childPoint.y, dataTransfer),
  );
  await new Promise((resolve) => setTimeout(resolve, 410));
  assert.deepEqual(radialLabels(window).sort(), ['三渲二', '写实', '保存到 1', '创建目录', '返回上级'].sort());
  window.close();
});

test('radial actions stay completely inside the inner orbit without intersecting', async () => {
  const crowdedFolders = Array.from({ length: 12 }, (_, index) => ({
    id: 100 + index,
    name: `目录 ${index + 1}`,
    path: `D:/Lap/目录 ${index + 1}`,
  }));
  for (const viewport of [{ width: 1200, height: 900 }, { width: 360, height: 640 }]) {
    const { window } = await createContentHarness({ ...viewport, folderFixtures: crowdedFolders });
    const buttons = [...window.document.querySelectorAll('.lap-capture-radial-item')];
    const stageSize = Math.min(840, window.innerWidth - 32, window.innerHeight - 32);
    const innerOrbitRadius = 270 * Math.max(0.3, Math.min(1, stageSize / 840));

    for (const button of buttons) {
      const centerRadius = Math.hypot(Number(button.dataset.radialX), Number(button.dataset.radialY));
      const itemRadius = Number(button.dataset.radialHitRadius);
      assert.ok(
        centerRadius + itemRadius <= innerOrbitRadius - 7,
        `${button.textContent.trim()} should have a visible gap from the orbit stroke`,
      );
    }

    for (let leftIndex = 0; leftIndex < buttons.length; leftIndex += 1) {
      for (let rightIndex = leftIndex + 1; rightIndex < buttons.length; rightIndex += 1) {
        const left = buttons[leftIndex];
        const right = buttons[rightIndex];
        const distance = Math.hypot(
          Number(left.dataset.radialX) - Number(right.dataset.radialX),
          Number(left.dataset.radialY) - Number(right.dataset.radialY),
        );
        const minimumGap = Number(left.dataset.radialHitRadius)
          + Number(right.dataset.radialHitRadius)
          + 7;
        assert.ok(distance >= minimumGap, 'radial actions should not overlap each other');
      }
    }
    assert.ok(radialLabels(window).includes('更多'), 'crowded levels should retain the full browser');
    window.close();
  }
});

test('the current folder action saves directly instead of navigating again', async () => {
  const { window, messages, dataTransfer } = await createContentHarness();
  const root = window.document.querySelector('[data-path="D:/Lap/lap资源"]');
  window.document.elementsFromPoint = () => [root];
  window.document.dispatchEvent(dragEvent(window, 'dragover', 600, 160, dataTransfer));
  await new Promise((resolve) => setTimeout(resolve, 410));

  const saveCurrent = [...window.document.querySelectorAll('.lap-capture-radial-item')]
    .find((button) => button.textContent.includes('保存到 lap资源'));
  assert.ok(saveCurrent);
  saveCurrent.click();
  assert.equal(window.document.querySelector('.lap-capture-overlay'), null);
  await new Promise((resolve) => setTimeout(resolve, 20));
  const capture = messages.find((message) => message.type === 'lap:capture');
  assert.equal(capture?.payload.folderPath, 'D:/Lap/lap资源');
  assert.equal(capture?.payload.workflowStatus, 'selected');
  window.close();
});

test('first install opens Token pairing and updates do not reopen it', async () => {
  const backgroundSource = fs.readFileSync(
    new URL('../../lap-extension/background.js', import.meta.url),
    'utf8',
  );
  let installedListener;
  let opened = 0;
  const stored = {};
  const chrome = {
    runtime: {
      onInstalled: {
        addListener(listener) {
          installedListener = listener;
        },
      },
      onMessage: { addListener() {} },
      openOptionsPage() {
        opened += 1;
        return Promise.resolve();
      },
    },
    storage: {
      local: {
        get: async (defaults) => defaults,
        set: async (value) => Object.assign(stored, value),
      },
    },
  };
  vm.runInNewContext(backgroundSource, { chrome, Headers, fetch, console, URL });

  assert.equal(typeof installedListener, 'function');
  installedListener({ reason: 'update' });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(opened, 0);
  installedListener({ reason: 'install' });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(opened, 1);
  assert.equal(stored.pairingOnboarding, true);
});

test('first-install options reveal and focus the Token field', async () => {
  const optionsHtml = fs.readFileSync(
    new URL('../../lap-extension/options.html', import.meta.url),
    'utf8',
  );
  const optionsSource = fs.readFileSync(
    new URL('../../lap-extension/options.js', import.meta.url),
    'utf8',
  );
  const window = new Window({ url: 'chrome-extension://lap/options.html' });
  const stored = {};
  window.document.write(optionsHtml);
  window.chrome = {
    storage: {
      local: {
        get: async (defaults) => ({ ...defaults, pairingOnboarding: true }),
        set: async (value) => Object.assign(stored, value),
      },
    },
  };
  window.fetch = async () => ({
    ok: false,
    status: 0,
    json: async () => ({}),
  });
  window.eval(optionsSource);
  await new Promise((resolve) => setTimeout(resolve, 50));

  const advanced = window.document.querySelector('#advancedSettings');
  const token = window.document.querySelector('#token');
  assert.equal(advanced.open, true);
  assert.equal(window.document.activeElement, token);
  assert.equal(stored.pairingOnboarding, false);
  window.close();
});
