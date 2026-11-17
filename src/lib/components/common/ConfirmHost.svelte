<script lang="ts">
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { confirmStore } from '$lib/stores/confirm.svelte';

  const o = $derived(confirmStore.options);
</script>

<AlertDialog.Root
  open={confirmStore.open}
  onOpenChange={(open) => {
    if (!open) confirmStore.settle(false);
  }}
>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>{o.title}</AlertDialog.Title>
      <AlertDialog.Description class="whitespace-pre-line">{o.message}</AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={() => confirmStore.settle(false)}>
        {o.cancelLabel ?? 'Cancel'}
      </AlertDialog.Cancel>
      <AlertDialog.Action
        variant={o.destructive ? 'destructive' : 'default'}
        onclick={() => confirmStore.settle(true)}
      >
        {o.confirmLabel ?? 'Confirm'}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
