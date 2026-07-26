<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/55 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[78vh] w-[880px] max-w-[96vw] overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <aside class="w-64 shrink-0 border-r border-base-content/10 bg-base-300/60 p-3">
        <div class="mb-3 flex items-start justify-between gap-2">
          <div>
            <h2 class="font-semibold">{{ $t('online_ai.title') }}</h2>
            <p class="text-xs text-base-content/45">{{ $t('online_ai.subtitle') }}</p>
          </div>
          <button class="btn btn-ghost btn-xs" type="button" @click="$emit('close')">{{ $t('online_ai.close') }}</button>
        </div>
        <button class="btn btn-primary btn-sm mb-3 w-full" type="button" @click="newProvider">{{ $t('online_ai.add_service') }}</button>
        <div class="space-y-1 overflow-y-auto">
          <button
            v-for="provider in providers"
            :key="provider.id"
            type="button"
            class="w-full rounded-box px-3 py-2 text-left"
            :class="selectedId === provider.id ? 'bg-primary text-primary-content' : 'hover:bg-base-100/50'"
            @click="selectProvider(provider)"
          >
            <span class="block truncate text-sm font-medium">{{ provider.name }}</span>
            <span class="block truncate text-[11px] opacity-60">{{ provider.model }}</span>
          </button>
        </div>
        <div v-if="!providers.length && !loading" class="mt-8 text-center text-xs text-base-content/40">{{ $t('online_ai.empty') }}</div>
      </aside>

      <main class="min-w-0 flex-1 overflow-y-auto p-5">
        <div class="mb-5">
          <h3 class="text-lg font-semibold">{{ form.id ? $t('online_ai.edit_service') : $t('online_ai.add_service') }}</h3>
          <p class="text-xs text-base-content/45">{{ $t('online_ai.description') }}</p>
        </div>

        <form class="grid grid-cols-2 gap-4" @submit.prevent="save">
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.service_name') }}</span>
            <input v-model="form.name" class="input input-bordered input-sm" required :placeholder="$t('online_ai.service_name_placeholder')" />
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.interface_type') }}</span>
            <select v-model="form.kind" class="select select-bordered select-sm">
              <option value="openai_compatible">{{ $t('online_ai.openai_compatible') }}</option>
              <option value="gemini">Google Gemini</option>
              <option value="anthropic">Anthropic</option>
            </select>
          </label>

          <label class="form-control col-span-2 gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.api_address') }}</span>
            <input v-model="form.baseUrl" class="input input-bordered input-sm font-mono" required :placeholder="baseUrlPlaceholder" />
            <span v-if="isOpenRouter" class="text-[11px] text-base-content/50">
              {{ $t('online_ai.openrouter_route') }}
              <span class="font-mono">https://openrouter.ai/api/v1/chat/completions</span>
            </span>
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.model') }}</span>
            <input v-model="form.model" class="input input-bordered input-sm font-mono" required :placeholder="$t('online_ai.model_placeholder')" />
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.api_key') }}</span>
            <input v-model="form.apiKey" class="input input-bordered input-sm font-mono" type="password" :placeholder="form.hasApiKey ? $t('online_ai.keep_api_key') : $t('online_ai.enter_api_key')" />
          </label>

          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.min_confidence') }}</span>
            <input v-model.number="form.minConfidence" class="range range-primary range-sm" type="range" min="0" max="1" step="0.05" />
            <span class="text-right text-xs text-base-content/45">{{ Math.round(form.minConfidence * 100) }}%</span>
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.max_tags') }}</span>
            <input v-model.number="form.maxTags" class="input input-bordered input-sm" type="number" min="1" max="50" />
          </label>

          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.output_language') }}</span>
            <select v-model="form.language" class="select select-bordered select-sm">
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.status') }}</span>
            <label class="flex h-8 items-center gap-2">
              <input v-model="form.enabled" class="toggle toggle-primary toggle-sm" type="checkbox" />
              <span class="text-sm">{{ $t('online_ai.enabled') }}</span>
            </label>
          </label>

          <div class="col-span-2 grid grid-cols-2 gap-3 rounded-box border border-base-content/10 bg-base-300/30 p-3">
            <label class="flex items-center gap-2 text-sm">
              <input v-model="form.autoApplyTags" class="toggle toggle-primary toggle-sm" type="checkbox" />
              {{ $t('online_ai.auto_apply_tags') }}
            </label>
            <label class="flex items-center gap-2 text-sm">
              <input v-model="form.autoMarkReviewed" class="toggle toggle-primary toggle-sm" type="checkbox" />
              {{ $t('online_ai.auto_mark_reviewed') }}
            </label>
          </div>

          <label class="form-control col-span-2 gap-1">
            <span class="text-xs text-base-content/60">{{ $t('online_ai.system_prompt') }}</span>
            <textarea v-model="form.systemPrompt" class="textarea textarea-bordered min-h-28" :placeholder="$t('online_ai.system_prompt_placeholder')"></textarea>
          </label>

          <div v-if="message" class="alert col-span-2 max-h-40 overflow-y-auto whitespace-pre-wrap break-words py-2 text-sm" :class="messageType === 'error' ? 'alert-error' : 'alert-success'">
            {{ message }}
          </div>

          <div class="col-span-2 flex items-center justify-between pt-2">
            <button v-if="form.id" class="btn btn-ghost btn-sm text-error" type="button" @click="remove">{{ $t('online_ai.delete_service') }}</button>
            <span v-else></span>
            <div class="flex gap-2">
              <button class="btn btn-ghost btn-sm" type="button" :disabled="busy || !form.id" @click="test">{{ $t('online_ai.test_connection') }}</button>
              <button class="btn btn-primary btn-sm" type="submit" :disabled="busy">{{ busy ? $t('online_ai.processing') : $t('online_ai.save_service') }}</button>
            </div>
          </div>
        </form>
      </main>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  deleteOnlineAiProvider,
  listOnlineAiProviders,
  saveOnlineAiProvider,
  testOnlineAiProvider,
} from '@/common/dam-api';

