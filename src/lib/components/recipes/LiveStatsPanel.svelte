<script lang="ts">
  import Sparkline from '$lib/components/common/Sparkline.svelte';
  import StatusDot from '$lib/components/common/StatusDot.svelte';
  import * as Card from '$lib/components/ui/card';
  import { liveStats } from '$lib/stores/liveStats.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { cn } from '$lib/utils/cn';
  import { formatSeconds, formatTokens, sparklinePoints } from '$lib/utils/metrics';

  const metricsEnabled = $derived(settingsStore.current.metrics_enabled);
  const slotsEnabled = $derived(settingsStore.current.slots_enabled);
  const m = $derived(liveStats.metrics);
  const props = $derived(liveStats.props);
  const nCtx = $derived(props?.default_generation_settings?.n_ctx ?? null);

  const peakRate = $derived(Math.max(1, ...liveStats.series.prompt, ...liveStats.series.gen));
  const activity = $derived(
    liveStats.samples.map((s) => ({ active: s.activeSlots, queue: s.queue })),
  );
  const peakActivity = $derived(
    Math.max(1, props?.total_slots ?? 0, ...activity.map((a) => a.active + a.queue)),
  );

  const tiles = $derived([
    {
      label: 'Generate',
      value: liveStats.recent.genTokPerSec.toFixed(1),
      unit: 'tok/s',
      foot: `${m.predictedTokensPerSecond.toFixed(1)} lifetime avg`,
      spark: liveStats.series.gen.slice(-30),
      show: metricsEnabled,
    },
    {
      label: 'Prompt',
      value: liveStats.recent.promptTokPerSec.toFixed(1),
      unit: 'tok/s',
      foot: `${m.promptTokensPerSecond.toFixed(1)} lifetime avg`,
      spark: liveStats.series.prompt.slice(-30),
      show: metricsEnabled,
    },
    {
      label: 'Slots busy',
      value: String(liveStats.activeSlots),
      unit: props ? `/ ${props.total_slots}` : '',
      foot: `${m.requestsProcessing} in flight`,
      spark: null,
      show: true,
    },
    {
      label: 'Queue',
      value: String(m.requestsDeferred),
      unit: 'deferred',
      foot: `largest batch ${m.nTokensMax}${nCtx ? ` / ${nCtx.toLocaleString()} ctx` : ''}`,
      spark: null,
      show: metricsEnabled,
    },
  ]);
</script>

