<template>
  <div class="reference-board" tabindex="-1" @pointerdown.capture="focusBoard">
    <div class="reference-board-toolbar" role="toolbar" :aria-label="$t('reference_board.title')">
      <button class="reference-board-move" type="button" :title="$t('reference_board.move_window')" @pointerdown="startWindowDrag">⠿</button>
      <div class="reference-board-tools">
        <button
          type="button"
          :class="{ 'is-active': alwaysOnTop }"
          :title="$t('reference_board.always_on_top')"
          @click="toggleAlwaysOnTop"
        >
          <component :is="alwaysOnTop ? IconPin : IconUnPin" />
        </button>
        <span class="reference-board-divider"></span>
        <button
          type="button"
          :title="$t('reference_board.zoom_out')"
          @click="zoomBy(0.84)"
        >
          <IconZoomOut />
        </button>
        <button
          type="button"
          :title="$t('reference_board.fit_all')"
          @click="fitAll"
        >
          <IconZoomFit />
        </button>
        <button
          type="button"
          :title="$t('reference_board.actual_size')"
          @click="showActualSize"
        >
          <IconZoomActual />
        </button>
        <button
          type="button"
          :title="$t('reference_board.zoom_in')"
          @click="zoomBy(1.19)"
        >
          <IconZoomIn />
        </button>
        <span class="reference-board-divider"></span>
        <button
          type="button"
          :title="$t('reference_board.minimize')"
          @click="minimizeWindow"
        >
          <IconWinMinus />
        </button>
        <button
          type="button"
          :title="$t('reference_board.close')"
          @click="closeWindow"
        >
          <IconClose />
        </button>
      </div>
    </div>

    <main
      ref="viewportRef"
      class="reference-board-viewport"
      :class="{ 'is-panning': interaction?.kind === 'pan' }"
      @pointerdown="startPan"
      @wheel.prevent="handleWheel"
      @dblclick.self="fitAll"
      @dragover.prevent
      @drop.prevent="handleDomDrop"
    >
      <div v-if="items.length === 0" class="reference-board-empty">
        <IconPhotoAll />
        <strong>{{ $t("reference_board.empty_title") }}</strong>
        <span>{{ $t("reference_board.empty_hint") }}</span>
      </div>

      <div class="reference-board-world" :style="worldStyle">
        <article
          v-for="item in items"
          :key="item.id"
          class="reference-board-item"
          :class="{ 'is-selected': item.id === selectedId }"
          :style="itemStyle(item)"
          @pointerdown.stop="startMoveItem($event, item)"
        >
          <img
            :src="assetSource(item)"
            :alt="item.name"
            draggable="false"
            @load="finishImageSize($event, item)"
            @error="item.failed = true"
          />
          <div v-if="item.failed" class="reference-board-image-error">
            <IconError />
            <span>{{ $t("reference_board.load_failed") }}</span>
          </div>
          <button
            v-if="item.id === selectedId"
            type="button"
            :title="$t('reference_board.remove')"
            @pointerdown.stop
            @click.stop="removeItem(item.id)"
          >
            <IconClose />
          </button>
        </article>
      </div>
    </main>

    <footer class="reference-board-status" aria-live="polite">
      <span v-if="persistError" class="reference-board-save-error">{{ $t("reference_board.save_failed") }}</span>
      <span>{{ $t("reference_board.controls_hint") }}</span>
      <strong>{{ Math.round(camera.scale * 100) }}%</strong>
    </footer>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { emit } from "@tauri-apps/api/event";
import { BOARD_STORAGE_KEY, readBoardState, writeBoardState } from "@/common/reference-board-state.mjs";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getAssetSrc, isTauriRuntime } from "@/common/utils";
import {
  IconClose,
  IconError,
  IconPhotoAll,
  IconPin,
  IconUnPin,
  IconWinMinus,
  IconZoomActual,
  IconZoomFit,
  IconZoomIn,
  IconZoomOut,
} from "@/common/icons";
import lapLogo from "@/assets/images/icon.png";

type IncomingAsset = {
  id?: number;
  file_path?: string;
  path?: string;
  name?: string;
  source?: string;
};

type BoardItem = {
  id: string;
  path: string;
  name: string;
  source?: string;
  x: number;
  y: number;
  width: number;
  height: number;
  sized: boolean;
  failed: boolean;
};

type Interaction =
  | {
      kind: "pan";
      pointerId: number;
      startX: number;
      startY: number;
      cameraX: number;
      cameraY: number;
    }
  | {
      kind: "move";
      pointerId: number;
      startX: number;
      startY: number;
      itemX: number;
      itemY: number;
      itemId: string;
    };

