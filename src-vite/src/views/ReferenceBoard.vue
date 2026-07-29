<template>
  <div class="reference-board">
    <header class="reference-board-titlebar" data-tauri-drag-region>
      <div class="reference-board-brand" data-tauri-drag-region>
        <img :src="lapLogo" alt="" draggable="false" />
        <span data-tauri-drag-region>
          <strong>{{ $t("reference_board.title") }}</strong>
          <small>{{
            $t("reference_board.asset_count", { count: items.length })
          }}</small>
        </span>
      </div>

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
    </header>

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

    <footer class="reference-board-status">
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
import { useRoute } from "vue-router";
import { listen } from "@tauri-apps/api/event";
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

const STORAGE_KEY = "lap-reference-board-v1";
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 32;
const route = useRoute();
const viewportRef = ref<HTMLElement | null>(null);
const items = ref<BoardItem[]>([]);
const selectedId = ref("");
const alwaysOnTop = ref(true);
const camera = reactive({ x: 0, y: 0, scale: 1 });
const interaction = ref<Interaction | null>(null);
let unlistenAssets: null | (() => void) = null;
let unlistenNativeDrop: null | (() => void) = null;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

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

function readInitialAssets(): IncomingAsset[] {
  const value = route.query.assets;
  if (typeof value !== "string" || !value) return [];
  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function restoreBoard() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const state = JSON.parse(raw);
    if (Array.isArray(state.items)) {
      items.value = state.items.filter(
        (item: BoardItem) => item?.id && item?.path,
      );
    }
    if (state.camera) {
      camera.x = Number(state.camera.x) || 0;
      camera.y = Number(state.camera.y) || 0;
      camera.scale = clampZoom(Number(state.camera.scale) || 1);
    }
  } catch {
    localStorage.removeItem(STORAGE_KEY);
  }
}

function persistBoard() {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      items: items.value,
      camera: { ...camera },
    }),
  );
}

function schedulePersist() {
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

function addAssets(incoming: IncomingAsset[]) {
  const records = incoming
    .map((asset) => {
      const path = String(asset.file_path || asset.path || "");
      return path ? { ...asset, path } : null;
    })
    .filter((asset): asset is IncomingAsset & { path: string } =>
      Boolean(asset),
    );
  if (!records.length) return;

  const center = viewportCenterInWorld();
  records.forEach((asset, index) => {
    const existing = items.value.find((item) => item.path === asset.path);
    if (existing) {
      selectedId.value = existing.id;
      return;
    }
    const offset = index * 26;
    const item: BoardItem = {
      id: `${Date.now()}-${index}-${Math.random().toString(36).slice(2, 8)}`,
      path: asset.path,
      name: String(asset.name || fileName(asset.path)),
      source: asset.source,
      x: center.x - 160 + offset,
      y: center.y - 110 + offset,
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
  item.width = Math.max(80, Math.round(image.naturalWidth * displayScale));
  item.height = Math.max(80, Math.round(image.naturalHeight * displayScale));
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
  addAssets(paths.map((path) => ({ path })));
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
  if (isTauriRuntime) await getCurrentWebviewWindow().close();
}

onMounted(async () => {
  restoreBoard();
  await nextTick();
  addAssets(readInitialAssets());

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
    unlistenAssets = await listen<IncomingAsset[]>(
      "reference-board:add-assets",
      (event) => {
        addAssets(Array.isArray(event.payload) ? event.payload : []);
      },
    );
    unlistenNativeDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        addAssets(event.payload.paths.map((path) => ({ path })));
      }
    });
    await getCurrentWebviewWindow().setAlwaysOnTop(true);
    await getCurrentWebviewWindow().show();
  }
});

watch([items, camera], schedulePersist, { deep: true });

onBeforeUnmount(() => {
  if (persistTimer) clearTimeout(persistTimer);
  persistBoard();
  window.removeEventListener("keydown", handleKeyDown, true);
  unbindInteractionListeners();
  unlistenAssets?.();
  unlistenNativeDrop?.();
});
</script>

<style scoped>
.reference-board {
  display: flex;
  width: 100vw;
  height: 100vh;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid
    color-mix(in oklab, var(--color-base-content) 11%, transparent);
  border-radius: 12px;
  background: color-mix(in oklab, var(--color-base-300) 96%, transparent);
  box-shadow:
    -28px 24px 78px rgb(91 55 255 / 0.12),
    28px -18px 72px rgb(241 113 59 / 0.08),
    0 28px 80px rgb(0 0 0 / 0.45);
  color: var(--color-base-content);
}

.reference-board-titlebar {
  display: flex;
  min-height: 48px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 0 8px 0 12px;
  border-bottom: 1px solid
    color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: color-mix(in oklab, var(--color-base-200) 90%, transparent);
  user-select: none;
  backdrop-filter: blur(18px);
}

.reference-board-brand {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 9px;
}

.reference-board-brand img {
  width: 28px;
  height: 28px;
  flex: none;
  border-radius: 9px;
  box-shadow: 0 7px 20px rgb(103 76 255 / 0.22);
}

.reference-board-brand > span {
  display: grid;
  min-width: 0;
  gap: 0;
}

.reference-board-brand strong {
  overflow: hidden;
  font-size: 12px;
  font-weight: 750;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.reference-board-brand small {
  color: color-mix(in oklab, var(--color-base-content) 48%, transparent);
  font-size: 9px;
  line-height: 12px;
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
  background: color-mix(in oklab, var(--color-base-300) 94%, #07080b);
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
  border: 1px solid
    color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  background: color-mix(in oklab, var(--color-base-100) 88%, transparent);
  box-shadow: 0 18px 48px rgb(0 0 0 / 0.34);
  cursor: move;
  transform-origin: center;
  transition:
    border-color 140ms ease,
    box-shadow 140ms ease;
}

.reference-board-item.is-selected {
  border-color: color-mix(in oklab, var(--color-primary) 74%, white 12%);
  box-shadow:
    0 0 0 3px color-mix(in oklab, var(--color-primary) 16%, transparent),
    -18px 16px 48px rgb(91 55 255 / 0.16),
    18px -10px 42px rgb(241 113 59 / 0.08),
    0 20px 52px rgb(0 0 0 / 0.42);
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
  display: flex;
  min-height: 28px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 0 12px;
  border-top: 1px solid
    color-mix(in oklab, var(--color-base-content) 8%, transparent);
  background: color-mix(in oklab, var(--color-base-200) 88%, transparent);
  color: color-mix(in oklab, var(--color-base-content) 44%, transparent);
  font-size: 9px;
  user-select: none;
}

.reference-board-status strong {
  color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
  font-variant-numeric: tabular-nums;
}
</style>
