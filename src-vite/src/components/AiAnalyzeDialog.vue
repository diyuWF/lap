<template>
  <div class="fixed inset-0 z-[130] flex items-center justify-center bg-black/65 p-4" @mousedown.self="$emit('close')">
    <section class="flex max-h-[86vh] w-[760px] max-w-[96vw] flex-col overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <header class="flex items-start justify-between border-b border-base-content/10 px-5 py-4">
        <div class="min-w-0">
          <h2 class="font-semibold">{{ $t('dam_features.analyze.title') }}</h2>
          <p class="mt-1 truncate text-xs text-base-content/45" :title="file?.file_path || file?.name">{{ file?.name || $t('dam_features.analyze.current_asset') }}</p>
        </div>
        <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">{{ $t('dam_features.common.close') }}</button>
      </header>

      <main class="min-h-0 flex-1 overflow-y-auto p-5">
        <div v-if="loadingProviders" class="flex min-h-48 items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="!providers.length" class="rounded-box border border-warning/25 bg-warning/10 p-4 text-sm">
          {{ $t('dam_features.analyze.no_provider') }}
        </div>

        <template v-else>
          <div class="grid grid-cols-2 gap-4">
            <label class="form-control col-span-2 gap-1">
              <span class="text-xs text-base-content/60">{{ $t('dam_features.analyze.service') }}</span>
              <select v-model="providerId" class="select select-bordered select-sm">
                <option v-for="provider in providers" :key="provider.id" :value="provider.id">
                  {{ provider.name }} · {{ provider.model }}
                </option>
              </select>
            </label>

            <label class="flex items-center gap-2 rounded-box border border-base-content/10 bg-base-300/25 p-3 text-sm">
              <input v-model="forceApply" class="toggle toggle-primary toggle-sm" type="checkbox" />
              {{ $t('dam_features.analyze.force_apply') }}
            </label>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-3 text-sm">
              <div class="text-xs text-base-content/45">{{ $t('dam_features.analyze.scope') }}</div>
              <div class="mt-1">{{ $t('dam_features.analyze.scope_detail') }}</div>
            </div>
          </div>

          <div v-if="busy" class="mt-5 flex min-h-40 flex-col items-center justify-center gap-3 rounded-box border border-base-content/10 bg-base-300/20">
            <span class="loading loading-spinner loading-md"></span>
            <span class="text-sm text-base-content/55">{{ $t('dam_features.analyze.generating') }}</span>
          </div>

          <div v-else-if="result" class="mt-5 space-y-4">
            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-1 text-xs text-base-content/45">{{ $t('dam_features.analyze.suggested_title') }}</div>
              <div class="font-medium">{{ result.analysis?.title || '—' }}</div>
            </div>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-1 text-xs text-base-content/45">{{ $t('dam_features.analyze.description') }}</div>
              <p class="whitespace-pre-wrap text-sm leading-6">{{ result.analysis?.description || '—' }}</p>
            </div>

            <div class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-2 flex items-center justify-between">
                <span class="text-xs text-base-content/45">{{ $t('dam_features.analyze.tags') }}</span>
                <span class="text-xs tabular-nums text-base-content/45">{{ $t('dam_features.analyze.confidence', { value: confidenceLabel }) }}</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <span v-for="tag in result.analysis?.tags || []" :key="tag" class="badge badge-primary badge-outline">{{ tag }}</span>
                <span v-if="!(result.analysis?.tags || []).length" class="text-sm text-base-content/40">{{ $t('dam_features.analyze.no_tags') }}</span>
              </div>
            </div>

            <div v-if="result.analysis?.dominantColors?.length" class="rounded-box border border-base-content/10 bg-base-300/25 p-4">
              <div class="mb-2 text-xs text-base-content/45">{{ $t('dam_features.analyze.dominant_colors') }}</div>
              <div class="flex flex-wrap gap-2">
                <div v-for="color in result.analysis.dominantColors" :key="color" class="flex items-center gap-2 rounded-box bg-base-100/40 px-2 py-1 text-xs font-mono">
                  <span class="h-4 w-4 rounded border border-base-content/15" :style="{ backgroundColor: color }"></span>
                  {{ color }}
                </div>
              </div>
            </div>

            <div class="alert py-2 text-sm" :class="result.tagsApplied ? 'alert-success' : 'alert-info'">
              {{ result.tagsApplied
                ? $t('dam_features.analyze.tags_applied', { count: result.appliedTagIds?.length || 0 })
                : $t('dam_features.analyze.queued') }}
            </div>
          </div>
        </template>
      </main>

      <footer class="flex items-center justify-between border-t border-base-content/10 px-5 py-3">
        <span class="text-xs text-base-content/40">{{ $t('dam_features.analyze.api_key_privacy') }}</span>
        <button class="btn btn-primary btn-sm" type="button" :disabled="busy || loadingProviders || !providerId" @click="analyze">
          {{ busy ? $t('dam_features.analyze.analyzing') : result ? $t('dam_features.analyze.analyze_again') : $t('dam_features.analyze.start') }}
        </button>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { analyzeFileWithOnlineAi, listOnlineAiProviders } from '@/common/dam-api';
import { useToast } from '@/common/toast';

const props = defineProps<{
  file: Record<string, any> | null;
}>();

const emit = defineEmits(['close', 'done']);
const { t } = useI18n();
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
    toast.success(t('dam_features.analyze.complete'));
    emit('done', result.value);
  } catch (error) {
    toast.error(String(error));
  } finally {
    busy.value = false;
  }
}

onMounted(loadProviders);
</script>
