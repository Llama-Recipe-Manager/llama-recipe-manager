import { toast } from 'svelte-sonner';

import { copyText } from '$lib/utils/clipboard';
import { errorMessage } from '$lib/utils/format';

export function notifyError(e: unknown): void {
  toast.error(errorMessage(e));
}

export async function copyWithToast(text: string, label = 'Copied to clipboard'): Promise<void> {
  if (await copyText(text)) toast.success(label);
  else toast.error('Could not copy to clipboard.');
}
