import { describe, expect, it } from 'vitest';

import { trayStatus } from '$lib/utils/tray';

describe('trayStatus', () => {
  it('names the recipe for each server phase', () => {
    expect(trayStatus('starting', 'Qwen 8B')).toBe('Starting Qwen 8B…');
    expect(trayStatus('ready', 'Qwen 8B')).toBe('Running Qwen 8B');
    expect(trayStatus('stopping', 'Qwen 8B')).toBe('Stopping Qwen 8B…');
  });

  it('reports idle regardless of the last recipe', () => {
    expect(trayStatus('idle', 'Qwen 8B')).toBe('No server running');
  });

  it('falls back when the recipe is unknown', () => {
    expect(trayStatus('ready', null)).toBe('Running server');
  });
});
