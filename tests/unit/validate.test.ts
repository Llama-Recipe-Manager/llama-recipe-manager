import { describe, expect, it } from 'vitest';

import { FORBIDDEN_FLAGS, UNSAFE_FLAGS, validateCommand } from '../../src/lib/utils/validate';

describe('validateCommand — local source (default)', () => {
  it('passes a benign command', () => {
    expect(validateCommand('--ctx-size 8192 --threads 8')).toEqual([]);
  });

  it('blocks every FORBIDDEN flag', () => {
    for (const flag of FORBIDDEN_FLAGS) {
      const errs = validateCommand(`llama-server ${flag} value`);
      expect(errs.length, `expected ${flag} to be blocked`).toBeGreaterThan(0);
      expect(errs[0]).toContain(flag);
    }
  });

  it('does NOT block UNSAFE flags for local recipes', () => {
    for (const flag of UNSAFE_FLAGS) {
      expect(
        validateCommand(`llama-server ${flag} value`),
        `expected ${flag} to be allowed locally`,
      ).toEqual([]);
    }
  });

  it('matches case-insensitively', () => {
    expect(validateCommand('llama-server --HOST 0.0.0.0')[0]).toContain('--host');
  });

  it('catches flag=value form', () => {
    expect(validateCommand('llama-server --host=0.0.0.0')[0]).toContain('--host');
  });

  it('rejects NUL bytes', () => {
    expect(validateCommand('foo\0bar')[0]).toContain('NUL');
  });
});

describe('validateCommand — community source', () => {
  it('blocks both FORBIDDEN and UNSAFE flags', () => {
    for (const flag of [...FORBIDDEN_FLAGS, ...UNSAFE_FLAGS]) {
      const errs = validateCommand(`llama-server ${flag}`, 'community');
      expect(errs.length, `expected ${flag} to be blocked for community`).toBeGreaterThan(0);
    }
  });

  it('still passes a benign command', () => {
    expect(validateCommand('--ctx-size 8192 --threads 8', 'community')).toEqual([]);
  });
});

describe('flag list invariants', () => {
  it('FORBIDDEN_FLAGS and UNSAFE_FLAGS are disjoint', () => {
    const forbidden = new Set<string>(FORBIDDEN_FLAGS);
    for (const flag of UNSAFE_FLAGS) {
      expect(forbidden.has(flag), `${flag} appears in both lists`).toBe(false);
    }
  });

  it('every flag starts with a hyphen', () => {
    for (const flag of [...FORBIDDEN_FLAGS, ...UNSAFE_FLAGS]) {
      expect(flag.startsWith('-')).toBe(true);
    }
  });
});
