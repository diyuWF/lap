<template>
  <div ref="host" class="relative h-full w-full overflow-hidden bg-base-300">
    <canvas ref="canvas" class="block h-full w-full"></canvas>

    <div class="absolute left-3 top-3 z-20 flex items-center gap-2 rounded-box border border-base-content/10 bg-base-200/85 p-2 shadow backdrop-blur">
      <button class="btn btn-ghost btn-xs" type="button" :disabled="loading" @click="resetCamera">重置视角</button>
      <label class="flex cursor-pointer items-center gap-2 text-xs">
        <input v-model="wireframe" class="toggle toggle-primary toggle-xs" type="checkbox" @change="applyWireframe" />
        线框
      </label>
    </div>

    <div v-if="loading" class="absolute inset-0 z-30 flex items-center justify-center bg-base-300/75">
      <div class="flex flex-col items-center gap-3 text-sm text-base-content/60">
        <span class="loading loading-spinner loading-md"></span>
        <span>正在加载三维模型…</span>
      </div>
    </div>

    <div v-else-if="errorMessage" class="absolute inset-0 z-30 flex items-center justify-center p-8">
      <div class="max-w-lg rounded-box border border-error/30 bg-base-200 p-5 text-center shadow-xl">
        <h3 class="mb-2 font-semibold text-error">三维模型加载失败</h3>
        <p class="break-words text-sm text-base-content/60">{{ errorMessage }}</p>
      </div>
    </div>

    <div v-else class="pointer-events-none absolute bottom-3 left-1/2 -translate-x-1/2 rounded-box bg-base-200/70 px-3 py-1.5 text-xs text-base-content/55 backdrop-blur">
      左键旋转 · 滚轮缩放 · 右键平移
    </div>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js';

const props = defineProps<{
  filePath: string;
  extension?: string;
}>();

const host = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const loading = ref(true);
const errorMessage = ref('');
const wireframe = ref(false);

let renderer: THREE.WebGLRenderer | null = null;
let scene: THREE.Scene | null = null;
let camera: THREE.PerspectiveCamera | null = null;
let controls: OrbitControls | null = null;
let modelRoot: THREE.Object3D | null = null;
let animationFrame = 0;
let resizeObserver: ResizeObserver | null = null;
let disposed = false;

function extensionOf(path: string) {
  const explicit = String(props.extension || '').trim().toLowerCase();
  if (explicit) return explicit;
  return path.split('.').pop()?.toLowerCase() || '';
}

function initializeScene() {
  if (!canvas.value || !host.value) return;
  renderer = new THREE.WebGLRenderer({ canvas: canvas.value, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.1;

  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(42, 1, 0.01, 100000);
  camera.position.set(3, 2.2, 4.5);

  controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.screenSpacePanning = true;

  scene.add(new THREE.HemisphereLight(0xffffff, 0x303040, 2.2));
  const keyLight = new THREE.DirectionalLight(0xffffff, 2.8);
  keyLight.position.set(4, 7, 5);
  scene.add(keyLight);
  const fillLight = new THREE.DirectionalLight(0xb8c8ff, 1.2);
  fillLight.position.set(-5, 2, -3);
  scene.add(fillLight);

  const grid = new THREE.GridHelper(20, 20, 0x808080, 0x4a4a4a);
  grid.name = '__lap_grid__';
  scene.add(grid);

  resizeObserver = new ResizeObserver(resize);
  resizeObserver.observe(host.value);
  resize();
  renderLoop();
}

function resize() {
  if (!host.value || !renderer || !camera) return;
  const width = Math.max(host.value.clientWidth, 1);
  const height = Math.max(host.value.clientHeight, 1);
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
}

function renderLoop() {
  if (disposed) return;
  controls?.update();
  if (renderer && scene && camera) renderer.render(scene, camera);
  animationFrame = requestAnimationFrame(renderLoop);
}

function disposeObject(root: THREE.Object3D | null) {
  root?.traverse((child: any) => {
    child.geometry?.dispose?.();
    const materials = Array.isArray(child.material) ? child.material : [child.material];
    for (const material of materials) {
      if (!material) continue;
      for (const value of Object.values(material)) {
        if (value instanceof THREE.Texture) value.dispose();
      }
      material.dispose?.();
    }
  });
}

function clearModel() {
  if (modelRoot && scene) scene.remove(modelRoot);
  disposeObject(modelRoot);
  modelRoot = null;
}

function normalizeObject(root: THREE.Object3D) {
  const box = new THREE.Box3().setFromObject(root);
  if (box.isEmpty()) throw new Error('模型中没有可显示的几何体');
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  root.position.sub(center);

  const maxDimension = Math.max(size.x, size.y, size.z, 0.001);
  const scale = 4 / maxDimension;
  root.scale.multiplyScalar(scale);
  root.updateMatrixWorld(true);
  return root;
}

function frameObject() {
  if (!camera || !controls || !modelRoot) return;
  const box = new THREE.Box3().setFromObject(modelRoot);
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  const radius = Math.max(size.length() * 0.5, 0.5);
  const distance = radius / Math.tan(THREE.MathUtils.degToRad(camera.fov * 0.5));
  camera.position.copy(center).add(new THREE.Vector3(1, 0.65, 1).normalize().multiplyScalar(distance * 1.25));
  camera.near = Math.max(radius / 1000, 0.001);
  camera.far = Math.max(radius * 1000, 1000);
  camera.updateProjectionMatrix();
  controls.target.copy(center);
  controls.minDistance = radius * 0.08;
  controls.maxDistance = radius * 30;
  controls.update();
}

function resetCamera() {
  frameObject();
}

function applyWireframe() {
  modelRoot?.traverse((child: any) => {
    const materials = Array.isArray(child.material) ? child.material : [child.material];
    for (const material of materials) {
      if (material && 'wireframe' in material) {
        material.wireframe = wireframe.value;
        material.needsUpdate = true;
      }
    }
  });
}

async function loadModel() {
  if (!scene || !props.filePath) return;
  loading.value = true;
  errorMessage.value = '';
  clearModel();

  try {
    const url = convertFileSrc(props.filePath);
    const extension = extensionOf(props.filePath);
    let loaded: THREE.Object3D;

    if (extension === 'glb' || extension === 'gltf') {
      const gltf = await new GLTFLoader().loadAsync(url);
      loaded = gltf.scene;
    } else if (extension === 'obj') {
      loaded = await new OBJLoader().loadAsync(url);
    } else if (extension === 'stl') {
      const geometry = await new STLLoader().loadAsync(url);
      geometry.computeVertexNormals();
      loaded = new THREE.Mesh(
        geometry,
        new THREE.MeshStandardMaterial({ color: 0xb8bec9, roughness: 0.72, metalness: 0.08 }),
      );
    } else {
      throw new Error(`暂不支持 .${extension || '未知'} 三维格式`);
    }

    modelRoot = normalizeObject(loaded);
    scene.add(modelRoot);
    applyWireframe();
    frameObject();
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await nextTick();
  initializeScene();
  await loadModel();
});

watch(() => props.filePath, loadModel);

onBeforeUnmount(() => {
  disposed = true;
  cancelAnimationFrame(animationFrame);
  resizeObserver?.disconnect();
  controls?.dispose();
  clearModel();
  renderer?.dispose();
  renderer?.forceContextLoss();
  renderer = null;
  scene = null;
  camera = null;
  controls = null;
});
</script>