const persistError = ref(false);
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 32;
const viewportRef = ref<HTMLElement | null>(null);
const items = ref<BoardItem[]>([]);
const selectedId = ref("");
const alwaysOnTop = ref(true);
const camera = reactive({ x: 0, y: 0, scale: 1 });
const interaction = ref<Interaction | null>(null);
let unlistenNativeDrop: null | (() => void) = null;
let unlistenCloseRequested: null | (() => void) = null;
let persistTimer: ReturnType<typeof setTimeout> | null = null;
let isClosing = false;

const worldStyle = computed(() => ({
  transform: `translate3d(${camera.x}px, ${camera.y}px, 0) scale(${camera.scale})`,
}));

function itemStyle(item: BoardItem) {
  return {
    left: `${item.x}px`,
    top: `${item.y}px`,
    width: `${item.width}px`,
    height: `${item.height}px`,
  };
}

function fileName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) || path;
}

function assetSource(item: BoardItem) {
  return item.source || getAssetSrc(item.path);
}

function restoreBoard() {
  try {
    const state = readBoardState(localStorage.getItem(BOARD_STORAGE_KEY));
    items.value = state.items;
    Object.assign(camera, state.camera);
    alwaysOnTop.value = state.alwaysOnTop;
  } catch {
    persistError.value = true;
  }
}

function persistBoard() {
  try {
    writeBoardState(localStorage, items.value, { ...camera }, alwaysOnTop.value);
    persistError.value = false;
    return true;
  } catch {
    persistError.value = true;
    return false;
  }
}

function focusBoard(event: PointerEvent) {
  const target = event.target as HTMLElement;
  if (!target.closest('button')) (event.currentTarget as HTMLElement).focus({ preventScroll: true });
}

async function startWindowDrag(event: PointerEvent) {
  if (event.button === 0 && isTauriRuntime) {
    event.preventDefault();
    await getCurrentWebviewWindow().startDragging();
  }
}

function schedulePersist() {
  if (isClosing) return;
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    persistTimer = null;
    persistBoard();
  }, 120);
}

function viewportCenterInWorld() {
  const viewport = viewportRef.value;
  const width = viewport?.clientWidth || window.innerWidth;
  const height = viewport?.clientHeight || window.innerHeight;
  return {
    x: (width / 2 - camera.x) / camera.scale,
    y: (height / 2 - camera.y) / camera.scale,
  };
}

function clientPointInWorld(point?: { x: number; y: number }) {
  if (!point) return viewportCenterInWorld();
  const viewport = viewportRef.value;
  if (!viewport) return viewportCenterInWorld();
  const rect = viewport.getBoundingClientRect();
  return {
    x: (point.x - rect.left - camera.x) / camera.scale,
    y: (point.y - rect.top - camera.y) / camera.scale,
  };
}

function addAssets(
  incoming: IncomingAsset[],
  clientPoint?: { x: number; y: number },
) {
  const records = incoming
    .map((asset) => {
      const path = String(asset.file_path || asset.path || "");
      return path ? { ...asset, path } : null;
    })
    .filter((asset): asset is IncomingAsset & { path: string } =>
      Boolean(asset),
    );
  if (!records.length) return;

  const dropPoint = clientPointInWorld(clientPoint);
  records.forEach((asset, index) => {
    const existing = items.value.find((item) => item.path === asset.path);
    if (existing) {
      selectedId.value = existing.id;
      return;
    }
    const column = index % 3;
    const row = Math.floor(index / 3);
    const offsetX = (column - Math.min(records.length - 1, 2) / 2) * 28;
    const offsetY = row * 28;
    const item: BoardItem = {
      id: `${Date.now()}-${index}-${Math.random().toString(36).slice(2, 8)}`,
      path: asset.path,
      name: String(asset.name || fileName(asset.path)),
      source: asset.source,
      x: dropPoint.x - 160 + offsetX,
      y: dropPoint.y - 110 + offsetY,
      width: 320,
      height: 220,
      sized: false,
      failed: false,
    };
    items.value.push(item);
    selectedId.value = item.id;
  });
}

