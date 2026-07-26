<template>
  <div class="fixed inset-0 z-[125] flex items-center justify-center bg-black/65 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[88vh] w-[1120px] max-w-[97vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-start justify-between border-b border-base-content/10 px-5 py-4">
        <div>
          <h2 class="font-semibold">{{ $t('dam_features.inbox.title') }}</h2>
          <p class="mt-1 text-xs text-base-content/45">{{ $t('dam_features.inbox.subtitle') }}</p>
        </div>
        <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">{{ $t('dam_features.common.close') }}</button>
      </header>

      <div class="grid grid-cols-[180px_1fr_180px_auto] items-end gap-3 border-b border-base-content/10 bg-base-300/20 px-5 py-3">
        <label class="form-control gap-1">
          <span class="text-xs text-base-content/55">{{ $t('dam_features.inbox.workflow_status') }}</span>
          <select v-model="workflowStatus" class="select select-bordered select-sm" @change="loadCandidates">
            <option value="inbox">{{ $t('dam_features.workflow.inbox') }}</option>
            <option value="reviewed">{{ $t('dam_features.workflow.reviewed') }}</option>
            <option value="selected">{{ $t('dam_features.workflow.selected') }}</option>
            <option value="archived">{{ $t('dam_features.workflow.archived') }}</option>
            <option value="all">{{ $t('dam_features.common.all') }}</option>
          </select>
        </label>

        <label class="form-control gap-1">
          <span class="text-xs text-base-content/55">{{ $t('dam_features.inbox.service') }}</span>
          <select v-model="providerId" class="select select-bordered select-sm" :disabled="!providers.length">
            <option v-for="provider in providers" :key="provider.id" :value="provider.id">
              {{ provider.name }} · {{ provider.model }}
            </option>
          </select>
        </label>

        <label class="flex h-8 items-center gap-2 text-sm">
          <input v-model="includeAnalyzed" class="toggle toggle-primary toggle-sm" type="checkbox" @change="loadCandidates" />
          {{ $t('dam_features.inbox.include_analyzed') }}
        </label>

        <button class="btn btn-ghost btn-sm" type="button" :disabled="loading || running" @click="loadCandidates">{{ $t('dam_features.inbox.refresh') }}</button>
      </div>

      <div class="flex items-center justify-between border-b border-base-content/10 px-5 py-2.5">
        <label class="flex items-center gap-2 text-sm">
          <input v-model="selectAll" class="checkbox checkbox-sm" type="checkbox" :disabled="!candidates.length || running" @change="toggleAll" />
          {{ $t('dam_features.inbox.selected_count', { selected: selectedIds.size, total: candidates.length }) }}
        </label>
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-2 text-sm">
            <input v-model="forceAutoApply" class="toggle toggle-primary toggle-sm" type="checkbox" :disabled="running" />
            {{ $t('dam_features.inbox.force_apply') }}
          </label>
          <button class="btn btn-primary btn-sm" type="button" :disabled="running || !selectedIds.size || !providerId" @click="runBatch">
            {{ running ? $t('dam_features.inbox.running') : $t('dam_features.inbox.analyze_count', { count: selectedIds.size }) }}
          </button>
        </div>
      </div>

      <main class="min-h-0 flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="!providers.length" class="flex h-full items-center justify-center">
          <div class="max-w-lg rounded-box border border-warning/25 bg-warning/10 p-5 text-center text-sm leading-6">
            {{ $t('dam_features.inbox.no_provider') }}
          </div>
        </div>

        <div v-else-if="!candidates.length" class="flex h-full flex-col items-center justify-center gap-2 text-base-content/45">
          <p class="text-sm">{{ $t('dam_features.inbox.empty') }}</p>
          <p class="text-xs">{{ $t('dam_features.inbox.empty_hint') }}</p>
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
              <span v-if="item.hasAiSuggestions" class="ml-2 text-warning">{{ $t('dam_features.inbox.has_suggestions') }}</span>
            </div>
          </article>
        </div>
      </main>

      <footer class="border-t border-base-content/10 px-5 py-3">
        <div v-if="running" class="flex items-center gap-3 text-sm text-base-content/55">
          <span class="loading loading-spinner loading-sm"></span>
          {{ $t('dam_features.inbox.sequential_hint') }}
        </div>
        <div v-else-if="batchResult" class="flex items-center justify-between gap-4 text-sm">
          <span>
            {{ $t('dam_features.inbox.batch_completed', { completed: batchResult.completed, total: batchResult.total }) }}
            {{ $t('dam_features.inbox.succeeded') }}
            <strong class="text-success">{{ batchResult.succeeded }}</strong>
            {{ $t('dam_features.inbox.failed') }}
            <strong :class="batchResult.failed ? 'text-error' : 'text-base-content/55'">{{ batchResult.failed }}</strong>
          </span>
          <div class="flex gap-2">
            <button v-if="batchResult.failed" class="btn btn-ghost btn-xs" type="button" @click="showFailures = !showFailures">
              {{ showFailures ? $t('dam_features.inbox.hide_failures') : $t('dam_features.inbox.show_failures') }}
            </button>
            <button class="btn btn-primary btn-xs" type="button" @click="$emit('open-review')">{{ $t('dam_features.inbox.open_review') }}</button>
          </div>
        </div>
        <div v-else class="text-xs text-base-content/40">{{ $t('dam_features.inbox.limit_hint') }}</div>

        <div v-if="showFailures && batchResult?.failures?.length" class="mt-3 max-h-32 overflow-y-auto rounded-box border border-error/20 bg-error/5 p-3 text-xs">
          <div v-for="failure in batchResult.failures" :key="failure.fileId" class="mb-1 last:mb-0">
            {{ $t('dam_features.common.file_id', { id: failure.fileId }) }}: {{ failure.error }}
          </div>
        </div>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  analyzeFilesWithOnlineAi,
  listOnlineAiBatchCandidates,
  listOnlineAiProviders,
} from '@/common/dam-api';
import { useToast } from '@/common/toast';

const emit = defineEmits(['close', 'open-review']);
const { t } = useI18n();
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
      toast.warning(t('dam_features.inbox.complete_with_failures', { count: batchResult.value.failed }));
    } else {
      toast.success(t('dam_features.inbox.complete_success', { count: batchResult.value.succeeded }));
    }
    await loadCandidates();
  } catch (error) {
    toast.error(String(error));
  } finally {
    running.value = false;
  }
}

function fileTypeLabel(fileType: number) {
  return ({
    1: t('dam_features.common.image'),
    2: t('dam_features.common.video'),
    3: 'RAW',
  } as Record<number, string>)[fileType] || t('dam_features.common.asset');
}

function workflowLabel(value: string) {
  return ({
    inbox: t('dam_features.workflow.inbox'),
    reviewed: t('dam_features.workflow.reviewed'),
    selected: t('dam_features.workflow.selected'),
    archived: t('dam_features.workflow.archived'),
  } as Record<string, string>)[value] || value;
}

onMounted(loadCandidates);
</script>
