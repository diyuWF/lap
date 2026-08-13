<template>
  <div
    :class="props.embedded
      ? 'relative flex h-full min-h-0 w-full flex-1 p-2'
      : 'fixed inset-0 z-[125] flex items-center justify-center bg-black/65 p-4'"
    @mousedown.self="!props.embedded && $emit('close')"
  >
    <section
      :class="props.embedded
        ? 'flex h-full min-h-0 w-full flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-lg'
        : 'flex h-[91vh] w-[1180px] max-w-[97vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl'"
    >
      <header class="flex items-start justify-between border-b border-base-content/10 px-5 py-4">
        <div>
          <h2 class="font-semibold">{{ $t('dam_features.inbox.title') }}</h2>
          <p class="mt-1 text-xs text-base-content/45">{{ $t('dam_features.inbox.subtitle') }}</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            v-if="props.embedded"
            class="btn btn-ghost btn-sm"
            type="button"
            @click="showEmbeddedSettings = !showEmbeddedSettings"
          >
            {{ showEmbeddedSettings ? $t('dam_features.common.close') : $t('dam_features.inbox.organization_scope') }}
          </button>
          <button v-else class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">{{ $t('dam_features.common.close') }}</button>
        </div>
      </header>

      <section
        v-show="!props.embedded || showEmbeddedSettings"
        class="grid max-h-[48vh] grid-cols-1 gap-3 overflow-y-auto border-b border-base-content/10 bg-base-300/20 px-5 py-4 xl:grid-cols-[minmax(360px,1.4fr)_minmax(250px,1fr)_minmax(250px,1fr)]"
      >
        <div class="rounded-box border border-base-content/10 bg-base-100/45 p-3">
          <div class="mb-2 text-xs font-semibold text-base-content/65">{{ $t('dam_features.inbox.organization_scope') }}</div>
          <div class="grid grid-cols-2 gap-2">
            <button
              class="rounded-box border px-3 py-2 text-left transition"
              :class="organizationMode === 'within_folder' ? 'border-primary bg-primary/10 text-primary' : 'border-base-content/10 bg-base-200/55'"
              type="button"
              :disabled="running || executing"
              @click="organizationMode = 'within_folder'"
            >
              <strong class="block text-sm">{{ $t('dam_features.inbox.scope_folder') }}</strong>
              <span class="mt-0.5 block text-[11px] leading-4 text-base-content/45">{{ $t('dam_features.inbox.scope_folder_hint') }}</span>
            </button>
            <button
              class="rounded-box border px-3 py-2 text-left transition"
              :class="organizationMode === 'library' ? 'border-primary bg-primary/10 text-primary' : 'border-base-content/10 bg-base-200/55'"
              type="button"
              :disabled="running || executing"
              @click="organizationMode = 'library'"
            >
              <strong class="block text-sm">{{ $t('dam_features.inbox.scope_library') }}</strong>
              <span class="mt-0.5 block text-[11px] leading-4 text-base-content/45">{{ $t('dam_features.inbox.scope_library_hint') }}</span>
            </button>
          </div>
          <label v-if="organizationMode === 'within_folder'" class="form-control mt-3 gap-1">
            <span class="text-xs text-base-content/55">{{ $t('dam_features.inbox.root_folder') }}</span>
            <select v-model="rootFolderId" class="select select-bordered select-sm" :disabled="running || executing || !folderOptions.length">
              <option :value="null" disabled>{{ $t('dam_features.inbox.root_folder_placeholder') }}</option>
              <option v-for="folder in folderOptions" :key="folder.id" :value="folder.id">{{ folder.label }}</option>
            </select>
          </label>
        </div>

        <div class="rounded-box border border-base-content/10 bg-base-100/45 p-3">
          <label class="form-control gap-1">
            <span class="text-xs font-semibold text-base-content/65">{{ $t('dam_features.inbox.service') }}</span>
            <select v-model="providerId" class="select select-bordered select-sm" :disabled="!providers.length || running || executing">
              <option v-for="provider in providers" :key="provider.id" :value="provider.id">
                {{ provider.name }} · {{ provider.model }}
              </option>
            </select>
          </label>
          <label class="mt-3 flex items-start gap-2 text-sm">
            <input v-model="forceAutoApply" class="toggle toggle-primary toggle-sm mt-0.5" type="checkbox" :disabled="running || executing" />
            <span>
              <strong class="block font-medium">{{ $t('dam_features.inbox.force_apply') }}</strong>
              <small class="block text-[11px] leading-4 text-base-content/45">{{ $t('dam_features.inbox.force_apply_hint') }}</small>
            </span>
          </label>
        </div>

        <div class="rounded-box border border-primary/20 bg-primary/5 p-3 text-xs leading-5 text-base-content/60">
          <strong class="block text-sm text-base-content/75">{{ $t('dam_features.inbox.safety_title') }}</strong>
          <p class="mt-1">{{ $t('dam_features.inbox.safety_description') }}</p>
          <p class="mt-2 text-base-content/45">{{ $t('dam_features.inbox.folder_count', { count: folderOptions.length }) }}</p>
        </div>
      </section>

      <nav class="flex items-center justify-between border-b border-base-content/10 px-5">
        <div class="flex gap-1">
          <button class="border-b-2 px-3 py-3 text-sm" :class="activeTab === 'candidates' ? 'border-primary text-primary' : 'border-transparent text-base-content/45'" type="button" @click="activeTab = 'candidates'">
            {{ $t('dam_features.inbox.candidate_tab', { count: candidates.length }) }}
          </button>
          <button class="border-b-2 px-3 py-3 text-sm" :class="activeTab === 'plans' ? 'border-primary text-primary' : 'border-transparent text-base-content/45'" type="button" @click="activeTab = 'plans'">
            {{ $t('dam_features.inbox.plan_tab', { count: plans.length }) }}
          </button>
        </div>
        <button class="btn btn-ghost btn-xs" type="button" :disabled="loading || running || executing" @click="refreshAll">{{ $t('dam_features.inbox.refresh') }}</button>
      </nav>

      <div v-if="activeTab === 'candidates'" class="flex items-center justify-between border-b border-base-content/10 px-5 py-2.5">
        <div class="flex items-center gap-4">
          <label class="flex items-center gap-2 text-sm">
            <input v-model="selectAll" class="checkbox checkbox-sm" type="checkbox" :disabled="!candidates.length || running" @change="toggleAll" />
            {{ $t('dam_features.inbox.selected_count', { selected: selectedIds.size, total: candidates.length }) }}
          </label>
          <label class="flex items-center gap-2 text-xs text-base-content/55">
            <input v-model="includeAnalyzed" class="toggle toggle-sm" type="checkbox" :disabled="running" @change="loadCandidates" />
            {{ $t('dam_features.inbox.include_analyzed') }}
          </label>
        </div>
        <button class="btn btn-primary btn-sm" type="button" :disabled="!canAnalyze" @click="runBatch">
          {{ running ? $t('dam_features.inbox.running') : $t('dam_features.inbox.analyze_count', { count: selectedIds.size }) }}
        </button>
      </div>

      <div v-else class="flex items-center justify-between border-b border-base-content/10 px-5 py-2.5">
        <label class="flex items-center gap-2 text-sm">
          <input v-model="selectAllPlans" class="checkbox checkbox-sm" type="checkbox" :disabled="!actionablePlans.length || executing" @change="toggleAllPlans" />
          {{ $t('dam_features.inbox.selected_plan_count', { selected: selectedPlanIds.size, total: actionablePlans.length }) }}
        </label>
        <button class="btn btn-primary btn-sm" type="button" :disabled="executing || !selectedPlanIds.size" @click="executePlans">
          {{ executing ? $t('dam_features.inbox.executing') : $t('dam_features.inbox.execute_count', { count: selectedPlanIds.size }) }}
        </button>
      </div>

      <main class="min-h-0 flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>

        <template v-else-if="activeTab === 'candidates'">
          <div v-if="!providers.length" class="mb-3 rounded-box border border-warning/25 bg-warning/10 p-3 text-sm leading-6">
            {{ $t('dam_features.inbox.no_provider') }}
          </div>
          <div v-if="!candidates.length" class="flex h-full flex-col items-center justify-center gap-2 text-base-content/45">
            <p class="text-sm">{{ $t('dam_features.inbox.empty') }}</p>
            <p class="text-xs">{{ $t('dam_features.inbox.empty_hint') }}</p>
          </div>
          <div v-else class="space-y-2">
            <article
              v-for="item in candidates"
              :key="item.fileId"
              class="grid grid-cols-[32px_56px_minmax(0,1fr)_80px_100px] items-center gap-3 rounded-box border border-base-content/10 bg-base-300/25 px-3 py-2"
            >
              <input class="checkbox checkbox-sm" type="checkbox" :checked="selectedIds.has(item.fileId)" :disabled="running" @change="toggleItem(item.fileId)" />
              <div class="grid h-14 w-14 place-items-center overflow-hidden rounded-box border border-base-content/10 bg-base-100/35 text-base-content/25">
                <img
                  v-if="candidateThumbSrc(item)"
                  :src="candidateThumbSrc(item)"
                  class="h-full w-full object-cover"
                  alt=""
                />
                <IconPhoto v-else class="h-6 w-6" />
              </div>
              <div class="min-w-0">
                <div class="truncate text-sm font-medium" :title="item.fileName">{{ item.fileName }}</div>
                <div class="truncate text-[11px] text-base-content/40" :title="item.filePath">{{ item.filePath }}</div>
              </div>
              <span class="badge badge-ghost badge-sm justify-self-start">{{ fileTypeLabel(item.fileType) }}</span>
              <div class="text-right text-xs text-base-content/45">
                <span>{{ workflowLabel(item.workflowStatus) }}</span>
                <span v-if="item.hasAiSuggestions" class="ml-2 text-warning">{{ $t('dam_features.inbox.has_suggestions') }}</span>
              </div>
            </article>
          </div>
        </template>

        <template v-else>
          <div v-if="!plans.length" class="flex h-full flex-col items-center justify-center gap-2 text-base-content/45">
            <p class="text-sm">{{ $t('dam_features.inbox.no_plans') }}</p>
            <p class="text-xs">{{ $t('dam_features.inbox.no_plans_hint') }}</p>
          </div>
          <div v-else class="space-y-2">
            <article
              v-for="plan in plans"
              :key="plan.suggestionId"
              class="grid grid-cols-[32px_minmax(190px,1.2fr)_minmax(190px,1fr)_90px_minmax(180px,1.2fr)] items-center gap-3 rounded-box border px-3 py-3"
              :class="plan.targetAvailable ? 'border-base-content/10 bg-base-300/25' : 'border-error/25 bg-error/5'"
            >
              <input
                class="checkbox checkbox-sm"
                type="checkbox"
                :checked="selectedPlanIds.has(plan.suggestionId)"
                :disabled="executing || !plan.targetAvailable"
                @change="togglePlan(plan.suggestionId)"
              />
              <div class="min-w-0">
                <div class="truncate text-sm font-medium" :title="plan.fileName">{{ plan.fileName }}</div>
                <div class="truncate text-[11px] text-base-content/40" :title="plan.filePath">{{ plan.filePath }}</div>
              </div>
              <div class="min-w-0">
                <div class="text-[11px] text-base-content/40">{{ $t('dam_features.inbox.target_folder') }}</div>
                <div class="truncate text-sm font-medium text-primary" :title="plan.targetFolderPath">{{ plan.targetFolderPath }}</div>
                <div v-if="!plan.targetAvailable" class="text-[11px] text-error">{{ $t('dam_features.inbox.target_missing') }}</div>
              </div>
              <div class="text-xs tabular-nums text-base-content/55">{{ confidenceLabel(plan.confidence) }}</div>
              <div class="min-w-0 text-xs leading-5 text-base-content/55" :title="plan.reason">{{ plan.reason || $t('dam_features.inbox.no_reason') }}</div>
            </article>
          </div>
        </template>
      </main>

      <footer class="border-t border-base-content/10 px-5 py-3">
        <div v-if="running" class="flex items-center gap-3 text-sm text-base-content/55">
          <span class="loading loading-spinner loading-sm"></span>
          {{ $t('dam_features.inbox.sequential_hint') }}
        </div>
        <div v-else-if="executing" class="flex items-center gap-3 text-sm text-base-content/55">
          <span class="loading loading-spinner loading-sm"></span>
          {{ $t('dam_features.inbox.executing_hint') }}
        </div>
        <div v-else-if="executionResult" class="flex items-center justify-between gap-4 text-sm">
          <span>{{ $t('dam_features.inbox.execution_completed', { succeeded: executionResult.succeeded, failed: executionResult.failed }) }}</span>
          <button v-if="executionResult.failed" class="btn btn-ghost btn-xs" type="button" @click="showFailures = !showFailures">
            {{ showFailures ? $t('dam_features.inbox.hide_failures') : $t('dam_features.inbox.show_failures') }}
          </button>
        </div>
        <div v-else-if="batchResult" class="flex items-center justify-between gap-4 text-sm">
          <span>{{ $t('dam_features.inbox.batch_completed', { completed: batchResult.completed, total: batchResult.total }) }}
            {{ $t('dam_features.inbox.succeeded') }} <strong class="text-success">{{ batchResult.succeeded }}</strong>
            {{ $t('dam_features.inbox.failed') }} <strong :class="batchResult.failed ? 'text-error' : 'text-base-content/55'">{{ batchResult.failed }}</strong>
          </span>
          <button class="btn btn-ghost btn-xs" type="button" @click="$emit('open-review')">{{ $t('dam_features.inbox.open_review') }}</button>
        </div>
        <div v-else class="text-xs text-base-content/40">{{ $t('dam_features.inbox.limit_hint') }}</div>

        <div v-if="showFailures && failureRows.length" class="mt-3 max-h-28 overflow-y-auto rounded-box border border-error/20 bg-error/5 p-3 text-xs">
          <div v-for="failure in failureRows" :key="failure.fileId || failure.suggestionId" class="mb-1 last:mb-0">
            {{ failure.fileId ? $t('dam_features.common.file_id', { id: failure.fileId }) : $t('dam_features.inbox.plan_id', { id: failure.suggestionId }) }}: {{ failure.error }}
          </div>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  analyzeFilesWithOnlineAi,
  executeAiFolderSuggestions,
  listAiFolderSuggestions,
  listDamFolders,
  listOnlineAiBatchCandidates,
  listOnlineAiProviders,
} from '@/common/dam-api';
import { getFileThumbById } from '@/common/api';
import { config } from '@/common/config';
import { IconPhoto } from '@/common/icons';
import { useToast } from '@/common/toast';
import {
  getThumbUrl,
  getThumbnailDataUrl,
  getThumbnailDataUrlInflight,
  isWin,
  setThumbnailDataUrlInflight,
} from '@/common/utils';

