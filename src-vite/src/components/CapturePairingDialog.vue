<template>
  <dialog ref="dialog" class="modal" aria-labelledby="capture-pairing-title" @cancel.prevent="decide(false)">
    <section class="modal-box max-w-md rounded-2xl border border-base-content/10 p-7">
      <p class="text-xs font-semibold text-primary">{{ $t('capturePairing.eyebrow') }}</p>
      <h2 id="capture-pairing-title" class="mt-2 text-xl font-semibold">{{ $t('capturePairing.title') }}</h2>
      <p class="mt-3 text-sm leading-6 text-base-content/70">{{ $t('capturePairing.hint') }}</p>
      <div class="my-5 rounded-xl bg-base-200 px-5 py-6 text-center">
        <p class="text-xs text-base-content/60">{{ $t('capturePairing.code') }}</p>
        <p class="mt-2 font-mono text-4xl font-semibold tracking-[0.18em]">{{ pending?.code }}</p>
        <p class="mt-3 text-xs text-base-content/60">{{ $t('capturePairing.expires', { seconds: secondsLeft }) }}</p>
      </div>
      <p class="text-xs leading-5 text-base-content/60">{{ $t('capturePairing.permission') }}</p>
      <details class="mt-3 text-xs text-base-content/60">
        <summary class="cursor-pointer">{{ $t('capturePairing.extension') }}</summary>
        <code class="mt-2 block break-all">{{ pending?.extensionId }}</code>
      </details>
      <p v-if="error" class="mt-3 text-sm text-error" role="alert">{{ error }}</p>
      <div class="modal-action">
        <button class="btn btn-ghost" :disabled="busy" autofocus @click="decide(false)">{{ $t('capturePairing.reject') }}</button>
        <button class="btn btn-primary" :disabled="busy || !secondsLeft" @click="decide(true)">{{ $t('capturePairing.approve') }}</button>
      </div>
    </section>
  </dialog>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const dialog = ref(null);
const pending = ref(null);
const secondsLeft = ref(0);
const busy = ref(false);
const error = ref('');
let unlisten;
let timer;
let disposed = false;

async function refresh() {
  try {
    const value = await invoke('get_capture_pairing');
    if (disposed || !value || pending.value?.requestId === value.requestId) return;
    pending.value = value;
    error.value = '';
    const deadline = Date.now() + value.secondsLeft * 1000;
    secondsLeft.value = value.secondsLeft;
    clearInterval(timer);
    timer = setInterval(() => {
      secondsLeft.value = Math.max(0, Math.ceil((deadline - Date.now()) / 1000));
      if (!secondsLeft.value) close();
    }, 1000);
    await nextTick();
    if (!disposed && !dialog.value?.open) dialog.value?.showModal();
  } catch {
    // Desktop capture may be unavailable; settings exposes its service status.
  }
}
function close() {
  clearInterval(timer);
  dialog.value?.close();
  pending.value = null;
}
async function decide(approve) {
  if (busy.value || !pending.value) return;
  busy.value = true;
  try {
    await invoke('decide_capture_pairing', { requestId: pending.value.requestId, approve });
    close();
  } catch {
    error.value = t('capturePairing.failed');
  } finally { busy.value = false; }
}
onMounted(async () => {
  unlisten = await listen('lap-capture-pairing', refresh);
  if (disposed) { unlisten(); return; }
  await refresh();
});
onUnmounted(() => { disposed = true; unlisten?.(); clearInterval(timer); });
</script>
