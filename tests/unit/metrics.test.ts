import { describe, expect, it } from 'vitest';

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
} from '../../src/lib/utils/metrics';

function sample(at: number, m: Partial<LlamaMetrics>, activeSlots = 0, queue = 0): ChartSample {
  return { at, m: { ...emptyMetrics(), ...m }, activeSlots, queue };
}

const SAMPLE = `# HELP llamacpp:prompt_tokens_total Number of prompt tokens processed.
# TYPE llamacpp:prompt_tokens_total counter
llamacpp:prompt_tokens_total 11
# HELP llamacpp:prompt_seconds_total Prompt process time
# TYPE llamacpp:prompt_seconds_total counter
llamacpp:prompt_seconds_total 0.196
llamacpp:tokens_predicted_total 186
llamacpp:tokens_predicted_seconds_total 5.391
llamacpp:n_decode_total 187
llamacpp:n_tokens_max 196
llamacpp:n_busy_slots_per_decode 1
llamacpp:prompt_tokens_seconds 56.1224
llamacpp:predicted_tokens_seconds 34.5019
llamacpp:requests_processing 0
llamacpp:requests_deferred 0
some_other_metric 99
`;

describe('parseMetrics', () => {
  it('parses every known metric and ignores unknowns/comments', () => {
    const m = parseMetrics(SAMPLE);
    expect(m.promptTokensTotal).toBe(11);
    expect(m.promptSecondsTotal).toBeCloseTo(0.196);
    expect(m.tokensPredictedTotal).toBe(186);
    expect(m.tokensPredictedSecondsTotal).toBeCloseTo(5.391);
    expect(m.decodeTotal).toBe(187);
    expect(m.nTokensMax).toBe(196);
    expect(m.busySlotsPerDecode).toBe(1);
    expect(m.promptTokensPerSecond).toBeCloseTo(56.1224);
    expect(m.predictedTokensPerSecond).toBeCloseTo(34.5019);
    expect(m.requestsProcessing).toBe(0);
    expect(m.requestsDeferred).toBe(0);
  });

  it('returns zeroed defaults for an empty body', () => {
    expect(parseMetrics('')).toEqual(emptyMetrics());
  });

  it('tolerates blank lines and stray whitespace', () => {
    const m = parseMetrics('\n\n  llamacpp:n_decode_total   42\n');
    expect(m.decodeTotal).toBe(42);
  });

  it('skips metrics with non-finite values', () => {
    const m = parseMetrics('llamacpp:requests_processing NaN\n');
    expect(m.requestsProcessing).toBe(0);
  });

  it('handles label-suffixed metric names defensively', () => {
    const m = parseMetrics('llamacpp:n_decode_total{instance="x"} 7\n');
    expect(m.decodeTotal).toBe(7);
  });
});

describe('recentRates', () => {
  it('divides token deltas by time deltas', () => {
    const a = parseMetrics(
      'llamacpp:tokens_predicted_total 100\nllamacpp:tokens_predicted_seconds_total 5\n',
    );
    const b = parseMetrics(
      'llamacpp:tokens_predicted_total 200\nllamacpp:tokens_predicted_seconds_total 7\n',
    );
    const r = recentRates(a, b);
    expect(r.genTokPerSec).toBe(50);
  });

  it('returns 0 when no time has elapsed (avoids NaN/Infinity)', () => {
    const a = emptyMetrics();
    const r = recentRates(a, a);
    expect(r.promptTokPerSec).toBe(0);
    expect(r.genTokPerSec).toBe(0);
  });
});

describe('sparklinePoints', () => {
  it('returns empty for no data', () => {
    expect(sparklinePoints([], 100, 20)).toBe('');
  });

  it('produces N space-separated x,y pairs', () => {
    const out = sparklinePoints([1, 2, 3], 100, 20);
    expect(out.split(' ')).toHaveLength(3);
  });
});

describe('isCounterReset', () => {
  it('returns false when counters are monotonically increasing', () => {
    const a = parseMetrics('llamacpp:tokens_predicted_total 100\n');
    const b = parseMetrics('llamacpp:tokens_predicted_total 150\n');
    expect(isCounterReset(a, b)).toBe(false);
  });

  it('detects a server restart by any counter dropping', () => {
    const a = parseMetrics('llamacpp:tokens_predicted_total 100\nllamacpp:n_decode_total 50\n');
    const b = parseMetrics('llamacpp:tokens_predicted_total 100\nllamacpp:n_decode_total 5\n');
    expect(isCounterReset(a, b)).toBe(true);
  });

  it('treats an unchanged snapshot as non-reset', () => {
    const a = parseMetrics('llamacpp:tokens_predicted_total 100\n');
    expect(isCounterReset(a, a)).toBe(false);
  });
});

describe('throughputSeries', () => {
  it('returns empty arrays with fewer than two samples', () => {
    expect(throughputSeries([])).toEqual({ prompt: [], gen: [] });
    expect(throughputSeries([sample(0, {})])).toEqual({ prompt: [], gen: [] });
  });

  it('produces N-1 deltas in the same order as input', () => {
    const s = [
      sample(0, { tokensPredictedTotal: 0, tokensPredictedSecondsTotal: 0 }),
      sample(2000, { tokensPredictedTotal: 100, tokensPredictedSecondsTotal: 2 }),
      sample(4000, { tokensPredictedTotal: 250, tokensPredictedSecondsTotal: 5 }),
    ];
    const out = throughputSeries(s);
    expect(out.gen).toHaveLength(2);
    expect(out.gen[0]).toBeCloseTo(50);
    expect(out.gen[1]).toBeCloseTo(50);
  });

  it('clamps negative deltas to zero', () => {
    const s = [
      sample(0, { tokensPredictedTotal: 100, tokensPredictedSecondsTotal: 5 }),
      sample(2000, { tokensPredictedTotal: 50, tokensPredictedSecondsTotal: 6 }),
    ];
    expect(throughputSeries(s).gen[0]).toBe(0);
  });
});

describe('formatters', () => {
  it('formats tokens with k/M suffixes', () => {
    expect(formatTokens(0)).toBe('0');
    expect(formatTokens(999)).toBe('999');
    expect(formatTokens(1500)).toBe('1.5k');
    expect(formatTokens(2_500_000)).toBe('2.5M');
  });

  it('formats seconds across magnitudes', () => {
    expect(formatSeconds(0.123)).toBe('123ms');
    expect(formatSeconds(2.5)).toBe('2.50s');
    expect(formatSeconds(125)).toBe('2m 5s');
  });
});
