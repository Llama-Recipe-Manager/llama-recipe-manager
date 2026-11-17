<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';

  import { Toaster } from '$lib/components/ui/sonner';
  import * as Tooltip from '$lib/components/ui/tooltip';

  let { children }: { children: Snippet } = $props();
</script>

<Tooltip.Provider delayDuration={300}>
  {@render children()}
</Tooltip.Provider>
<Toaster position="bottom-right" richColors closeButton />
