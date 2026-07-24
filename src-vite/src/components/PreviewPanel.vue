<template>
  <div class="relative h-full w-full overflow-hidden bg-base-300">
    <div v-if="loading" class="absolute inset-0 flex items-center justify-center">
      <div class="flex flex-col items-center gap-3 text-sm text-base-content/55">
        <span class="loading loading-spinner loading-md"></span>
        <span>正在准备预览…</span>
      </div>
    </div>

    <div v-else-if="errorMessage" class="absolute inset-0 flex items-center justify-center p-8">
      <div class="max-w-lg rounded-box border border-error/25 bg-base-200 p-5 text-center shadow-xl">
        <h3 class="mb-2 font-semibold text-error">无法打开预览</h3>
        <p class="break-words text-sm text-base-content/60">{{ errorMessage }}</p>
        <button class="btn btn-sm mt-4" type="button" @click="revealFile">在资源管理器中显示</button>
      </div>
    </div>

    <Model3dViewer
      v-else-if="descriptor?.kind === 'model3d'"
      :file-path="descriptor.filePath"
      :extension="descriptor.extension"
    />

    <div v-else-if="descriptor?.kind === 'svg'" class="h-full w-full overflow-auto p-5 checkerboard">
      <img
        :src="assetUrl"
        :alt="descriptor.fileName"
        class="mx-auto block min-h-0 max-h-full max-w-full object-contain drop-shadow-xl"
      />
    </div>

    <div v-else-if="descriptor?.kind === 'pdf'" class="h-full w-full bg-base-100">
      <iframe
        :key="assetUrl"
        :src="assetUrl"
        class="h-full w-full border-0"
        :title="descriptor.fileName"
      ></iframe>
    </div>

    <div v-else class="absolute inset-0 flex items-center justify-center p-8">
      <div class="w-full max-w-xl rounded-box border border-base-content/10 bg-base-200/90 p-6 text-center shadow-xl">
        <div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-base-300 text-xl font-bold text-base-content/50">
          {{ extensionLabel }}
        </div>
        <h3 class="truncate font-semibold" :title="descriptor?.fileName">{{ descriptor?.fileName }}</h3>
        <p class="mt-2 text-sm leading-6 text-base-content/55">
          {{ descriptor?.message || '该文件暂不支持内置预览。' }}
        </p>
        <div class="mt-5 flex justify-center gap-2">
          <button class="btn btn-sm" type="button" @click="revealFile">在资源管理器中显示</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getPreviewDescriptor } from '@/common/dam-api';
import Model3dViewer from '@/components/Model3dViewer.vue';

const props = defineProps<{
  file: Record<string, any> | null;
}>();

const descriptor = ref<any>(null);
const loading = ref(false);
const errorMessage = ref('');
let requestToken = 0;

const assetUrl = computed(() => {
  const path = descriptor.value?.filePath || props.file?.file_path || '';
  return path ? convertFileSrc(path) : '';
});

const extensionLabel = computed(() => {
  const extension = descriptor.value?.extension || '';
  return extension ? extension.toUpperCase().slice(0, 8) : 'FILE';
});

async function loadDescriptor() {
  const fileId = Number(props.file?.id || 0);
  const token = ++requestToken;
  descriptor.value = null;
  errorMessage.value = '';
  if (!fileId) {
    errorMessage.value = '文件记录无效。';
    return;
  }

  loading.value = true;
  try {
    const result = await getPreviewDescriptor(fileId);
    if (token === requestToken) descriptor.value = result;
  } catch (error) {
    if (token === requestToken) errorMessage.value = String(error);
  } finally {
    if (token === requestToken) loading.value = false;
  }
}

async function revealFile() {
  const path = descriptor.value?.filePath || props.file?.file_path;
  if (!path) return;
  try {
    await invoke('reveal_path', { path });
  } catch (error) {
    errorMessage.value = String(error);
  }
}

watch(() => props.file?.id, loadDescriptor, { immediate: true });
</script>

<style scoped>
.checkerboard {
  background-color: rgb(128 128 128 / 0.08);
  background-image:
    linear-gradient(45deg, rgb(128 128 128 / 0.12) 25%, transparent 25%),
    linear-gradient(-45deg, rgb(128 128 128 / 0.12) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, rgb(128 128 128 / 0.12) 75%),
    linear-gradient(-45deg, transparent 75%, rgb(128 128 128 / 0.12) 75%);
  background-size: 24px 24px;
  background-position: 0 0, 0 12px, 12px -12px, -12px 0;
}
</style>
