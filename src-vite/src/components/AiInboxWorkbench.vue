<template>
  <div class="fixed inset-0 z-[125] flex items-center justify-center bg-black/65 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[88vh] w-[1120px] max-w-[97vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-start justify-between border-b border-base-content/10 px-5 py-4">
        <div>
          <h2 class="font-semibold">AI Inbox 工作台</h2>
          <p class="mt-1 text-xs text-base-content/45">批量发现未整理素材，生成建议后进入人工审核队列。</p>
        </div>
        <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">关闭</button>
      </header>

      <div class="grid grid-cols-[180px_1fr_180px_auto] items-end gap-3 border-b border-base-content/10 bg-base-300/20 px-5 py-3">
        <label class="form-control gap-1">
          <span class="text-xs text-base-content/55">工作流状态</span>
          <select v-model="workflowStatus" class="select select-bordered select-sm" @change="loadCandidates">
            <option value="inbox">待整理</option>
            <option value="reviewed">已审核</option>
            <option value="selected">已选用</option>
            <option value="archived">已归档</option>
            <option value="all">全部</option>
          </select>
        </label>

        <label class="form-control gap-1">
          <span class="text-xs text-base-content/55">在线 AI 服务</span>
          <select v-model="providerId" class="select select-bordered select-sm" :disabled="!providers.length">
            <option v-for="provider in providers" :key="provider.id" :value="provider.id">
              {{ provider.name }} · {{ provider.model }}
            </option>
          </select>
        </label>

        <label class="flex h-8 items-center gap-2 text-sm">
          <input v-model="includeAnalyzed" class="toggle toggle-primary toggle-sm" type="checkbox" @change="loadCandidates" />
          包含已分析素材
        </label>

        <button class="btn btn-ghost btn-sm" type="button" :disabled="loading || running" @click="loadCandidates">刷新候选</button>
      </div>

      <div class="flex items-center justify-between border-b border-base-content/10 px-5 py-2.5">
        <label class="flex items-center gap-2 text-sm">
          <input v-model="selectAll" class="checkbox checkbox-sm" type="checkbox" :disabled="!candidates.length || running" @change="toggleAll" />
          已选择 {{ selectedIds.size }} / {{ candidates.length }}
        </label>
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-2 text-sm">
            <input v-model="forceAutoApply" class="toggle toggle-primary toggle-sm" type="checkbox" :disabled="running" />
            达到阈值后直接应用标签
          </label>
          <button class="btn btn-primary btn-sm" type="button" :disabled="running || !selectedIds.size || !providerId" @click="runBatch">
            {{ running ? '批量分析中…' : `分析 ${selectedIds.size} 个素材` }}
          </button>
        </div>
      </div>

      <main class="min-h-0 flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="!providers.length" class="flex h-full items-center justify-center">
          <div class="max-w-lg rounded-box border border-warning/25 bg-warning/10 p-5 text-center text-sm leading-6">
            没有已启用且配置了密钥的在线 AI 服务。请先在设置中的“在线 AI 服务”完成配置。
          </div>
        </div>

        <div v-else-if="!candidates.length" class="flex h-full flex-col items-center justify-center gap-2 text-base-content/45">
          <p class="text-sm">当前筛选条件下没有待整理素材</p>
          <p class="text-xs">关闭“跳过已分析素材”后可重新分析历史素材。</p>
        </div>

        <div v-else class="space-y-2">
          <article
            v-for="item in candidates"
            :key="item.fileId"
            class="grid grid-cols-[32px_1fr_100px_120px] items-center gap-3 rounded-box border border-base-content/10 bg-base-300/25 px-3 py-3"
          >
            <input
              class="checkbox checkbox-sm"
              type="checkbox"
              :checked="selectedIds.has(item.fileId)"
              :disabled="running"
              @change="toggleItem(item.fileId)"
            />
            <div class="min-w-0">
              <div class="truncate text-sm font-medium" :title="item.fileName">{{ item.fileName }}</div>
              <div class="truncate text-[11px] text-base-content/40" :title="item.filePath">{{ item.filePath }}</div>
            </div>
            <span class="badge badge-ghost badge-sm justify-self-start">{{ fileTypeLabel(item.fileType) }}</span>
            <div class="text-right text-xs text-base-content/45">
              <span>{{ workflowLabel(item.workflowStatus) }}</span>
              <span v-if="item.hasAiSuggestions" class="ml-2 text-warning">已有建议</span>
            </div>
          </article>
        </div>
      </main>

      <footer class="border-t border-base-content/10 px-5 py-3">
        <div v-if="running" class="flex items-center gap-3 text-sm text-base-content/55">
          <span class="loading loading-spinner loading-sm"></span>
          正在逐个提交分析。为了避免 API 限流，本批任务按顺序执行。
        </div>
        <div v-else-if="batchResult" class="flex items-center justify-between gap-4 text-sm">
          <span>
            本批完成 {{ batchResult.completed }} / {{ batchResult.total }}，成功
            <strong class="text-success">{{ batchResult.succeeded }}</strong>，失败
            <strong :class="batchResult.failed ? 'text-error' : 'text-base-content/55'">{{ batchResult.failed }}</strong>。
          </span>
          <div class="flex gap-2">
            <button v-if="batchResult.failed" class="btn btn-ghost btn-xs" type="button" @click="showFailures = !showFailures">
              {{ showFailures ? '收起失败项' : '查看失败项' }}
            </button>
            <button class="btn btn-primary btn-xs" type="button" @click="$emit('open-review')">打开审核队列</button>
          </div>
        </div>
        <div v-else class="text-xs text-base-content/40">单次最多处理 200 个素材。默认跳过已经产生 AI 建议的素材。</div>

        <div v-if="showFailures && batchResult?.failures?.length" class="mt-3 max-h-32 overflow-y-auto rounded-box border border-error/20 bg-error/5 p-3 text-xs">
          <div v-for="failure in batchResult.failures" :key="failure.fileId" class="mb-1 last:mb-0">
            文件 {{ failure.fileId }}：{{ failure.error }}
          </div>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import {
  analyzeFilesWithOnlineAi,
  listOnlineAiBatchCandidates,
  listOnlineAiProviders,
} from '@/common/dam-api';
import { useToast } from '@/common/toast';

