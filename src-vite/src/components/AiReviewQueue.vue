<template>
  <div class="fixed inset-0 z-[120] flex items-center justify-center bg-black/60 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[82vh] w-[1040px] max-w-[96vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-center justify-between border-b border-base-content/10 px-5 py-4">
        <div>
          <h2 class="font-semibold">{{ $t('dam_features.review.title') }}</h2>
          <p class="mt-0.5 text-xs text-base-content/45">{{ $t('dam_features.review.subtitle') }}</p>
        </div>
        <div class="flex items-center gap-2">
          <select v-model="status" class="select select-bordered select-sm" @change="load">
            <option value="pending">{{ $t('dam_features.review.pending') }}</option>
            <option value="accepted">{{ $t('dam_features.review.accepted') }}</option>
            <option value="rejected">{{ $t('dam_features.review.rejected') }}</option>
            <option value="applied">{{ $t('dam_features.review.applied') }}</option>
            <option value="all">{{ $t('dam_features.common.all') }}</option>
          </select>
          <button class="btn btn-ghost btn-sm" type="button" :disabled="loading" @click="load">{{ $t('dam_features.common.refresh') }}</button>
          <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">{{ $t('dam_features.common.close') }}</button>
        </div>
      </header>

      <div v-if="status === 'pending' && rows.length" class="flex items-center justify-between border-b border-base-content/10 bg-base-300/30 px-5 py-2">
        <label class="flex items-center gap-2 text-sm">
          <input v-model="selectAll" class="checkbox checkbox-sm" type="checkbox" @change="toggleAll" />
          {{ $t('dam_features.review.selected_count', { count: selectedIds.size }) }}
        </label>
        <div class="flex gap-2">
          <button class="btn btn-ghost btn-xs" type="button" :disabled="busy || !selectedIds.size" @click="reviewSelected('reject')">{{ $t('dam_features.review.batch_reject') }}</button>
          <button class="btn btn-primary btn-xs" type="button" :disabled="busy || !selectedIds.size" @click="reviewSelected('accept')">{{ $t('dam_features.review.batch_accept') }}</button>
        </div>
      </div>

      <main class="min-h-0 flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>
        <div v-else-if="!rows.length" class="flex h-full flex-col items-center justify-center text-base-content/45">
          <p class="text-sm">{{ $t('dam_features.review.empty') }}</p>
        </div>
        <div v-else class="space-y-2">
          <article
            v-for="row in rows"
            :key="row.id"
            class="grid grid-cols-[28px_minmax(160px,1.2fr)_100px_minmax(220px,2fr)_90px_150px] items-center gap-3 rounded-box border border-base-content/10 bg-base-300/25 px-3 py-3"
          >
            <input
              v-if="row.status === 'pending'"
              class="checkbox checkbox-sm"
              type="checkbox"
              :checked="selectedIds.has(row.id)"
              @change="toggleRow(row.id)"
            />
            <span v-else class="text-center text-xs text-base-content/30">—</span>

            <div class="min-w-0">
              <div class="truncate text-sm font-medium" :title="row.fileName">{{ row.fileName || $t('dam_features.common.file_id', { id: row.fileId }) }}</div>
              <div class="truncate text-[11px] text-base-content/40" :title="row.filePath">{{ row.filePath || $t('dam_features.review.path_unavailable') }}</div>
            </div>

            <span class="badge badge-ghost badge-sm justify-self-start">{{ kindLabel(row.kind) }}</span>

            <div class="min-w-0 break-words text-sm" :title="row.value">{{ row.value }}</div>

            <div class="text-xs tabular-nums text-base-content/55">
              {{ confidenceLabel(row.confidence) }}
            </div>

            <div v-if="row.status === 'pending'" class="flex justify-end gap-1">
              <button class="btn btn-ghost btn-xs" type="button" :disabled="busy" @click="reviewOne(row.id, 'reject')">{{ $t('dam_features.review.reject') }}</button>
              <button class="btn btn-primary btn-xs" type="button" :disabled="busy" @click="reviewOne(row.id, 'accept')">{{ $t('dam_features.review.accept') }}</button>
            </div>
            <div v-else class="text-right text-xs text-base-content/45">{{ statusLabel(row.status) }}</div>
          </article>
        </div>
      </main>

      <footer class="flex items-center justify-between border-t border-base-content/10 px-5 py-3 text-xs text-base-content/45">
        <span>{{ $t('dam_features.review.total', { count: rows.length }) }}</span>
        <button v-if="status !== 'pending'" class="btn btn-ghost btn-xs text-error" type="button" :disabled="busy" @click="clearReviewed">{{ $t('dam_features.review.clear') }}</button>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { clearReviewedAiSuggestions, listAiSuggestions, reviewAiSuggestion } from '@/common/ai-review-api';
import { useToast } from '@/common/toast';

const emit = defineEmits(['close']);
const { t } = useI18n();
const toast = useToast();
const status = ref('pending');
const rows = ref<any[]>([]);
const loading = ref(false);
const busy = ref(false);
const selectedIds = ref(new Set<number>());
const selectAll = ref(false);

async function load() {
  loading.value = true;
  selectedIds.value = new Set();
  selectAll.value = false;
  try {
    rows.value = await listAiSuggestions(status.value, 500);
  } catch (error) {
    toast.error(String(error));
  } finally {
    loading.value = false;
  }
}

function toggleRow(id: number) {
  const next = new Set(selectedIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selectedIds.value = next;
  selectAll.value = rows.value.filter((row) => row.status === 'pending').every((row) => next.has(row.id));
}

function toggleAll() {
  selectedIds.value = selectAll.value
    ? new Set(rows.value.filter((row) => row.status === 'pending').map((row) => row.id))
    : new Set();
}

async function reviewOne(id: number, action: 'accept' | 'reject') {
  busy.value = true;
  try {
    await reviewAiSuggestion(id, action);
    toast.success(action === 'accept'
      ? t('dam_features.review.accepted_toast')
      : t('dam_features.review.rejected_toast'));
    await load();
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

async function reviewSelected(action: 'accept' | 'reject') {
  if (!selectedIds.value.size) return;
  busy.value = true;
  let completed = 0;
  try {
    for (const id of selectedIds.value) {
      await reviewAiSuggestion(id, action);
      completed += 1;
    }
    toast.success(t('dam_features.review.processed', { count: completed }));
    await load();
  } catch (error) {
    toast.error(t('dam_features.review.failed_after', { count: completed, error: String(error) }));
    await load();
  } finally {
    busy.value = false;
  }
}

async function clearReviewed() {
  if (!confirm(t('dam_features.review.clear_confirm'))) return;
  busy.value = true;
  try {
    const count = await clearReviewedAiSuggestions();
    toast.success(t('dam_features.review.cleared', { count }));
    await load();
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

function kindLabel(kind: string) {
  return ({
    tag: t('dam_features.review.kind_tag'),
    title: t('dam_features.review.kind_title'),
    description: t('dam_features.review.kind_description'),
    color: t('dam_features.review.kind_color'),
  } as Record<string, string>)[kind] || kind;
}

function statusLabel(value: string) {
  return ({
    pending: t('dam_features.review.pending'),
    accepted: t('dam_features.review.accepted'),
    rejected: t('dam_features.review.rejected'),
    applied: t('dam_features.review.applied'),
  } as Record<string, string>)[value] || value;
}

function confidenceLabel(value: number | null) {
  return typeof value === 'number' ? `${Math.round(value * 100)}%` : '—';
}

onMounted(load);
</script>
