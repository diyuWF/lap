<template>
  <Teleport to="body">
    <div class="lap-action-backdrop" @pointerdown.self="$emit('cancel')">
      <section
        class="lap-action-card"
        :class="`lap-action-${variant}`"
        role="dialog"
        aria-modal="true"
        aria-labelledby="lap-action-title"
        aria-describedby="lap-action-message"
        @keydown="handleKeydown"
      >
        <div class="lap-action-brand"><span class="lap-action-brand-dot"></span> LAP</div>
        <div class="lap-action-heading">
          <span class="lap-action-icon" aria-hidden="true">
            <svg v-if="variant === 'board'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H10l2 2h6.5A2.5 2.5 0 0 1 21 9.5v9A2.5 2.5 0 0 1 18.5 21h-13A2.5 2.5 0 0 1 3 18.5z" />
            </svg>
            <svg v-else-if="variant === 'ai'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="m12 2 1.9 6.1L20 10l-6.1 1.9L12 18l-1.9-6.1L4 10l6.1-1.9zM19 17l.7 2.3L22 20l-2.3.7L19 23l-.7-2.3L16 20l2.3-.7z" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 8v4m0 4h.01" />
            </svg>
          </span>
          <h2 id="lap-action-title">{{ title }}</h2>
        </div>
        <p id="lap-action-message" class="lap-action-message">{{ message }}</p>
        <div class="lap-action-buttons">
          <button ref="cancelButton" type="button" class="lap-action-secondary" @click="$emit('cancel')">{{ cancelLabel }}</button>
          <button type="button" class="lap-action-primary" @click="$emit('confirm')">{{ okLabel }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue';

withDefaults(defineProps<{
  title: string;
  message: string;
  okLabel: string;
  cancelLabel: string;
  variant?: 'default' | 'board' | 'ai' | 'danger';
}>(), { variant: 'default' });

const emit = defineEmits<{ confirm: []; cancel: [] }>();
const cancelButton = ref<HTMLButtonElement | null>(null);
onMounted(() => nextTick(() => cancelButton.value?.focus()));

function handleKeydown(event: KeyboardEvent) {
  event.stopPropagation();
  if (event.key === 'Escape') {
    event.preventDefault();
    emit('cancel');
  }
  if (event.key !== 'Tab') return;
  const buttons = Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('button'));
  if (event.shiftKey && document.activeElement === buttons[0]) {
    event.preventDefault();
    buttons[buttons.length - 1]?.focus();
  } else if (!event.shiftKey && document.activeElement === buttons[buttons.length - 1]) {
    event.preventDefault();
    buttons[0]?.focus();
  }
}
</script>

<style scoped>
.lap-action-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9990;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgba(23, 24, 29, 0.36);
  backdrop-filter: blur(8px);
}
.lap-action-card {
  box-sizing: border-box;
  width: min(390px, 100%);
  padding: 22px;
  border: 1px solid var(--lap-border, rgba(127, 127, 127, .2));
  border-radius: 24px;
  background: var(--lap-surface-raised, var(--color-base-100));
  color: var(--color-base-content);
  box-shadow: 0 24px 70px rgba(17, 17, 24, .22);
}
.lap-action-brand {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 19px;
  color: color-mix(in srgb, var(--color-base-content) 50%, transparent);
  font-size: 10px;
  font-weight: 750;
  letter-spacing: .18em;
}
.lap-action-brand-dot { width: 7px; height: 7px; border-radius: 50%; background: #a779d9; }
.lap-action-heading { display: flex; align-items: center; gap: 13px; }
.lap-action-icon {
  display: grid;
  flex: 0 0 43px;
  height: 43px;
  place-items: center;
  border-radius: 14px;
  background: color-mix(in srgb, #a779d9 14%, var(--lap-surface-soft, var(--color-base-200)));
  color: #875ab9;
}
.lap-action-icon svg { width: 22px; height: 22px; }
.lap-action-ai .lap-action-icon { color: #9055cb; }
.lap-action-danger .lap-action-icon { color: var(--color-error); background: color-mix(in srgb, var(--color-error) 12%, var(--lap-surface-soft)); }
.lap-action-heading h2 { margin: 0; font-size: 18px; font-weight: 650; letter-spacing: -.02em; line-height: 1.35; }
.lap-action-message { margin: 13px 0 24px 56px; font-size: 13px; line-height: 1.6; color: color-mix(in srgb, var(--color-base-content) 66%, transparent); overflow-wrap: anywhere; }
.lap-action-buttons { display: flex; justify-content: flex-end; gap: 9px; }
.lap-action-buttons button { min-height: 39px; padding: 0 16px; border-radius: 11px; font-size: 12px; font-weight: 600; cursor: pointer; transition: filter .15s, transform .15s; }
.lap-action-buttons button:hover { filter: brightness(1.09); }
.lap-action-buttons button:active { transform: scale(.98); }
.lap-action-buttons button:focus-visible { outline: 2px solid #a779d9; outline-offset: 2px; }
.lap-action-secondary { border: 1px solid var(--lap-border-strong, var(--lap-border)); background: var(--lap-surface-soft, var(--color-base-200)); color: var(--color-base-content); }
.lap-action-primary { border: 1px solid transparent; background: var(--color-primary); color: var(--color-primary-content); }
.lap-action-danger .lap-action-primary { background: var(--color-error); color: var(--color-error-content); }
@media (prefers-reduced-motion: reduce) { .lap-action-buttons button { transition: none; } }
</style>