const props = defineProps({
  embedded: {
    type: Boolean,
    default: false,
  },
});
const emit = defineEmits(['close', 'open-review', 'queue-updated']);
const { t } = useI18n();
const toast = useToast();
const providers = ref<any[]>([]);
const candidates = ref<any[]>([]);
const folders = ref<any[]>([]);
const folderOptions = ref<any[]>([]);
const plans = ref<any[]>([]);
const providerId = ref('');
const workflowStatus = ref('inbox');
const organizationMode = ref<'within_folder' | 'library'>('within_folder');
const rootFolderId = ref<number | null>(null);
const includeAnalyzed = ref(false);
const forceAutoApply = ref(false);
const loading = ref(false);
const running = ref(false);
const executing = ref(false);
const selectedIds = ref(new Set<number>());
const selectedPlanIds = ref(new Set<number>());
const selectAll = ref(false);
const selectAllPlans = ref(false);
const batchResult = ref<any>(null);
const executionResult = ref<any>(null);
const showFailures = ref(false);
const activeTab = ref<'candidates' | 'plans'>('candidates');
const showEmbeddedSettings = ref(false);
const candidateThumbUrls = ref<Record<number, string>>({});
let candidateThumbLoadToken = 0;

const actionablePlans = computed(() => plans.value.filter((plan) => plan.targetAvailable));
const canAnalyze = computed(() => (
  !running.value
  && !executing.value
  && selectedIds.value.size > 0
  && Boolean(providerId.value)
  && (organizationMode.value === 'library' || Boolean(rootFolderId.value))
));
const failureRows = computed(() => executionResult.value?.failures || batchResult.value?.failures || []);

