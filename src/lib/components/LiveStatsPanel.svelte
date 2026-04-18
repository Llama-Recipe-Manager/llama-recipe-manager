<script lang="ts">
  import { onDestroy } from 'svelte';

  import { settingsStore } from '$lib/stores/settings.svelte';
  import { fetchHealthOk, fetchMetricsText, fetchProps, fetchSlots } from '$lib/utils/llamaClient';
  import {
    emptyMetrics,
    formatSeconds,
    formatTokens,
    isCounterReset,
    parseMetrics,
    recentRates,
    sparklinePoints,
    throughputSeries,
    type ChartSample,
    type LlamaMetrics,
    type ServerProps,
    type SlotInfo,
  } from '$lib/utils/metrics';

  /** How often to scrape (ms). 2s feels lively without burning CPU. */
  const POLL_MS = 2000;
  /** Window over which "recent throughput" is computed. */
  const RATE_WINDOW_MS = 8000;
  /** Number of samples in the chart ring buffer (~2 min at POLL_MS=2s). */
  const CHART_POINTS = 60;

  const metricsEnabled = $derived(settingsStore.current.metrics_enabled);
  const slotsEnabled = $derived(settingsStore.current.slots_enabled);

  let metrics = $state<LlamaMetrics>(emptyMetrics());
  let samples = $state<ChartSample[]>([]);
  let props = $state<ServerProps | null>(null);
  let slots = $state<SlotInfo[]>([]);
  let healthy = $state(false);
  let lastError = $state<string | null>(null);
  let lastUpdate = $state<number | null>(null);

  let timer: ReturnType<typeof setInterval> | null = null;
  let inflight: AbortController | null = null;

  const recent = $derived.by(() => {
    if (samples.length < 2) return { promptTokPerSec: 0, genTokPerSec: 0 };
    const cutoff = Date.now() - RATE_WINDOW_MS;
    const baseline = samples.find((s) => s.at >= cutoff) ?? samples[0];
    return recentRates(baseline.m, metrics);
  });

  const activeSlots = $derived(slots.filter((s) => s.is_processing).length);

  const series = $derived(throughputSeries(samples));
  const genSpark = $derived(series.gen.slice(-30));
  const peakRate = $derived(Math.max(1, ...series.prompt, ...series.gen));
  const activitySeries = $derived(samples.map((s) => ({ active: s.activeSlots, queue: s.queue })));
  const peakActivity = $derived(
    Math.max(1, props?.total_slots ?? 0, ...activitySeries.map((a) => a.active + a.queue)),
  );

  async function poll() {
    // The WebView throttles setInterval to ~1Hz when hidden, which would
    // produce sparse spikes in the chart. Skip the scrape entirely so the
    // ring buffer stays clean.
    if (typeof document !== 'undefined' && document.visibilityState === 'hidden') return;

    inflight?.abort();
    inflight = new AbortController();
    const { signal } = inflight;
    const settings = settingsStore.current;
    try {
      const reqs: Promise<unknown>[] = [
        fetchHealthOk(settings, signal).then((ok) => (healthy = ok)),
      ];

      // /props is always available — it's our baseline.
      reqs.push(
        fetchProps<ServerProps>(settings, signal)
          .then((p) => (props = p))
          .catch((e) => {
            // Props can momentarily 404 during model swap; tolerate.
            if (signal.aborted) return;
            throw e;
          }),
      );

      const metricsPromise: Promise<LlamaMetrics | null> = metricsEnabled
        ? fetchMetricsText(settings, signal).then((text) => parseMetrics(text))
        : Promise.resolve(null);
      reqs.push(metricsPromise);

      if (slotsEnabled) {
        reqs.push(
          fetchSlots<SlotInfo[]>(settings, signal)
            .then((s) => (slots = s))
            .catch(() => {
              // Slots can be disabled mid-session; just clear.
              slots = [];
            }),
        );
      } else {
        slots = [];
      }

      await Promise.all(reqs);
      const fresh = await metricsPromise;
      if (fresh) {
        // Detect llama-server restart (counters dropped) and reset history.
        if (samples.length > 0 && isCounterReset(metrics, fresh)) {
          samples = [];
        }
        metrics = fresh;
        const sample: ChartSample = {
          at: Date.now(),
          m: fresh,
          activeSlots: slots.filter((s) => s.is_processing).length,
          queue: fresh.requestsDeferred,
        };
        samples = [...samples.slice(-(CHART_POINTS - 1)), sample];
      }

      lastError = null;
      lastUpdate = Date.now();
    } catch (e) {
      if ((e as Error).name === 'AbortError') return;
      lastError = (e as Error).message ?? String(e);
    }
  }

  $effect(() => {
    poll();
    timer = setInterval(poll, POLL_MS);
    return () => {
      if (timer) clearInterval(timer);
      timer = null;
      inflight?.abort();
      inflight = null;
    };
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
    inflight?.abort();
  });