function finishImageSize(event: Event, item: BoardItem) {
  if (item.sized) return;
  const image = event.currentTarget as HTMLImageElement;
  if (!image.naturalWidth || !image.naturalHeight) return;
  const longestSide = Math.max(image.naturalWidth, image.naturalHeight);
  const displayScale = Math.min(1, 420 / longestSide);
  const centerX = item.x + item.width / 2;
  const centerY = item.y + item.height / 2;
  item.width = Math.max(80, Math.round(image.naturalWidth * displayScale));
  item.height = Math.max(80, Math.round(image.naturalHeight * displayScale));
  item.x = centerX - item.width / 2;
  item.y = centerY - item.height / 2;
  item.sized = true;
}

function clampZoom(value: number) {
  return Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, value));
}

function zoomAround(nextScale: number, clientX: number, clientY: number) {
  const viewport = viewportRef.value;
  if (!viewport) return;
  const rect = viewport.getBoundingClientRect();
  const localX = clientX - rect.left;
  const localY = clientY - rect.top;
  const worldX = (localX - camera.x) / camera.scale;
  const worldY = (localY - camera.y) / camera.scale;
  const scale = clampZoom(nextScale);
  camera.x = localX - worldX * scale;
  camera.y = localY - worldY * scale;
  camera.scale = scale;
}

function handleWheel(event: WheelEvent) {
  const factor = Math.exp(-event.deltaY * 0.0016);
  zoomAround(camera.scale * factor, event.clientX, event.clientY);
}

function zoomBy(factor: number) {
  const viewport = viewportRef.value;
  if (!viewport) return;
  const rect = viewport.getBoundingClientRect();
  zoomAround(
    camera.scale * factor,
    rect.left + rect.width / 2,
    rect.top + rect.height / 2,
  );
}

function fitAll() {
  const viewport = viewportRef.value;
  if (!viewport || !items.value.length) return;
  const left = Math.min(...items.value.map((item) => item.x));
  const top = Math.min(...items.value.map((item) => item.y));
  const right = Math.max(...items.value.map((item) => item.x + item.width));
  const bottom = Math.max(...items.value.map((item) => item.y + item.height));
  const contentWidth = Math.max(1, right - left);
  const contentHeight = Math.max(1, bottom - top);
  const padding = 72;
  const scale = clampZoom(
    Math.min(
      (viewport.clientWidth - padding * 2) / contentWidth,
      (viewport.clientHeight - padding * 2) / contentHeight,
      1,
    ),
  );
  camera.scale = scale;
  camera.x = viewport.clientWidth / 2 - (left + contentWidth / 2) * scale;
  camera.y = viewport.clientHeight / 2 - (top + contentHeight / 2) * scale;
}

function showActualSize() {
  const viewport = viewportRef.value;
  if (!viewport) return;
  const selected =
    items.value.find((item) => item.id === selectedId.value) || items.value[0];
  camera.scale = 1;
  if (selected) {
    camera.x = viewport.clientWidth / 2 - (selected.x + selected.width / 2);
    camera.y = viewport.clientHeight / 2 - (selected.y + selected.height / 2);
  }
}

function startPan(event: PointerEvent) {
  const target = event.target as HTMLElement;
  if (
    event.button !== 1 &&
    (event.button !== 0 || target.closest(".reference-board-item, button"))
  )
    return;
  event.preventDefault();
  selectedId.value = "";
  interaction.value = {
    kind: "pan",
    pointerId: event.pointerId,
    startX: event.clientX,
    startY: event.clientY,
    cameraX: camera.x,
    cameraY: camera.y,
  };
  bindInteractionListeners();
}

function startMoveItem(event: PointerEvent, item: BoardItem) {
  if (event.button !== 0) return;
  event.preventDefault();
  selectedId.value = item.id;
  interaction.value = {
    kind: "move",
    pointerId: event.pointerId,
    startX: event.clientX,
    startY: event.clientY,
    itemX: item.x,
    itemY: item.y,
    itemId: item.id,
  };
  bindInteractionListeners();
}

function bindInteractionListeners() {
  document.addEventListener("pointermove", moveInteraction, true);
  document.addEventListener("pointerup", endInteraction, true);
  document.addEventListener("pointercancel", endInteraction, true);
}

function unbindInteractionListeners() {
  document.removeEventListener("pointermove", moveInteraction, true);
  document.removeEventListener("pointerup", endInteraction, true);
  document.removeEventListener("pointercancel", endInteraction, true);
}

function moveInteraction(event: PointerEvent) {
  const active = interaction.value;
  if (!active || active.pointerId !== event.pointerId) return;
  event.preventDefault();
  const deltaX = event.clientX - active.startX;
  const deltaY = event.clientY - active.startY;
  if (active.kind === "pan") {
    camera.x = active.cameraX + deltaX;
    camera.y = active.cameraY + deltaY;
    return;
  }
  const item = items.value.find((entry) => entry.id === active.itemId);
  if (!item) return;
  item.x = active.itemX + deltaX / camera.scale;
  item.y = active.itemY + deltaY / camera.scale;
}