function normalizePath(value: string) {
  return String(value || '').replace(/\\/g, '/').replace(/\/+$/, '');
}

function buildFolderOptions(items: any[]) {
  const records = items.map((folder) => ({
    ...folder,
    key: normalizePath(folder.path),
    children: [] as any[],
  })).filter((folder) => folder.key);
  const byKey = new Map(records.map((folder) => [folder.key.toLowerCase(), folder]));
  const roots: any[] = [];
  for (const folder of records) {
    const parts = folder.key.split('/');
    parts.pop();
    let parent = null;
    while (parts.length) {
      parent = byKey.get(parts.join('/').toLowerCase()) || null;
      if (parent) break;
      parts.pop();
    }
    if (parent) parent.children.push(folder);
    else roots.push(folder);
  }
  const result: any[] = [];
  const walk = (nodes: any[], parents: string[] = []) => {
    nodes.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN', { numeric: true }));
    for (const node of nodes) {
      const labels = [...parents, node.name];
      result.push({ ...node, label: labels.join(' / ') });
      walk(node.children, labels);
    }
  };
  walk(roots);
  return result;
}

async function loadProviders() {
  const allProviders = await listOnlineAiProviders();
  providers.value = allProviders.filter((provider: any) => provider.enabled && provider.hasApiKey);
  if (!providers.value.some((provider: any) => provider.id === providerId.value)) {
    providerId.value = providers.value[0]?.id || '';
  }
}

