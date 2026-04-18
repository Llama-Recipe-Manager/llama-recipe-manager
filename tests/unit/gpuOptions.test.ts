import { describe, expect, it } from 'vitest';

import { ALL_GPU_VALUES, GPU_CUSTOM_VALUE, GPU_OPTIONS } from '../../src/lib/utils/gpuOptions';

describe('gpuOptions', () => {
  it('exposes at least one group per major vendor', () => {
    const groups = GPU_OPTIONS.map((g) => g.group.toLowerCase());
    expect(groups.some((g) => g.includes('apple'))).toBe(true);
    expect(groups.some((g) => g.includes('nvidia'))).toBe(true);
    expect(groups.some((g) => g.includes('amd'))).toBe(true);
    expect(groups.some((g) => g.includes('intel'))).toBe(true);
  });

  it('every group has at least one item', () => {
    for (const group of GPU_OPTIONS) {
      expect(group.items.length, `group ${group.group} is empty`).toBeGreaterThan(0);
    }
  });

  it('flattened list is the sum of all groups', () => {
    const sum = GPU_OPTIONS.reduce((acc, g) => acc + g.items.length, 0);
    expect(ALL_GPU_VALUES.length).toBe(sum);
  });

  it('all entries are unique', () => {
    expect(new Set(ALL_GPU_VALUES).size).toBe(ALL_GPU_VALUES.length);
  });

  it('GPU_CUSTOM_VALUE never collides with a real entry', () => {
    expect(ALL_GPU_VALUES).not.toContain(GPU_CUSTOM_VALUE);
  });

  it('every entry mentions a memory size', () => {
    for (const value of ALL_GPU_VALUES) {
      expect(value).toMatch(/\d+\s?GB/i);
    }
  });
});
