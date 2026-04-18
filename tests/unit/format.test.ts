import { describe, expect, it } from 'vitest';

import { errorMessage, formatDate } from '../../src/lib/utils/format';

describe('formatDate', () => {
  it('renders an ISO timestamp', () => {
    const out = formatDate('2026-04-18T12:34:56Z');
    expect(out).toMatch(/2026/);
  });
});

describe('errorMessage', () => {
  it('returns plain strings as-is', () => {
    expect(errorMessage('boom')).toBe('boom');
  });

  it('extracts message from Error', () => {
    expect(errorMessage(new Error('nope'))).toBe('nope');
  });

  it('falls back to String() for unknown shapes', () => {
    expect(errorMessage(42)).toBe('42');
    expect(errorMessage({ kind: 'weird' })).toBe('[object Object]');
  });

  it('handles null / undefined', () => {
    expect(errorMessage(null)).toBe('null');
    expect(errorMessage(undefined)).toBe('undefined');
  });
});