async function loadFolders() {
  folders.value = await listDamFolders();
  folderOptions.value = buildFolderOptions(folders.value)
    .filter((folder) => !['inbox', '待整理', '待整理区域'].includes(String(folder.name || '').trim().toLowerCase()));
  if (!folderOptions.value.some((folder) => folder.id === rootFolderId.value)) {
    rootFolderId.value = folderOptions.value[0]?.id || null;
  }
}

async function loadCandidates() {
  const nextCandidates = await listOnlineAiBatchCandidates(workflowStatus.value, 200, includeAnalyzed.value);
  candidates.value = Array.isArray(nextCandidates) ? nextCandidates : [];
  selectedIds.value = new Set(candidates.value.map((item) => Number(item.fileId)));
  selectAll.value = candidates.value.length > 0;
  emit('queue-updated', candidates.value.length);
  void loadWindowsCandidateThumbnails(candidates.value);
}

function candidateThumbSrc(item: any) {
  const fileId = Number(item?.fileId || 0);
  if (fileId <= 0) return '';
  return isWin
    ? candidateThumbUrls.value[fileId] || ''
    : getThumbUrl(fileId, false, Math.min(256, Number(config.settings.thumbnailSize || 256)));
}

async function loadWindowsCandidateThumbnails(items: any[]) {
  if (!isWin) return;
  const loadToken = ++candidateThumbLoadToken;
  const thumbnailSize = Math.min(256, Number(config.settings.thumbnailSize || 256));
  const limitedItems = items.slice(0, 80);
  const nextUrls: Record<number, string> = {};

  for (let offset = 0; offset < limitedItems.length; offset += 4) {
    const batch = limitedItems.slice(offset, offset + 4);
    const entries = await Promise.all(batch.map(async (item) => {
      const fileId = Number(item.fileId || 0);
      if (fileId <= 0) return [fileId, ''] as const;
      const inflight = getThumbnailDataUrlInflight(fileId, thumbnailSize);
      const dataUrl = await (inflight || setThumbnailDataUrlInflight(
        fileId,
        thumbnailSize,
        getFileThumbById(fileId, thumbnailSize, false)
          .then(thumb => getThumbnailDataUrl(thumb, '', false, thumbnailSize, item.filePath)),
      ));
      return [fileId, dataUrl || ''] as const;
    }));
    if (loadToken !== candidateThumbLoadToken) return;
    for (const [fileId, dataUrl] of entries) {
      if (fileId > 0 && dataUrl) nextUrls[fileId] = dataUrl;
    }
    candidateThumbUrls.value = { ...candidateThumbUrls.value, ...nextUrls };
  }
}