function endInteraction(event: PointerEvent) {
  if (!interaction.value || interaction.value.pointerId !== event.pointerId)
    return;
  interaction.value = null;
  unbindInteractionListeners();
}

function removeItem(id: string) {
  items.value = items.value.filter((item) => item.id !== id);
  if (selectedId.value === id) selectedId.value = "";
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    void closeWindow();
    return;
  }
  if (
    (event.key === "Delete" || event.key === "Backspace") &&
    selectedId.value
  ) {
    event.preventDefault();
    removeItem(selectedId.value);
  }
  if ((event.ctrlKey || event.metaKey) && event.key === "0") {
    event.preventDefault();
    fitAll();
  }
}

function handleDomDrop(event: DragEvent) {
  const uri =
    event.dataTransfer?.getData("text/uri-list") ||
    event.dataTransfer?.getData("text/plain") ||
    "";
  const paths = uri
    .split(/\r?\n/)
    .map((entry) => entry.trim())
    .filter((entry) => entry.startsWith("file://"))
    .map((entry) =>
      decodeURIComponent(new URL(entry).pathname).replace(
        /^\/([A-Za-z]:\/)/,
        "$1",
      ),
    );
  addAssets(
    paths.map((path) => ({ path })),
    { x: event.clientX, y: event.clientY },
  );
}

async function toggleAlwaysOnTop() {
  alwaysOnTop.value = !alwaysOnTop.value;
  if (isTauriRuntime) {
    await getCurrentWebviewWindow().setAlwaysOnTop(alwaysOnTop.value);
  }
}

async function minimizeWindow() {
  if (isTauriRuntime) await getCurrentWebviewWindow().minimize();
}

async function closeWindow() {
  if (isClosing) return;
  isClosing = true;
  try {
    if (persistTimer) { clearTimeout(persistTimer); persistTimer = null; }
    interaction.value = null;
    unbindInteractionListeners();
    if (!persistBoard()) return;
    if (isTauriRuntime) {
      // Keep the native window and its geometry; reopening shows this same board.
      await getCurrentWebviewWindow().hide();
      await emit("reference-board:closed");
    }
  } finally { isClosing = false; }
}

onMounted(async () => {
  document.documentElement.classList.add("reference-board-window");
  restoreBoard();
  await nextTick();

  if (!isTauriRuntime && items.value.length === 0) {
    addAssets([
      {
        path: lapLogo,
        source: lapLogo,
        name: "Lap",
      },
    ]);
  }

  window.addEventListener("keydown", handleKeyDown, true);
  if (isTauriRuntime) {
    unlistenNativeDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const { paths, position } = event.payload;
        void getCurrentWebviewWindow().scaleFactor().then((scaleFactor) => {
          addAssets(
            paths.map((path) => ({ path })),
            {
              x: position.x / scaleFactor,
              y: position.y / scaleFactor,
            },
          );
        });
      }
    });
    const currentWindow = getCurrentWebviewWindow();
    unlistenCloseRequested = await currentWindow.onCloseRequested((event) => {
      event.preventDefault();
      void closeWindow();
    });
    await currentWindow.setAlwaysOnTop(alwaysOnTop.value);
    await currentWindow.show();
  }
});

watch([items, camera, alwaysOnTop], schedulePersist, { deep: true });

onBeforeUnmount(() => {
  document.documentElement.classList.remove("reference-board-window");
  if (persistTimer) clearTimeout(persistTimer);
  if (!isClosing) persistBoard();
  window.removeEventListener("keydown", handleKeyDown, true);
  unbindInteractionListeners();
  unlistenNativeDrop?.();
  unlistenCloseRequested?.();
});
</script>