const emit = defineEmits(['close']);
const { t } = useI18n();
const providers = ref<any[]>([]);
const selectedId = ref<string | null>(null);
const loading = ref(true);
const busy = ref(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');
const form = reactive(defaultForm());

function defaultForm() {
  return {
    id: null as string | null,
    name: '',
    kind: 'openai_compatible',
    baseUrl: 'https://api.openai.com/v1',
    model: '',
    apiKey: '',
    hasApiKey: false,
    enabled: true,
    autoApplyTags: false,
    autoMarkReviewed: false,
    minConfidence: 0.75,
    maxTags: 12,
    language: 'zh-CN',
    systemPrompt: '',
    authHeader: 'Authorization',
    authPrefix: 'Bearer ',
    extraHeaders: {},
  };
}

const baseUrlPlaceholder = computed(() => {
  if (form.kind === 'gemini') return 'https://generativelanguage.googleapis.com/v1beta';
  if (form.kind === 'anthropic') return 'https://api.anthropic.com/v1';
  return 'https://api.openai.com/v1';
});

const isOpenRouter = computed(() => {
  if (form.kind !== 'openai_compatible') return false;
  try {
    return new URL(form.baseUrl).hostname.toLowerCase() === 'openrouter.ai';
  } catch {
    return false;
  }
});

async function load() {
  loading.value = true;
  try {
    providers.value = await listOnlineAiProviders();
    if (selectedId.value) {
      const selected = providers.value.find((provider) => provider.id === selectedId.value);
      if (selected) selectProvider(selected);
    }
  } finally {
    loading.value = false;
  }
}

function newProvider() {
  Object.assign(form, defaultForm());
  selectedId.value = null;
  message.value = '';
}

function selectProvider(provider: any) {
  selectedId.value = provider.id;
  Object.assign(form, {
    ...defaultForm(),
    ...provider,
    apiKey: '',
    hasApiKey: provider.hasApiKey,
  });
  message.value = '';
}

async function save() {
  busy.value = true;
  message.value = '';
  try {
    const saved = await saveOnlineAiProvider({
      id: form.id,
      name: form.name,
      kind: form.kind,
      baseUrl: form.baseUrl,
      model: form.model,
      apiKey: form.apiKey || null,
      enabled: form.enabled,
      autoApplyTags: form.autoApplyTags,
      autoMarkReviewed: form.autoMarkReviewed,
      minConfidence: form.minConfidence,
      maxTags: form.maxTags,
      language: form.language,
      systemPrompt: form.systemPrompt || null,
      authHeader: form.kind === 'openai_compatible' ? form.authHeader : '',
      authPrefix: form.kind === 'openai_compatible' ? form.authPrefix : '',
      extraHeaders: form.extraHeaders,
    });
    selectedId.value = saved.id;
    await load();
    messageType.value = 'success';
    message.value = t('online_ai.saved');
  } catch (error: any) {
    messageType.value = 'error';
    message.value = String(error);
  } finally {
    busy.value = false;
  }
}

async function test() {
  if (!form.id) return;
  busy.value = true;
  message.value = '';
  try {
    const result = await testOnlineAiProvider(form.id);
    messageType.value = 'success';
    message.value = t('online_ai.connection_success', { elapsed: result.elapsedMs });
  } catch (error: any) {
    messageType.value = 'error';
    message.value = String(error);
  } finally {
    busy.value = false;
  }
}

async function remove() {
  if (!form.id || !confirm(t('online_ai.delete_confirm', { name: form.name }))) return;
  await deleteOnlineAiProvider(form.id);
  newProvider();
  await load();
}

onMounted(load);
</script>
