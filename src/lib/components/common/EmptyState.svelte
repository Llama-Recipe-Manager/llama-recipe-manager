<script lang="ts">
  import type { Component, Snippet } from 'svelte';

  let {
    icon: IconCmp,
    title,
    description,
    children,
  }: {
    icon: Component<{ class?: string }>;
    title: string;
    description?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="flex flex-1 flex-col items-center justify-center gap-3 p-10 text-center">
  <div class="mb-1 grid size-12 place-items-center rounded-xl bg-primary/10 text-primary">
    <IconCmp class="size-5" />
  </div>
  <h2 class="text-base font-semibold">{title}</h2>
  {#if description}
    <p class="max-w-sm text-muted-foreground">{description}</p>
  {/if}
  {#if children}
    <div class="mt-2 flex gap-2">{@render children()}</div>
  {/if}
</div>
