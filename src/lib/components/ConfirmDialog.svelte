<script lang="ts">
  let {
    open,
    title,
    message,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    onConfirm,
    onCancel,
  }: {
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let focusedButton: 'confirm' | 'cancel' = $state('cancel');

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      const confirmBtn = document.querySelector('[data-confirm-btn]');
      const cancelBtn = document.querySelector('[data-cancel-btn]');
      if (!confirmBtn || !cancelBtn) return;
      const nodes = [cancelBtn, confirmBtn];
      const idx = nodes.indexOf(document.activeElement as HTMLElement);
      if (idx === -1) return;
      let nextIdx = e.shiftKey ? idx - 1 : idx + 1;
      if (nextIdx < 0) nextIdx = nodes.length - 1;
      if (nextIdx >= nodes.length) nextIdx = 0;
      (nodes[nextIdx] as HTMLElement).focus();
      e.preventDefault();
    } else if (e.key === 'Enter' && focusedButton === 'confirm') {
      onConfirm();
    } else if (e.key === 'Escape') {
      onCancel();
    }
  }

  $effect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => onKeyDown(e);
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  $effect(() => {
    if (open) {
      focusedButton = 'cancel';
      setTimeout(() => {
        (document.querySelector('[data-cancel-btn]') as HTMLElement)?.focus();
      }, 50);
    }
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={onCancel}></div>
  <div
    class="dialog"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="confirm-title"
    aria-describedby="confirm-message"
    tabindex="-1"
    onkeydown={onKeyDown}
  >
    <h2 id="confirm-title" class="title">{title}</h2>
    <p id="confirm-message" class="message">{message}</p>
    <div class="actions">
      <button
        class="btn cancel"
        data-cancel-btn
        onclick={onCancel}
        onmouseenter={() => (focusedButton = 'cancel')}
      >
        {cancelLabel}
      </button>
      <button
        class="btn danger"
        data-confirm-btn
        onclick={onConfirm}
        onmouseenter={() => (focusedButton = 'confirm')}
      >
        {confirmLabel}
      </button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    z-index: 100;
    animation: fade-in 0.12s ease-out;
  }

  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(400px, calc(100vw - 32px));
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg, 12px);
    padding: 20px 22px 18px;
    z-index: 101;
    box-shadow:
      0 18px 48px rgba(0, 0, 0, 0.32),
      0 4px 12px rgba(0, 0, 0, 0.18);
    animation: pop-in 0.14s ease-out;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translate(-50%, -48%) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
    }
  }

  .title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 10px;
  }

  .message {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0 0 18px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 18px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    cursor: pointer;
    border: none;
    white-space: nowrap;
  }

  .btn.cancel {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.cancel:hover {
    background: var(--bg-quaternary, var(--bg-tertiary));
  }

  .btn.danger {
    background: var(--danger);
    color: white;
  }

  .btn.danger:hover {
    background: var(--danger-hover);
  }
</style>
