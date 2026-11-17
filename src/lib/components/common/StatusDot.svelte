<script lang="ts" module>
  export type DotState = 'idle' | 'warming' | 'ready' | 'error';
</script>

<script lang="ts">
  import { cn } from '$lib/utils/cn';

  let { state, class: className = '' }: { state: DotState; class?: string } = $props();
</script>

<span
  class={cn(
    'inline-block size-2 shrink-0 rounded-full',
    state === 'ready' && 'bg-success text-success',
    state === 'warming' && 'pulse-ring bg-warning text-warning',
    state === 'error' && 'bg-destructive',
    state === 'idle' && 'bg-faint/60',
    className,
  )}
></span>
