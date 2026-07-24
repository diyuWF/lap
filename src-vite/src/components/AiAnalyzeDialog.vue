<template>
  <div class="fixed inset-0 z-[130] flex items-center justify-center bg-black/65 p-4" @mousedown.self="$emit('close')">
    <section class="flex max-h-[86vh] w-[760px] max-w-[96vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-start justify-between border-b border-base-content/10 px-5 py-4">
        <div class="min-w-0">
          <h2 class="font-semibold">在线 AI 分析</h2>
          <p class="mt-1 truncate text-xs text-base-content/45" :title="file?.file_path || file?.name">{{ file?.name || '当前素材' }}</p>
        </div>
        <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">关闭</button>
      </header>

      <main class="min-h-0 flex-1 overflow-y-auto p-5">
        <div v-if="loadingProviders" class="flex min-h-48 items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="!providers.length" class="rounded-box border border-warning/25 bg-warning/10 p-4 text-sm">
          尚未配置可用的在线 AI 服务。请先到“设置 → 高级 → AI 自动整理”中添加服务。
        </div>

        <template v-else>
          <div class="grid grid-cols-2 gap-4">
            <label class="form-control col-span-2 gap-1">
              <span class="text-xs text-base-content/60">AI 服务</span>
              <select v-model="providerId" class="select select-bordered select-sm">
                <option v-for="provider in providers" :key="provider.id" :value="provider.id">
                  {{ provider.name }} · {{ provider.model }}
                </option>
              </select>
            </label>

            <label class="flex items-center gap-2 rounded-box border border-base-content/10 bg-base-300/25 p-3 text-sm">
              <input v-model="forceApply" class="toggle toggle-primary toggle-sm" type="checkbox" />
              本次达到阈值后直接应用标签
            </label>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-3 text-sm">
              <div class="text-xs text-base-content/45">分析范围</div>
              <div class="mt-1">画面主体、风格、构图、光线、颜色与用途</div>
            </div>
          </div>

          <div v-if="busy" class="mt-5 flex min-h-40 flex-col items-center justify-center gap-3 rounded-box border border-base-content/10 bg-base-300/20">
            <span class="loading loading-spinner loading-md"></span>
            <span class="text-sm text-base-content/55">正在生成结构化素材信息…</span>
          </div>

          <div v-else-if="result" class="mt-5 space-y-4">
            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-1 text-xs text-base-content/45">建议标题</div>
              <div class="font-medium">{{ result.analysis?.title || '—' }}</div>
            </div>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-1 text-xs text-base-content/45">描述</div>
              <p class="whitespace-pre-wrap text-sm leading-6">{{ result.analysis?.description || '—' }}</p>
            </div>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-2 flex items-center justify-between">
                <span class="text-xs text-base-content/45">标签</span>
                <span class="text-xs tabular-nums text-base-content/45">置信度 {{ confidenceLabel }}</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <span v-for="tag in result.analysis?.tags || []" :key="tag" class="badge badge-primary badge-outline">{{ tag }}</span>
                <span v-if="!(result.analysis?.tags || []).length" class="text-sm text-base-content/40">没有返回标签</span>
              </div>
            </div>

            <div v-if="result.analysis?.dominantColors?.length" class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-2 text-xs text-base-content/45">主色</div>
              <div class="flex flex-wrap gap-2">
                <div v-for="color in result.analysis.dominantColors" :key="color" class="flex items-center gap-2 rounded-box bg-base-100/40 px-2 py-1 text-xs font-mono">
                  <span class="h-4 w-4 rounded border border-base-content/15" :style="{ backgroundColor: color }"></span>
                  {{ color }}
                </div>
              </div>
            </div>

            <div class="alert py-2 text-sm" :class="result.tagsApplied ? 'alert-success' : 'alert-info'">
              {{ result.tagsApplied ? `已应用 ${result.appliedTagIds?.length || 0} 个标签。` : '建议已进入审核队列，尚未修改正式标签。' }}
            </div>
          </div>
        </template>
      </main>

      <footer class="flex items-center justify-between border-t border-base-content/10 px-5 py-3">
        <span class="text-xs text-base-content/40">API 密钥只从本机配置读取，不会显示在结果中。</span>
        <button class="btn btn-primary btn-sm" type="button" :disabled="busy || loadingProviders || !providerId" @click="analyze">
          {{ busy ? '分析中…' : result ? '重新分析' : '开始分析' }}
        </button>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { analyzeFileWithOnlineAi, listOnlineAiProviders } from '@/common/dam-api';
import { useToast } from '@/common/toast';

const props = defineProps<{
  file: Record<string, any> | null;
}>();

const emit = defineEmits(['close', 'done']);
const toast = useToast();
const providers = ref<any[]>([]);
const providerId = ref('');
const loadingProviders = ref(true);
const busy = ref(false);
const forceApply = ref(false);
const result = ref<any>(null);

const confidenceLabel = computed(() => {
  const confidence = Number(result.value?.analysis?.confidence);
  return Number.isFinite(confidence) ? `${Math.round(confidence * 100)}%` : '—';
});

async function loadProviders() {
  loadingProviders.value = true;
  try {
    providers.value = (await listOnlineAiProviders()).filter((provider: any) => provider.enabled && provider.hasApiKey);
    providerId.value = providers.value[0]?.id || '';
  } catch (error) {
    toast.error(String(error));
  } finally {
    loadingProviders.value = false;
  }
}

async function analyze() {
  const fileId = Number(props.file?.id || 0);
  if (!fileId || !providerId.value) return;
  busy.value = true;
  result.value = null;
  try {
    result.value = await analyzeFileWithOnlineAi(fileId, providerId.value, forceApply.value);
    toast.success('AI 分析完成');
    emit('done', result.value);
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

onMounted(loadProviders);
</script>