<div class="flex flex-col gap-4">
  <div class="flex items-center gap-2 text-xs text-muted-foreground">
    <StatusDot state={liveStats.healthy ? 'ready' : 'warming'} />
    {#if liveStats.lastError}
      <span class="text-destructive">{liveStats.lastError}</span>
    {:else if !liveStats.healthy}
      Waiting for /health…
    {:else}
      Healthy · polling every 2s
    {/if}
    {#if !metricsEnabled}
      <span class="ml-auto text-faint">
        Enable the Prometheus metrics endpoint in Settings for throughput and token totals.
      </span>
    {/if}
  </div>

  <div class="grid grid-cols-2 gap-3 xl:grid-cols-4">
    {#each tiles.filter((t) => t.show) as tile (tile.label)}
      <Card.Root class="gap-1 py-3">
        <Card.Content class="px-4">
          <div class="text-[11px] font-medium tracking-wide text-faint uppercase">
            {tile.label}
          </div>
          <div class="mt-1 flex items-baseline gap-1">
            <span class="font-mono text-2xl font-semibold tabular-nums">{tile.value}</span>
            <span class="text-xs text-muted-foreground">{tile.unit}</span>
          </div>
          {#if tile.spark && tile.spark.length > 1}
            <Sparkline values={tile.spark} height={28} class="mt-1" />
          {/if}
          <div class="mt-1 text-[11px] text-faint">{tile.foot}</div>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>

  {#if slotsEnabled && liveStats.slots.length > 0}
    <div class="flex flex-wrap gap-1.5" aria-label="Slot activity">
      {#each liveStats.slots as slot (slot.id)}
        <div
          class={cn(
            'grid h-6 min-w-8 place-items-center rounded border px-1.5 font-mono text-[11px] text-faint',
            slot.is_processing && 'border-success/40 bg-success/15 text-success',
          )}
          title="Slot {slot.id}{slot.is_processing ? ' · processing' : ''}"
        >
          {slot.id}
        </div>
      {/each}
    </div>
  {/if}

  {#if metricsEnabled}
    <div class="grid gap-3 lg:grid-cols-2">
      <Card.Root class="gap-2 py-3">
        <Card.Header class="flex items-center justify-between px-4">
          <Card.Title class="text-xs font-medium">Throughput</Card.Title>
          <span class="flex items-center gap-3 text-[11px] text-faint">
            <span class="flex items-center gap-1"
              ><span class="h-0.5 w-3 rounded bg-primary"></span>generate</span
            >
            <span class="flex items-center gap-1"
              ><span class="h-0.5 w-3 rounded bg-faint"></span>prompt</span
            >
            <span>peak {peakRate.toFixed(0)} tok/s</span>
          </span>
        </Card.Header>
        <Card.Content class="px-4">
          <svg viewBox="0 0 240 80" preserveAspectRatio="none" class="h-28 w-full">
            {#each [20, 40, 60] as y (y)}
              <line x1="0" x2="240" y1={y} y2={y} class="stroke-border" stroke-width="0.5" />
            {/each}
            {#if liveStats.series.prompt.length > 1}
              <polyline
                points={sparklinePoints(
                  liveStats.series.prompt.map((v) => v / peakRate),
                  240,
                  80,
                )}
                fill="none"
                class="stroke-faint"
                stroke-width="1.25"
                vector-effect="non-scaling-stroke"
              />
            {/if}
            {#if liveStats.series.gen.length > 1}
              <polyline
                points={sparklinePoints(
                  liveStats.series.gen.map((v) => v / peakRate),
                  240,
                  80,
                )}
                fill="none"
                class="stroke-primary"
                stroke-width="1.75"
                vector-effect="non-scaling-stroke"
              />
            {/if}
          </svg>
        </Card.Content>
      </Card.Root>

      <Card.Root class="gap-2 py-3">
        <Card.Header class="flex items-center justify-between px-4">
          <Card.Title class="text-xs font-medium">Activity</Card.Title>
          <span class="flex items-center gap-3 text-[11px] text-faint">
            <span class="flex items-center gap-1"
              ><span class="size-2 rounded-sm bg-success"></span>active</span
            >
            <span class="flex items-center gap-1"
              ><span class="size-2 rounded-sm bg-warning"></span>queued</span
            >
            {#if props}<span>{props.total_slots} slots</span>{/if}
          </span>
        </Card.Header>
        <Card.Content class="px-4">
          <svg viewBox="0 0 240 80" preserveAspectRatio="none" class="h-28 w-full">
            {#each [20, 40, 60] as y (y)}
              <line x1="0" x2="240" y1={y} y2={y} class="stroke-border" stroke-width="0.5" />
            {/each}
            {#if activity.length > 0}
              {@const barW = 240 / activity.length}
              {#each activity as a, i (i)}
                {@const activeH = (a.active / peakActivity) * 76}
                {@const queueH = (a.queue / peakActivity) * 76}
                <rect
                  x={i * barW + 0.5}
                  y={78 - activeH}
                  width={Math.max(0.5, barW - 1)}
                  height={activeH}
                  class="fill-success/70"
                />
                {#if queueH > 0}
                  <rect
                    x={i * barW + 0.5}
                    y={78 - activeH - queueH}
                    width={Math.max(0.5, barW - 1)}
                    height={queueH}
                    class="fill-warning/80"
                  />
                {/if}
              {/each}
            {/if}
          </svg>
        </Card.Content>
      </Card.Root>
    </div>
  {/if}

  {#if props}
    <Card.Root class="py-3">
      <Card.Content class="grid grid-cols-2 gap-x-6 gap-y-2 px-4 text-xs sm:grid-cols-4">
        <div>
          <div class="text-faint">Model alias</div>
          <div class="truncate font-mono" title={props.model_alias}>{props.model_alias}</div>
        </div>
        <div>
          <div class="text-faint">Context</div>
          <div class="font-mono">{nCtx ? nCtx.toLocaleString() : '—'}</div>
        </div>
        <div>
          <div class="text-faint">Vision</div>
          <div class="font-mono">{props.modalities?.vision ? 'yes' : 'no'}</div>
        </div>
        <div>
          <div class="text-faint">Build</div>
          <div class="truncate font-mono" title={props.build_info}>{props.build_info || '—'}</div>
        </div>
      </Card.Content>
    </Card.Root>
  {/if}

  {#if metricsEnabled}
    <p class="text-xs text-faint">
      {formatTokens(m.promptTokensTotal)} prompt tokens in {formatSeconds(m.promptSecondsTotal)} ·
      {formatTokens(m.tokensPredictedTotal)} generated in {formatSeconds(
        m.tokensPredictedSecondsTotal,
      )} ·
      {formatTokens(m.decodeTotal)} decodes
    </p>
  {/if}
</div>
