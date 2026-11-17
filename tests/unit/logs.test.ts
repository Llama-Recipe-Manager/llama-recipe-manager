import { describe, expect, it } from 'vitest';

import { logLevel } from '$lib/utils/logs';

describe('logLevel', () => {
  it('uses the llama.cpp log prefix when present', () => {
    expect(logLevel('0.01.234.567 E failed to load model')).toBe('error');
    expect(logLevel('0.01.234.567 W something odd')).toBe('warn');
    expect(logLevel('0.01.234.567 I loading error-free model')).toBe('info');
  });

  it('falls back to keywords without a prefix', () => {
    expect(logLevel('llama_model_load: error loading model')).toBe('error');
    expect(logLevel('main: warning: no GPU found')).toBe('warn');
    expect(logLevel('srv  log_server_r: request: GET /health 200')).toBe('info');
  });

  it('does not match keywords inside other words', () => {
    expect(logLevel('n_ctx_train = 8192, terror = 0')).toBe('info');
  });
});
