export interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

class ConfirmStore {
  open = $state(false);
  options = $state<ConfirmOptions>({ title: '', message: '' });
  private resolve: ((ok: boolean) => void) | null = null;

  ask(options: ConfirmOptions): Promise<boolean> {
    this.resolve?.(false);
    this.options = options;
    this.open = true;
    return new Promise((resolve) => (this.resolve = resolve));
  }

  settle(ok: boolean): void {
    this.open = false;
    this.resolve?.(ok);
    this.resolve = null;
  }
}

export const confirmStore = new ConfirmStore();
export const confirm = (options: ConfirmOptions) => confirmStore.ask(options);