</script>

<section class="live">
  <header class="head">
    <div class="title">
      <span class="dot" class:on={healthy}></span>
      <h3>Live</h3>
      <span class="sub">
        {#if !healthy && !lastError}
          waiting for /health…
        {:else if lastError}
          <span class="err">{lastError}</span>
        {:else}
          updated {lastUpdate
            ? `${Math.max(0, Math.round((Date.now() - lastUpdate) / 1000))}s ago`
            : ''}
        {/if}
      </span>
    </div>
    {#if !metricsEnabled}
      <span class="hint">
        Enable “Prometheus metrics endpoint” in Settings for throughput & token totals.
      </span>
    {/if}
  </header>

  {#if metricsEnabled}
    <div class="tiles">
      <div class="tile">
        <div class="tile-label">Prompt</div>
        <div class="tile-value">
          {recent.promptTokPerSec.toFixed(1)}<span class="unit">tok/s</span>
        </div>
        <div class="tile-foot">avg {metrics.promptTokensPerSecond.toFixed(1)} lifetime</div>
      </div>
      <div class="tile primary">
        <div class="tile-label">Generate</div>
        <div class="tile-value">
          {recent.genTokPerSec.toFixed(1)}<span class="unit">tok/s</span>
        </div>
        <div class="tile-foot">
          avg {metrics.predictedTokensPerSecond.toFixed(1)} lifetime
        </div>
        {#if genSpark.length > 1}
          <svg class="spark" viewBox="0 0 120 28" preserveAspectRatio="none">
            <polyline points={sparklinePoints(genSpark, 120, 28)} />
          </svg>
        {/if}
      </div>
    </div>
  {/if}

  <div class="chips">
    {#if props}
      <span class="chip">
        Active <strong>{activeSlots}</strong>/<strong>{props.total_slots}</strong> slots
      </span>
    {/if}
    {#if metricsEnabled}
      <span class="chip" class:warn={metrics.requestsDeferred > 0}>
        Queue <strong>{metrics.requestsDeferred}</strong>
      </span>
      <span class="chip">
        In flight <strong>{metrics.requestsProcessing}</strong>
      </span>
      {#if props?.default_generation_settings?.n_ctx}
        <span class="chip">
          Largest batch <strong>{metrics.nTokensMax}</strong> /
          {props.default_generation_settings.n_ctx.toLocaleString()} ctx
        </span>
      {:else}
        <span class="chip">
          Largest batch <strong>{metrics.nTokensMax}</strong>
        </span>
      {/if}
    {/if}
    <span class="chip" class:ok={healthy} class:err-chip={!healthy}>
      Health {healthy ? 'ready' : 'not ready'}
    </span>
  </div>

  {#if slotsEnabled && slots.length > 0}
    <div class="slot-strip" aria-label="Slot activity">
      {#each slots as slot (slot.id)}
        <div class="slot" class:busy={slot.is_processing} title="Slot {slot.id}">
          <span class="slot-id">{slot.id}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if metricsEnabled}
    <div class="charts">
      <!-- Throughput line chart: prompt (faded) + generate (accent) -->
      <div class="chart">
        <div class="chart-head">
          <span class="chart-title">Throughput</span>
          <span class="chart-legend">
            <span class="lg gen"></span> generate
            <span class="lg prompt"></span> prompt
            <span class="chart-axis">peak {peakRate.toFixed(0)} tok/s</span>
          </span>
        </div>
        <svg class="chart-svg" viewBox="0 0 240 60" preserveAspectRatio="none">
          <line class="grid" x1="0" y1="20" x2="240" y2="20" />
          <line class="grid" x1="0" y1="40" x2="240" y2="40" />
          {#if series.prompt.length > 1}
            <polyline
              class="prompt"
              points={sparklinePoints(
                series.prompt.map((v) => v / peakRate),
                240,
                60,
              )}
            />
          {/if}
          {#if series.gen.length > 1}
            <polyline
              class="gen"
              points={sparklinePoints(
                series.gen.map((v) => v / peakRate),
                240,
                60,
              )}
            />
          {/if}
        </svg>
      </div>

      <!-- Activity stacked bars: active slots (green) + queued (amber) -->
      <div class="chart">
        <div class="chart-head">
          <span class="chart-title">Activity</span>
          <span class="chart-legend">
            <span class="lg active"></span> active
            <span class="lg queue"></span> queued
            {#if props}
              <span class="chart-axis">{props.total_slots} slots</span>
            {/if}
          </span>
        </div>
        <svg class="chart-svg" viewBox="0 0 240 60" preserveAspectRatio="none">
          <line class="grid" x1="0" y1="20" x2="240" y2="20" />
          <line class="grid" x1="0" y1="40" x2="240" y2="40" />
          {#if activitySeries.length > 0}
            {@const barW = 240 / activitySeries.length}
            {#each activitySeries as a, i (i)}
              {@const activeH = (a.active / peakActivity) * 56}
              {@const queueH = (a.queue / peakActivity) * 56}
              <rect
                class="bar active"
                x={i * barW + 0.5}
                y={58 - activeH}
                width={Math.max(0.5, barW - 1)}
                height={activeH}
              />
              {#if queueH > 0}
                <rect
                  class="bar queue"
                  x={i * barW + 0.5}
                  y={58 - activeH - queueH}
                  width={Math.max(0.5, barW - 1)}
                  height={queueH}
                />
              {/if}
            {/each}
          {/if}
        </svg>
      </div>
    </div>
  {/if}

  {#if props}
    <div class="model-bar">
      <span class="muted">Model</span>
      <code>{props.model_alias}</code>
      <span class="dotsep">·</span>
      <span class="muted">Slots</span>
      <strong>{props.total_slots}</strong>
      {#if props.default_generation_settings?.n_ctx}
        <span class="dotsep">·</span>
        <span class="muted">Ctx</span>
        <strong>{props.default_generation_settings.n_ctx.toLocaleString()}</strong>
      {/if}
      {#if props.modalities}
        <span class="dotsep">·</span>
        <span class="muted">Vision</span>
        <strong>{props.modalities.vision ? 'yes' : 'no'}</strong>
      {/if}
      {#if props.build_info}
        <span class="dotsep">·</span>
        <span class="muted">Build</span>
        <code class="small">{props.build_info}</code>
      {/if}
    </div>
  {/if}

  {#if metricsEnabled}
    <div class="totals">
      Total: {formatTokens(metrics.promptTokensTotal)} prompt tok in
      {formatSeconds(metrics.promptSecondsTotal)} ·
      {formatTokens(metrics.tokensPredictedTotal)} generated in
      {formatSeconds(metrics.tokensPredictedSecondsTotal)} ·
      {formatTokens(metrics.decodeTotal)} decodes
    </div>
  {/if}
</section>

<style>
  .live {
    display: flex;
    flex-direction: column;
    gap: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
    padding: 14px 16px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .sub {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .err {
    color: var(--danger);
  }

  .hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-tertiary);
  }

  .dot.on {
    background: var(--success);
    box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.5);
    animation: pulse 1.6s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.5);
    }
    70% {
      box-shadow: 0 0 0 6px rgba(52, 199, 89, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0);
    }
  }

  .tiles {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .tile {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    position: relative;
    overflow: hidden;
  }

  .tile.primary {
    border-color: rgba(0, 113, 227, 0.4);
  }

  .tile-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    font-weight: 600;
  }

  .tile-value {
    font-size: 26px;
    font-weight: 700;
    line-height: 1.1;
    margin-top: 4px;
    font-variant-numeric: tabular-nums;
  }

  .unit {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    margin-left: 4px;
  }

  .tile-foot {
    font-size: 11px;
    color: var(--text-tertiary);
    margin-top: 2px;
  }

  .spark {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 120px;
    height: 28px;
    opacity: 0.85;
  }

  .spark polyline {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    font-size: 11px;
    padding: 3px 9px;
    border-radius: 100px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .chip strong {
    color: var(--text-primary);
    font-weight: 600;
  }

  .chip.warn {
    background: rgba(255, 159, 10, 0.12);
    color: var(--warning, #ff9f0a);
  }

  .chip.ok {
    background: rgba(52, 199, 89, 0.13);
    color: var(--success);
  }

  .chip.err-chip {
    background: rgba(255, 59, 48, 0.13);
    color: var(--danger);
  }

  .slot-strip {
    display: flex;
    gap: 6px;
  }

  .slot {
    flex: 1;
    height: 24px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background-color 0.2s,
      border-color 0.2s;
  }

  .slot.busy {
    background: rgba(52, 199, 89, 0.18);
    border-color: rgba(52, 199, 89, 0.5);
  }

  .slot-id {
    font-size: 10px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .slot.busy .slot-id {
    color: var(--success);
    font-weight: 600;
  }

  .charts {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  @media (max-width: 720px) {
    .charts {
      grid-template-columns: 1fr;
    }
  }

  .chart {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 10px 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .chart-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .chart-title {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    font-weight: 600;
  }

  .chart-legend {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .chart-axis {
    margin-left: 6px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .lg {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 2px;
    margin-right: 3px;
    vertical-align: middle;
  }

  .lg.gen {
    background: var(--accent, #0a84ff);
  }
  .lg.prompt {
    background: var(--text-tertiary);
    opacity: 0.7;
  }
  .lg.active {
    background: var(--success, #34c759);
  }
  .lg.queue {
    background: var(--warning, #ff9f0a);
  }

  .chart-svg {
    width: 100%;
    height: 60px;
    display: block;
  }

  .chart-svg .grid {
    stroke: var(--border);
    stroke-width: 1;
    stroke-dasharray: 2 3;
    opacity: 0.6;
  }

  .chart-svg polyline {
    fill: none;
    stroke-width: 1.5;
    vector-effect: non-scaling-stroke;
  }

  .chart-svg polyline.gen {
    stroke: var(--accent, #0a84ff);
  }

  .chart-svg polyline.prompt {
    stroke: var(--text-tertiary);
    opacity: 0.6;
  }

  .chart-svg .bar.active {
    fill: var(--success, #34c759);
  }

  .chart-svg .bar.queue {
    fill: var(--warning, #ff9f0a);
  }

  .model-bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 12px;
  }

  .model-bar code {
    font-family: var(--font-mono);
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    word-break: break-all;
  }

  .model-bar code.small {
    font-size: 11px;
  }

  .model-bar strong {
    color: var(--text-primary);
    font-weight: 600;
  }

  .muted {
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .dotsep {
    color: var(--text-tertiary);
  }

  .totals {
    font-size: 11px;
    color: var(--text-tertiary);
    border-top: 1px solid var(--border);
    padding-top: 8px;
    font-variant-numeric: tabular-nums;
  }
</style>