const emit = defineEmits(['close', 'open-review']);
const toast = useToast();
const providers = ref<any[]>([]);
const candidates = ref<any[]>([]);
const providerId = ref('');
const workflowStatus = ref('inbox');
const includeAnalyzed = ref(false);
const forceAutoApply = ref(false);
const loading = ref(false);
const running = ref(false);
const selectedIds = ref(new Set<number>());
const selectAll = ref(false);
const batchResult = ref<any>(null);
const showFailures = ref(false);

async function loadProviders() {
  const allProviders = await listOnlineAiProviders();
  providers.value = allProviders.filter((provider: any) => provider.enabled && provider.hasApiKey);
  if (!providers.value.some((provider: any) => provider.id === providerId.value)) {
    providerId.value = providers.value[0]?.id || '';
  }
}

async function loadCandidates() {
  loading.value = true;
  selectedIds.value = new Set();
  selectAll.value = false;
  try {
    await loadProviders();
    candidates.value = await listOnlineAiBatchCandidates(
      workflowStatus.value,
      200,
      includeAnalyzed.value,
    );
  } catch (error) {
    toast.error(String(error));
  } finally {
    loading.value = false;
  }
}

function toggleItem(fileId: number) {
  const next = new Set(selectedIds.value);
  if (next.has(fileId)) next.delete(fileId);
  else next.add(fileId);
  selectedIds.value = next;
  selectAll.value = candidates.value.length > 0 && candidates.value.every((item) => next.has(item.fileId));
}

function toggleAll() {
  selectedIds.value = selectAll.value
    ? new Set(candidates.value.map((item) => item.fileId))
    : new Set();
}

async function runBatch() {
  if (!providerId.value || !selectedIds.value.size) return;
  running.value = true;
  batchResult.value = null;
  showFailures.value = false;
  try {
    batchResult.value = await analyzeFilesWithOnlineAi({
      fileIds: [...selectedIds.value],
      providerId: providerId.value,
      forceAutoApply: forceAutoApply.value,
      continueOnError: true,
    });
    if (batchResult.value.failed) {
      toast.warning(`批量分析完成，${batchResult.value.failed} 个素材失败`);
    } else {
      toast.success(`已完成 ${batchResult.value.succeeded} 个素材的 AI 分析`);
    }
    await loadCandidates();
  } catch (error) {
    toast.error(String(error));
  } finally {
    running.value = false;
  }
}

function fileTypeLabel(fileType: number) {
  return ({ 1: '图片', 2: '视频', 3: 'RAW' } as Record<number, string>)[fileType] || '素材';
}

function workflowLabel(value: string) {
  return ({ inbox: 'Inbox', reviewed: '已审核', selected: '已选用', archived: '已归档' } as Record<string, string>)[value] || value;
}

onMounted(loadCandidates);
</script>
