<script lang="ts">
  import '../app.css';
  import { onDestroy, onMount, type Snippet } from 'svelte';
  import { subscribeTheme } from '$lib/stores/theme.svelte';

  let { children }: { children: Snippet } = $props();

  let unsubscribe: (() => void) | null = null;

  onMount(async () => {
    unsubscribe = await subscribeTheme();
  });

  onDestroy(() => {
    unsubscribe?.();
  });
</script>

{@render children()}
