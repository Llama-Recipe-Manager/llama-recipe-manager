<script lang="ts">
  import type { Snippet } from 'svelte';

  import { Label } from '$lib/components/ui/label';
  import { cn } from '$lib/utils/cn';

  let {
    label,
    for: htmlFor,
    hint,
    error,
    class: className = '',
    children,
  }: {
    label: string;
    for?: string;
    hint?: string;
    error?: string;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<div class={cn('flex flex-col gap-1.5', className)}>
  <Label for={htmlFor} class="text-xs font-medium text-muted-foreground">{label}</Label>
  {@render children()}
  {#if error}
    <p class="text-xs text-destructive">{error}</p>
  {:else if hint}
    <p class="text-xs text-faint">{hint}</p>
  {/if}
</div>
