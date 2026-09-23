import { createVNode, render } from 'vue';
import ActionPrompt from '@/components/ActionPrompt.vue';

interface ActionOptions {
  title: string;
  okLabel: string;
  cancelLabel: string;
  kind?: 'info' | 'warning';
  variant?: 'default' | 'board' | 'ai' | 'danger';
}

/** A silent confirmation that uses the current Lap theme rather than a system dialog. */
export function confirmAction(message: string, options: ActionOptions): Promise<boolean> {
  const host = document.createElement('div');
  const previousFocus = document.activeElement;
  document.body.append(host);

  return new Promise(resolve => {
    let settled = false;
    const finish = (accepted: boolean) => {
      if (settled) return;
      settled = true;
      render(null, host);
      host.remove();
      if (previousFocus instanceof HTMLElement && previousFocus.isConnected) previousFocus.focus();
      resolve(accepted);
    };
    render(createVNode(ActionPrompt, {
      title: options.title,
      message,
      okLabel: options.okLabel,
      cancelLabel: options.cancelLabel,
      variant: options.variant || (options.kind === 'warning' ? 'danger' : 'default'),
      onConfirm: () => finish(true),
      onCancel: () => finish(false),
    }), host);
  });
}