<style scoped>
:global(html.reference-board-window),
:global(html.reference-board-window body),
:global(html.reference-board-window #app) {
  background: transparent !important;
  border: 0 !important;
  box-shadow: none !important;
}
.reference-board {
  --board-surface: rgb(0 0 0 / 0.46);
  --board-controls: rgb(16 16 19 / 0.72);
  position: relative;
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  outline: none;
  background: var(--board-surface);
  color: var(--color-base-content);
}
:global(html[data-lap-appearance="light"] .reference-board) {
  --board-surface: rgb(255 255 255 / 0.46);
  --board-controls: rgb(255 255 255 / 0.78);
}
.reference-board-toolbar {
  position: absolute;
  z-index: 5;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 4px;
  padding: 5px;
  border: 0;
  border-radius: 12px;
  background: var(--board-controls);
  backdrop-filter: blur(16px);
  opacity: 0.25;
  transition: opacity 150ms ease;
}
.reference-board-toolbar:hover, .reference-board-toolbar:focus-within { opacity: 1; }
.reference-board-move {
  width: 28px;
  border: 0;
  color: var(--color-base-content);
  background: transparent;
  cursor: move;
  font-size: 24px;
}
.reference-board-tools {
  display: flex;
  align-items: center;
  gap: 3px;
}

.reference-board-tools button {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  border: 1px solid transparent;
  border-radius: 9px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 62%, transparent);
  cursor: pointer;
  transition:
    border-color 140ms ease,
    background 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.reference-board-tools button:hover,
.reference-board-tools button.is-active {
  border-color: color-mix(in oklab, var(--color-primary) 34%, transparent);
  background: color-mix(
    in oklab,
    var(--color-primary) 12%,
    var(--color-base-100)
  );
  color: var(--color-base-content);
  transform: translateY(-1px);
}

.reference-board-tools button :deep(svg) {
  width: 15px;
  height: 15px;
}

.reference-board-divider {
  width: 1px;
  height: 17px;
  margin: 0 3px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.reference-board-viewport {
  position: relative;
  min-height: 0;
  flex: 1;
  overflow: hidden;
  background: transparent;
  cursor: grab;
  touch-action: none;
}

.reference-board-viewport.is-panning {
  cursor: grabbing;
}

.reference-board-world {
  position: absolute;
  inset: 0;
  transform-origin: 0 0;
  will-change: transform;
}

.reference-board-item {
  position: absolute;
  display: grid;
  place-items: center;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
  cursor: move;
  transform-origin: center;
  transition:
    border-color 140ms ease,
    box-shadow 140ms ease;
}

.reference-board-item.is-selected {
  outline: 1px solid color-mix(in oklab, var(--color-primary) 70%, transparent);
}

.reference-board-item > img {
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
  object-fit: contain;
}

.reference-board-item > button {
  position: absolute;
  top: 7px;
  right: 7px;
  display: grid;
  width: 26px;
  height: 26px;
  place-items: center;
  border: 1px solid rgb(255 255 255 / 0.12);
  border-radius: 8px;
  background: rgb(17 18 22 / 0.78);
  color: rgb(255 255 255 / 0.76);
  cursor: pointer;
  backdrop-filter: blur(12px);
}

.reference-board-item > button:hover {
  background: rgb(31 32 38 / 0.92);
  color: white;
}

.reference-board-item > button :deep(svg) {
  width: 13px;
  height: 13px;
}

.reference-board-image-error {
  position: absolute;
  inset: 0;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: 7px;
  padding: 18px;
  color: color-mix(in oklab, var(--color-base-content) 48%, transparent);
  font-size: 11px;
  text-align: center;
}

.reference-board-image-error :deep(svg) {
  width: 24px;
  height: 24px;
}

.reference-board-empty {
  position: absolute;
  top: 50%;
  left: 50%;
  z-index: 2;
  display: grid;
  width: min(360px, calc(100% - 48px));
  justify-items: center;
  gap: 8px;
  padding: 28px;
  border: 1px solid
    color-mix(in oklab, var(--color-base-content) 8%, transparent);
  border-radius: 22px;
  background: color-mix(in oklab, var(--color-base-100) 58%, transparent);
  color: color-mix(in oklab, var(--color-base-content) 52%, transparent);
  text-align: center;
  transform: translate(-50%, -50%);
  backdrop-filter: blur(18px);
}

.reference-board-empty :deep(svg) {
  width: 32px;
  height: 32px;
  color: color-mix(
    in oklab,
    var(--color-primary) 68%,
    var(--color-base-content)
  );
}

.reference-board-empty strong {
  color: var(--color-base-content);
  font-size: 14px;
}

.reference-board-empty span {
  max-width: 270px;
  font-size: 10px;
  line-height: 1.5;
}

.reference-board-status {
  position: absolute;
  z-index: 4;
  inset: auto 12px 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  color: color-mix(in oklab, var(--color-base-content) 65%, transparent);
  font-size: 10px;
  pointer-events: none;
  user-select: none;
}
.reference-board-save-error { color: var(--color-error); }

.reference-board-status strong {
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
  font-variant-numeric: tabular-nums;
}
</style>
