<script lang="ts">
  import { sparklinePoints } from '$lib/utils/metrics';
  import { cn } from '$lib/utils/cn';

  let {
    values,
    height = 40,
    class: className = '',
    stroke = 'var(--color-primary)',
  }: { values: number[]; height?: number; class?: string; stroke?: string } = $props();

  const W = 300;
  const points = $derived(sparklinePoints(values, W, height));
  const area = $derived(points ? `2,${height - 2} ${points} ${W - 2},${height - 2}` : '');
  const gradId = `spark-${Math.random().toString(36).slice(2, 8)}`;
</script>

<svg
  viewBox="0 0 {W} {height}"
  preserveAspectRatio="none"
  class={cn('block w-full', className)}
  style:height="{height}px"
  aria-hidden="true"
>
  <defs>
    <linearGradient id={gradId} x1="0" x2="0" y1="0" y2="1">
      <stop offset="0%" stop-color={stroke} stop-opacity="0.28" />
      <stop offset="100%" stop-color={stroke} stop-opacity="0" />
    </linearGradient>
  </defs>
  {#if points}
    <polygon points={area} fill="url(#{gradId})" />
    <polyline
      {points}
      fill="none"
      {stroke}
      stroke-width="1.5"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  {/if}
</svg>
