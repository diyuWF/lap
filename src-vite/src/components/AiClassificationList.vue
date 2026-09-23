<template>
  <div class="sidebar-panel min-h-0">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ $t('album.smart_album_list') }}</span>
      <TButton
        :icon="IconRefresh"
        :buttonSize="'small'"
        :tooltip="$t('dam_features.inbox.refresh')"
        :disabled="loading"
        @click="refreshCount"
      />
    </div>

    <button
      class="sidebar-item sidebar-item-media sidebar-item-selected mx-1 border-0 text-left"
      type="button"
      @click="$emit('refresh-ai-classification')"
    >
      <span class="mr-2 grid h-10 w-10 shrink-0 place-items-center rounded-box bg-base-200 text-primary">
        <IconSparkles class="h-5 w-5" />
      </span>
      <span class="min-w-0 flex-1">
        <strong class="block truncate text-sm font-medium">{{ $t('dam_features.inbox.collection_title') }}</strong>
        <small class="block truncate text-[11px] text-base-content/45">{{ $t('dam_features.inbox.collection_hint') }}</small>
      </span>
      <span class="sidebar-item-count">{{ countLabel }}</span>
    </button>

    <div class="mx-3 mt-3 text-xs leading-5 text-base-content/60">
      {{ $t('dam_features.inbox.collection_description') }}
    </div>

    <button
      class="btn btn-primary btn-sm mx-2 mt-3 rounded-box"
      type="button"
      :disabled="loading || busy || candidateCount === 0"
      @click="$emit('run-ai-classification')"
    >
      <span v-if="loading || busy" class="loading loading-spinner loading-xs"></span>
      {{ busy ? $t('dam_features.inbox.one_click_running') : $t('dam_features.inbox.one_click_classify', { count: candidateCount }) }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { listOnlineAiBatchCandidates } from '@/common/dam-api';
import { IconRefresh, IconSparkles } from '@/common/icons';
import TButton from '@/components/TButton.vue';

defineProps({ busy: { type: Boolean, default: false } });
defineEmits(['run-ai-classification', 'refresh-ai-classification']);

const candidateCount = ref(0);
const loading = ref(false);
const countLabel = computed(() => candidateCount.value >= 200 ? '200+' : String(candidateCount.value));

async function refreshCount() {
  loading.value = true;
  try {
    const candidates = await listOnlineAiBatchCandidates('inbox', 200, false);
    candidateCount.value = Array.isArray(candidates) ? candidates.length : 0;
  } catch (error) {
    console.error('Failed to refresh the AI classification queue:', error);
  } finally {
    loading.value = false;
  }
}

defineExpose({ refreshCount });
onMounted(refreshCount);
</script>