async function loadPlans() {
  plans.value = await listAiFolderSuggestions(500);
  selectedPlanIds.value = new Set();
  selectAllPlans.value = false;
}

async function refreshAll() {
  loading.value = true;
  try {
    await Promise.all([loadProviders(), loadFolders(), loadCandidates(), loadPlans()]);
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
  selectedIds.value = selectAll.value ? new Set(candidates.value.map((item) => item.fileId)) : new Set();
}

function togglePlan(suggestionId: number) {
  const next = new Set(selectedPlanIds.value);
  if (next.has(suggestionId)) next.delete(suggestionId);
  else next.add(suggestionId);
  selectedPlanIds.value = next;
  selectAllPlans.value = actionablePlans.value.length > 0
    && actionablePlans.value.every((plan) => next.has(plan.suggestionId));
}

function toggleAllPlans() {
  selectedPlanIds.value = selectAllPlans.value
    ? new Set(actionablePlans.value.map((plan) => plan.suggestionId))
    : new Set();
}

async function runBatch() {
  if (!canAnalyze.value) return;
  running.value = true;
  batchResult.value = null;
  executionResult.value = null;
  showFailures.value = false;
  try {
    batchResult.value = await analyzeFilesWithOnlineAi({
      fileIds: [...selectedIds.value],
      providerId: providerId.value,
      forceAutoApply: forceAutoApply.value,
      continueOnError: true,
      organizationMode: organizationMode.value,
      rootFolderId: organizationMode.value === 'within_folder' ? rootFolderId.value : null,
    });
    if (batchResult.value.failed) {
      toast.warning(t('dam_features.inbox.complete_with_failures', { count: batchResult.value.failed }));
    } else {
      toast.success(t('dam_features.inbox.plan_success', { count: batchResult.value.succeeded }));
    }
    await Promise.all([loadCandidates(), loadPlans()]);
    activeTab.value = 'plans';
  } catch (error) {
    toast.error(String(error));
  } finally {
    running.value = false;
  }
}

async function runAllCandidates() {
  if (loading.value) return;
  if (!candidates.value.length) {
    await loadCandidates();
  }
  if (!candidates.value.length) {
    toast.warning(t('dam_features.inbox.empty'));
    return;
  }
  if (!providerId.value) {
    toast.warning(t('dam_features.inbox.no_provider'));
    return;
  }
  selectedIds.value = new Set(candidates.value.map((item) => Number(item.fileId)));
  selectAll.value = true;
  await runBatch();
}

async function executePlans() {
  if (!selectedPlanIds.value.size) return;
  if (!confirm(t('dam_features.inbox.execute_confirm', { count: selectedPlanIds.value.size }))) return;
  executing.value = true;
  executionResult.value = null;
  batchResult.value = null;
  showFailures.value = false;
  try {
    executionResult.value = await executeAiFolderSuggestions([...selectedPlanIds.value]);
    if (executionResult.value.failed) {
      toast.warning(t('dam_features.inbox.execution_with_failures', { count: executionResult.value.failed }));
    } else {
      toast.success(t('dam_features.inbox.execution_success', { count: executionResult.value.succeeded }));
    }
    await Promise.all([loadCandidates(), loadPlans()]);
  } catch (error) {
    toast.error(String(error));
  } finally {
    executing.value = false;
  }
}

function fileTypeLabel(fileType: number) {
  return ({ 1: t('dam_features.common.image'), 2: t('dam_features.common.video'), 3: 'RAW' } as Record<number, string>)[fileType]
    || t('dam_features.common.asset');
}

function workflowLabel(value: string) {
  return ({
    inbox: t('dam_features.workflow.inbox'),
    reviewed: t('dam_features.workflow.reviewed'),
    selected: t('dam_features.workflow.selected'),
    archived: t('dam_features.workflow.archived'),
  } as Record<string, string>)[value] || value;
}

function confidenceLabel(value: number | null) {
  return typeof value === 'number' ? `${Math.round(value * 100)}%` : '—';
}

function handleWindowFocus() {
  if (!running.value && !executing.value) void refreshAll();
}

defineExpose({ refreshAll, runAllCandidates });

onMounted(() => {
  void refreshAll();
  window.addEventListener('focus', handleWindowFocus);
});

onBeforeUnmount(() => {
  candidateThumbLoadToken += 1;
  window.removeEventListener('focus', handleWindowFocus);
});
</script>
