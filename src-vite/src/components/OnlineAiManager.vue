<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/55 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[78vh] w-[880px] max-w-[96vw] overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <aside class="w-64 shrink-0 border-r border-base-content/10 bg-base-300/60 p-3">
        <div class="mb-3 flex items-start justify-between gap-2">
          <div>
            <h2 class="font-semibold">在线 AI 自动整理</h2>
            <p class="text-xs text-base-content/45">自带 API Key，无厂商锁定</p>
          </div>
          <button class="btn btn-ghost btn-xs" type="button" @click="$emit('close')">关闭</button>
        </div>
        <button class="btn btn-primary btn-sm mb-3 w-full" type="button" @click="newProvider">添加 AI 服务</button>
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
        <div v-if="!providers.length && !loading" class="mt-8 text-center text-xs text-base-content/40">尚未配置在线 AI 服务</div>
      </aside>

      <main class="min-w-0 flex-1 overflow-y-auto p-5">
        <div class="mb-5">
          <h3 class="text-lg font-semibold">{{ form.id ? '编辑 AI 服务' : '添加 AI 服务' }}</h3>
          <p class="text-xs text-base-content/45">支持 OpenAI 兼容接口、Gemini 和 Anthropic。API Key 仅保存在本机应用数据目录。</p>
        </div>

        <form class="grid grid-cols-2 gap-4" @submit.prevent="save">
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">服务名称</span>
            <input v-model="form.name" class="input input-bordered input-sm" required placeholder="例如 OpenRouter" />
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">接口类型</span>
            <select v-model="form.kind" class="select select-bordered select-sm">
              <option value="openai_compatible">OpenAI 兼容</option>
              <option value="gemini">Google Gemini</option>
              <option value="anthropic">Anthropic</option>
            </select>
          </label>

          <label class="form-control col-span-2 gap-1">
            <span class="text-xs text-base-content/60">API 地址</span>
            <input v-model="form.baseUrl" class="input input-bordered input-sm font-mono" required :placeholder="baseUrlPlaceholder" />
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">模型</span>
            <input v-model="form.model" class="input input-bordered input-sm font-mono" required placeholder="模型标识" />
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">API Key</span>
            <input v-model="form.apiKey" class="input input-bordered input-sm font-mono" type="password" :placeholder="form.hasApiKey ? '留空则保留现有 Key' : '输入 API Key'" />
          </label>

          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">最低自动应用置信度</span>
            <input v-model.number="form.minConfidence" class="range range-primary range-sm" type="range" min="0" max="1" step="0.05" />
            <span class="text-right text-xs text-base-content/45">{{ Math.round(form.minConfidence * 100) }}%</span>
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">最大标签数</span>
            <input v-model.number="form.maxTags" class="input input-bordered input-sm" type="number" min="1" max="50" />
          </label>

          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">输出语言</span>
            <select v-model="form.language" class="select select-bordered select-sm">
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </label>
          <label class="form-control gap-1">
            <span class="text-xs text-base-content/60">状态</span>
            <label class="flex h-8 items-center gap-2">
              <input v-model="form.enabled" class="toggle toggle-primary toggle-sm" type="checkbox" />
              <span class="text-sm">启用该服务</span>
            </label>
          </label>

          <div class="col-span-2 grid grid-cols-2 gap-3 rounded-box border border-base-content/10 bg-base-300/30 p-3">
            <label class="flex items-center gap-2 text-sm">
              <input v-model="form.autoApplyTags" class="toggle toggle-primary toggle-sm" type="checkbox" />
              达到阈值后自动应用标签
            </label>
            <label class="flex items-center gap-2 text-sm">
              <input v-model="form.autoMarkReviewed" class="toggle toggle-primary toggle-sm" type="checkbox" />
              自动应用后标记为已整理
            </label>
          </div>

          <label class="form-control col-span-2 gap-1">
            <span class="text-xs text-base-content/60">系统提示词</span>
            <textarea v-model="form.systemPrompt" class="textarea textarea-bordered min-h-28" placeholder="留空使用内置素材分类提示词"></textarea>
          </label>

          <div v-if="message" class="alert col-span-2 py-2 text-sm" :class="messageType === 'error' ? 'alert-error' : 'alert-success'">{{ message }}</div>

          <div class="col-span-2 flex items-center justify-between pt-2">
            <button v-if="form.id" class="btn btn-ghost btn-sm text-error" type="button" @click="remove">删除服务</button>
            <span v-else></span>
            <div class="flex gap-2">
              <button class="btn btn-ghost btn-sm" type="button" :disabled="busy || !form.id" @click="test">测试连接</button>
              <button class="btn btn-primary btn-sm" type="submit" :disabled="busy">{{ busy ? '处理中…' : '保存服务' }}</button>
            </div>
          </div>
        </form>
      </main>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import {
  deleteOnlineAiProvider,
  listOnlineAiProviders,
  saveOnlineAiProvider,
  testOnlineAiProvider,
} from '@/common/dam-api';

const emit = defineEmits(['close']);
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
    message.value = '在线 AI 服务已保存。';
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
    message.value = `${result.message}（${result.elapsedMs} ms）`;
  } catch (error: any) {
    messageType.value = 'error';
    message.value = String(error);
  } finally {
    busy.value = false;
  }
}

async function remove() {
  if (!form.id || !confirm(`删除在线 AI 服务“${form.name}”？`)) return;
  await deleteOnlineAiProvider(form.id);
  newProvider();
  await load();
}

onMounted(load);
</script>
