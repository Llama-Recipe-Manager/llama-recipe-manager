import { describe, expect, it } from 'vitest';

import { ripeFileName } from '$lib/utils/ripe';

describe('ripeFileName', () => {
  it('slugifies the recipe name', () => {
    expect(ripeFileName('Qwen3 35B Q4 (fast)')).toBe('qwen3-35b-q4-fast.ripe');
  });

  it('strips path separators and leading/trailing punctuation', () => {
    expect(ripeFileName('../etc/passwd')).toBe('etc-passwd.ripe');
  });

  it('falls back when nothing usable is left', () => {
    expect(ripeFileName('🦙🦙')).toBe('recipe.ripe');
  });
});
