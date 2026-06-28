import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest';
import { applyTheme } from '../../src/lib/utils/theme';

const OG = globalThis.document;

beforeEach(() => {
  const fake = { setAttribute: vi.fn(), removeAttribute: vi.fn() };
  Object.defineProperty(globalThis, 'document', {
    value: { documentElement: fake },
    writable: true,
    configurable: true,
  });
});

afterEach(() => {
  Object.defineProperty(globalThis, 'document', {
    value: OG,
    writable: true,
    configurable: true,
  });
});

describe('applyTheme', () => {
  it('sets data-theme to light', () => {
    applyTheme('light');
    expect(globalThis.document.documentElement.setAttribute).toHaveBeenCalledWith(
      'data-theme',
      'light',
    );
  });

  it('sets data-theme to dark', () => {
    applyTheme('dark');
    expect(globalThis.document.documentElement.setAttribute).toHaveBeenCalledWith(
      'data-theme',
      'dark',
    );
  });

  it('overrides previous theme', () => {
    applyTheme('dark');
    applyTheme('light');
    expect(globalThis.document.documentElement.setAttribute).toHaveBeenCalledTimes(2);
    expect(globalThis.document.documentElement.setAttribute).toHaveBeenLastCalledWith(
      'data-theme',
      'light',
    );
  });
});
