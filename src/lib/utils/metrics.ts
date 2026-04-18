/**
 * Parsing & helpers for llama.cpp's `/metrics` (Prometheus text format) and
 * the companion JSON endpoints used by the live stats panel.
 */

export interface LlamaMetrics {
  promptTokensTotal: number;
  promptSecondsTotal: number;
  tokensPredictedTotal: number;
  tokensPredictedSecondsTotal: number;
  decodeTotal: number;
  nTokensMax: number;
  busySlotsPerDecode: number;
  promptTokensPerSecond: number;
  predictedTokensPerSecond: number;
  requestsProcessing: number;
  requestsDeferred: number;
}

const METRIC_KEYS: Record<string, keyof LlamaMetrics> = {
  'llamacpp:prompt_tokens_total': 'promptTokensTotal',
  'llamacpp:prompt_seconds_total': 'promptSecondsTotal',
  'llamacpp:tokens_predicted_total': 'tokensPredictedTotal',
  'llamacpp:tokens_predicted_seconds_total': 'tokensPredictedSecondsTotal',
  'llamacpp:n_decode_total': 'decodeTotal',
  'llamacpp:n_tokens_max': 'nTokensMax',
  'llamacpp:n_busy_slots_per_decode': 'busySlotsPerDecode',
  'llamacpp:prompt_tokens_seconds': 'promptTokensPerSecond',
  'llamacpp:predicted_tokens_seconds': 'predictedTokensPerSecond',
  'llamacpp:requests_processing': 'requestsProcessing',
  'llamacpp:requests_deferred': 'requestsDeferred',
};

export function emptyMetrics(): LlamaMetrics {
  return {
    promptTokensTotal: 0,
    promptSecondsTotal: 0,
    tokensPredictedTotal: 0,
    tokensPredictedSecondsTotal: 0,
    decodeTotal: 0,
    nTokensMax: 0,
    busySlotsPerDecode: 0,
    promptTokensPerSecond: 0,
    predictedTokensPerSecond: 0,
    requestsProcessing: 0,
    requestsDeferred: 0,
  };
}

/**
 * Parse llama.cpp's Prometheus-format `/metrics` body. Comments (`# HELP`,
 * `# TYPE`) and any unknown metric lines are ignored. Missing metrics fall
 * back to 0 so the caller can render a zeroed dashboard instead of crashing.
 */
export function parseMetrics(text: string): LlamaMetrics {
  const out = emptyMetrics();
  for (const rawLine of text.split('\n')) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;

    // Lines look like:  llamacpp:prompt_tokens_total 11
    // (no labels are emitted by the server today, but be defensive about them
    //  if they ever appear: split on whitespace, last token is the value.)
    const space = line.lastIndexOf(' ');
    if (space < 0) continue;
    const name = line.slice(0, space).split('{')[0].trim();
    const valueStr = line.slice(space + 1).trim();
    const key = METRIC_KEYS[name];
    if (!key) continue;
    const value = Number(valueStr);
    if (!Number.isFinite(value)) continue;
    out[key] = value;
  }
  return out;
}

/**
 * Compute "recent" throughput from two metric snapshots — typically the most
 * recent sample minus one taken a few seconds ago. Falls back to 0 (instead
 * of NaN/Infinity) when no work happened in the window.
 */
export function recentRates(
  prev: LlamaMetrics,
  curr: LlamaMetrics,
): { promptTokPerSec: number; genTokPerSec: number } {
  const dPromptTok = curr.promptTokensTotal - prev.promptTokensTotal;
  const dPromptSec = curr.promptSecondsTotal - prev.promptSecondsTotal;
  const dGenTok = curr.tokensPredictedTotal - prev.tokensPredictedTotal;
  const dGenSec = curr.tokensPredictedSecondsTotal - prev.tokensPredictedSecondsTotal;
  return {
    promptTokPerSec: dPromptSec > 0 ? dPromptTok / dPromptSec : 0,
    genTokPerSec: dGenSec > 0 ? dGenTok / dGenSec : 0,
  };
}

export interface SlotInfo {
  id: number;
  n_ctx: number;
  is_processing: boolean;
  speculative?: boolean;
}

export interface ServerProps {
  model_alias: string;
  model_path: string;
  total_slots: number;
  build_info: string;
  is_sleeping?: boolean;
  modalities?: { vision?: boolean; audio?: boolean };
  default_generation_settings?: { n_ctx?: number };
}

/**
 * Render an SVG polyline `points` attribute scaled into the given box. Empty
 * series returns an empty string so the caller can short-circuit.
 */
export function sparklinePoints(
  values: number[],
  width: number,
  height: number,
  pad = 2,
): string {
  if (values.length === 0) return '';
  const max = Math.max(1, ...values);
  const innerW = width - pad * 2;
  const innerH = height - pad * 2;
  const step = values.length > 1 ? innerW / (values.length - 1) : 0;
  return values
    .map((v, i) => {
      const x = pad + i * step;
      const y = pad + innerH - (v / max) * innerH;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(' ');
}

export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return Math.round(n).toString();
}

/**
 * One snapshot in the chart ring buffer. We keep both the raw metrics and a
 * couple of derived gauges so chart series builders don't need to know about
 * polling details.
 */
export interface ChartSample {
  at: number;
  m: LlamaMetrics;
  activeSlots: number;
  queue: number;
}

/**
 * Returns true if any monotonic counter in `curr` went *down* relative to
 * `prev`, which can only happen when llama-server restarted. The chart ring
 * buffer should be cleared when this fires so derived rates don't go negative.
 */
export function isCounterReset(prev: LlamaMetrics, curr: LlamaMetrics): boolean {
  return (
    curr.promptTokensTotal < prev.promptTokensTotal ||
    curr.tokensPredictedTotal < prev.tokensPredictedTotal ||
    curr.decodeTotal < prev.decodeTotal ||
    curr.promptSecondsTotal < prev.promptSecondsTotal ||
    curr.tokensPredictedSecondsTotal < prev.tokensPredictedSecondsTotal
  );
}

/**
 * Build per-sample throughput series (tok/s) from a chart buffer. Each series
 * value is computed from the delta between consecutive samples, so the result
 * has `samples.length - 1` entries. Returns zero-length arrays when there
 * isn't enough history yet.
 */
export function throughputSeries(samples: ChartSample[]): { prompt: number[]; gen: number[] } {
  const prompt: number[] = [];
  const gen: number[] = [];
  for (let i = 1; i < samples.length; i += 1) {
    const r = recentRates(samples[i - 1].m, samples[i].m);
    prompt.push(Math.max(0, r.promptTokPerSec));
    gen.push(Math.max(0, r.genTokPerSec));
  }
  return { prompt, gen };
}

export function formatSeconds(s: number): string {
  if (s < 1) return `${(s * 1000).toFixed(0)}ms`;
  if (s < 60) return `${s.toFixed(2)}s`;
  const m = Math.floor(s / 60);
  const r = Math.round(s - m * 60);
  return `${m}m ${r}s`;
}
