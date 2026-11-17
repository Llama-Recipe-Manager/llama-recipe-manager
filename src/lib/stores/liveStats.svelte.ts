import { settingsStore } from '$lib/stores/settings.svelte';
import { fetchHealthOk, fetchMetricsText, fetchProps, fetchSlots } from '$lib/utils/llamaClient';
import {
  emptyMetrics,
  isCounterReset,
  parseMetrics,
  recentRates,
  throughputSeries,
  type ChartSample,
  type LlamaMetrics,
  type ServerProps,
  type SlotInfo,
} from '$lib/utils/metrics';

const POLL_MS = 2000;
const RATE_WINDOW_MS = 8000;
/** ~2 minutes of history at POLL_MS. */
const CHART_POINTS = 60;

class LiveStatsStore {
  metrics = $state<LlamaMetrics>(emptyMetrics());
  samples = $state<ChartSample[]>([]);
  props = $state<ServerProps | null>(null);
  slots = $state<SlotInfo[]>([]);
  healthy = $state(false);
  lastError = $state<string | null>(null);
  lastUpdate = $state<number | null>(null);

  recent = $derived.by(() => {
    if (this.samples.length < 2) return { promptTokPerSec: 0, genTokPerSec: 0 };
    const cutoff = Date.now() - RATE_WINDOW_MS;
    const baseline = this.samples.find((s) => s.at >= cutoff) ?? this.samples[0];
    return recentRates(baseline.m, this.metrics);
  });
  series = $derived(throughputSeries(this.samples));
  activeSlots = $derived(this.slots.filter((s) => s.is_processing).length);

  private timer: ReturnType<typeof setInterval> | null = null;
  private inflight: AbortController | null = null;

  start(): void {
    if (this.timer) return;
    this.reset();
    void this.poll();
    this.timer = setInterval(() => void this.poll(), POLL_MS);
  }

  stop(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    this.inflight?.abort();
    this.inflight = null;
    this.healthy = false;
  }

  private reset(): void {
    this.metrics = emptyMetrics();
    this.samples = [];
    this.props = null;
    this.slots = [];
    this.lastError = null;
    this.lastUpdate = null;
  }

  private async poll(): Promise<void> {
    // Hidden WebViews throttle timers to ~1Hz, which would leave sparse spikes in the charts.
    if (typeof document !== 'undefined' && document.visibilityState === 'hidden') return;

    this.inflight?.abort();
    const ctrl = new AbortController();
    this.inflight = ctrl;
    const { signal } = ctrl;
    const settings = settingsStore.current;
    try {
      const metricsPromise = settings.metrics_enabled
        ? fetchMetricsText(settings, signal).then(parseMetrics)
        : Promise.resolve(null);
      await Promise.all([
        fetchHealthOk(settings, signal).then((ok) => (this.healthy = ok)),
        fetchProps<ServerProps>(settings, signal)
          .then((p) => (this.props = p))
          .catch((e) => {
            if (!signal.aborted) throw e;
          }),
        settings.slots_enabled
          ? fetchSlots<SlotInfo[]>(settings, signal)
              .then((s) => (this.slots = s))
              .catch(() => (this.slots = []))
          : (this.slots = []),
        metricsPromise,
      ]);
      const fresh = await metricsPromise;
      if (fresh) {
        if (this.samples.length > 0 && isCounterReset(this.metrics, fresh)) this.samples = [];
        this.metrics = fresh;
        this.samples = [
          ...this.samples.slice(-(CHART_POINTS - 1)),
          {
            at: Date.now(),
            m: fresh,
            activeSlots: this.activeSlots,
            queue: fresh.requestsDeferred,
          },
        ];
      }
      this.lastError = null;
      this.lastUpdate = Date.now();
    } catch (e) {
      if ((e as Error).name === 'AbortError') return;
      this.lastError = (e as Error).message ?? String(e);
    }
  }
}

export const liveStats = new LiveStatsStore();
