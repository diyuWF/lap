<template>
  <div class="fixed inset-0 z-[120] flex items-center justify-center bg-black/60 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[82vh] w-[1040px] max-w-[96vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-center justify-between border-b border-base-content/10 px-5 py-4">
        <div>
          <h2 class="font-semibold">AI 建议审核</h2>
          <p class="mt-0.5 text-xs text-base-content/45">只有接受后的标签才会进入正式素材库。</p>
        </div>
        <div class="flex items-center gap-2">
          <select v-model="status" class="select select-bordered select-sm" @change="load">
            <option value="pending">待审核</option>
            <option value="accepted">已接受</option>
            <option value="rejected">已拒绝</option>
            <option value="applied">已自动应用</option>
            <option value="all">全部</option>
          </select>
          <button class="btn btn-ghost btn-sm" type="button" :disabled="loading" @click="load">刷新</button>
          <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">关闭</button>
        </div>
      </header>

      <div v-if="status === 'pending' && rows.length" class="flex items-center justify-between border-b border-base-content/10 bg-base-300/30 px-5 py-2">
        <label class="flex items-center gap-2 text-sm">
          <input v-model="selectAll" class="checkbox checkbox-sm" type="checkbox" @change="toggleAll" />
          已选择 {{ selectedIds.size }} 条
        </label>
        <div class="flex gap-2">
          <button class="btn btn-ghost btn-xs" type="button" :disabled="busy || !selectedIds.size" @click="reviewSelected('reject')">批量拒绝</button>
          <button class="btn btn-primary btn-xs" type="button" :disabled="busy || !selectedIds.size" @click="reviewSelected('accept')">批量接受</button>
        </div>
      </div>

      <main class="min-h-0 flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>
        <div v-else-if="!rows.length" class="flex h-full flex-col items-center justify-center text-base-content/45">
          <p class="text-sm">当前没有 AI 建议</p>
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
              <div class="truncate text-sm font-medium" :title="row.fileName">{{ row.fileName || `文件 ${row.fileId}` }}</div>
              <div class="truncate text-[11px] text-base-content/40" :title="row.filePath">{{ row.filePath || '路径不可用' }}</div>
            </div>

            <span class="badge badge-ghost badge-sm justify-self-start">{{ kindLabel(row.kind) }}</span>

            <div class="min-w-0 break-words text-sm" :title="row.value">{{ row.value }}</div>

            <div class="text-xs tabular-nums text-base-content/55">
              {{ confidenceLabel(row.confidence) }}
            </div>

            <div v-if="row.status === 'pending'" class="flex justify-end gap-1">
              <button class="btn btn-ghost btn-xs" type="button" :disabled="busy" @click="reviewOne(row.id, 'reject')">拒绝</button>
              <button class="btn btn-primary btn-xs" type="button" :disabled="busy" @click="reviewOne(row.id, 'accept')">接受</button>
            </div>
            <div v-else class="text-right text-xs text-base-content/45">{{ statusLabel(row.status) }}</div>
          </article>
        </div>
      </main>

      <footer class="flex items-center justify-between border-t border-base-content/10 px-5 py-3 text-xs text-base-content/45">
        <span>共 {{ rows.length }} 条</span>
        <button v-if="status !== 'pending'" class="btn btn-ghost btn-xs text-error" type="button" :disabled="busy" @click="clearReviewed">清理已处理记录</button>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { clearReviewedAiSuggestions, listAiSuggestions, reviewAiSuggestion } from '@/common/ai-review-api';
import { useToast } from '@/common/toast';

const emit = defineEmits(['close']);
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
    toast.success(action === 'accept' ? '已接受 AI 建议' : '已拒绝 AI 建议');
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
    toast.success(`已处理 ${completed} 条 AI 建议`);
    await load();
  } catch (error) {
    toast.error(`已处理 ${completed} 条，随后失败：${String(error)}`);
    await load();
  } finally {
    busy.value = false;
  }
}

async function clearReviewed() {
  if (!confirm('清理所有已经接受、拒绝或自动应用的 AI 建议记录？')) return;
  busy.value = true;
  try {
    const count = await clearReviewedAiSuggestions();
    toast.success(`已清理 ${count} 条记录`);
    await load();
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

function kindLabel(kind: string) {
  return ({ tag: '标签', title: '标题', description: '描述', color: '颜色' } as Record<string, string>)[kind] || kind;
}

function statusLabel(value: string) {
  return ({ pending: '待审核', accepted: '已接受', rejected: '已拒绝', applied: '已自动应用' } as Record<string, string>)[value] || value;
}

function confidenceLabel(value: number | null) {
  return typeof value === 'number' ? `${Math.round(value * 100)}%` : '—';
}

onMounted(load);
</script>
