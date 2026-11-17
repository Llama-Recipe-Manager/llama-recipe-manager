<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import Copy from '@lucide/svelte/icons/copy';
  import { onDestroy } from 'svelte';

  import { Button } from '$lib/components/ui/button';
  import { copyText } from '$lib/utils/clipboard';
  import { notifyError } from '$lib/utils/notify';

  let {
    text,
    label = 'Copy',
    showLabel = false,
  }: { text: string; label?: string; showLabel?: boolean } = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function copy() {
    if (!(await copyText(text))) {
      notifyError('Could not copy to clipboard.');
      return;
    }
    copied = true;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => (copied = false), 1500);
  }

  onDestroy(() => {
    if (timer) clearTimeout(timer);
  });
</script>

<Button
  variant="ghost"
  size={showLabel ? 'xs' : 'icon-xs'}
  class="text-muted-foreground"
  aria-label={copied ? 'Copied' : label}
  title={copied ? 'Copied' : label}
  onclick={copy}
>
  {#if copied}<Check class="text-success" />{:else}<Copy />{/if}
  {#if showLabel}{copied ? 'Copied' : label}{/if}
</Button>
