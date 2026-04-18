import { describe, expect, it } from 'vitest';

import { FEATURES } from '../../src/lib/featureFlags';

describe('FEATURES flag map', () => {
  it('has the expected keys and types', () => {
    expect(typeof FEATURES.community).toBe('boolean');
    expect(typeof FEATURES.account).toBe('boolean');
  });

  it('community and account are gated off by default', () => {
    expect(FEATURES.community).toBe(false);
    expect(FEATURES.account).toBe(false);
  });
});
