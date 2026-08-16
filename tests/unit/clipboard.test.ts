import { afterEach, describe, expect, it, vi } from 'vitest';

import { copyText } from '../../src/lib/utils/clipboard';

interface FakeDoc {
  createElement: ReturnType<typeof vi.fn>;
  execCommand: ReturnType<typeof vi.fn>;
  body: { appendChild: ReturnType<typeof vi.fn> };
}

function stubNavigator(writeText?: (text: string) => Promise<void>) {
  vi.stubGlobal('navigator', { clipboard: { writeText } });
}

function stubDocument(execOk: boolean): FakeDoc {
  const doc: FakeDoc = {
    createElement: vi.fn((tag: string) => ({
      _tag: tag,
      value: '',
      style: {},
      setAttribute: vi.fn(),
      select: vi.fn(),
      setSelectionRange: vi.fn(),
      remove: vi.fn(),
    })),
    execCommand: vi.fn(() => execOk),
    body: { appendChild: vi.fn() },
  };
  vi.stubGlobal('document', doc);
  return doc;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('copyText', () => {
  it('uses the async Clipboard API when available', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    stubNavigator(writeText);
    stubDocument(true);

    await expect(copyText('hello')).resolves.toBe(true);
    expect(writeText).toHaveBeenCalledWith('hello');
  });

  it('falls back to execCommand when the Clipboard API is missing', async () => {
    stubNavigator(undefined as never);
    const doc = stubDocument(true);

    await expect(copyText('fallback')).resolves.toBe(true);
    expect(doc.createElement).toHaveBeenCalledWith('textarea');
    expect(doc.execCommand).toHaveBeenCalledWith('copy');
  });

  it('falls back to execCommand when the Clipboard API rejects', async () => {
    stubNavigator(vi.fn().mockRejectedValue(new Error('denied')));
    const doc = stubDocument(true);

    await expect(copyText('retry')).resolves.toBe(true);
    expect(doc.execCommand).toHaveBeenCalledWith('copy');
  });

  it('reports failure when execCommand returns false', async () => {
    stubNavigator(undefined as never);
    stubDocument(false);

    await expect(copyText('nope')).resolves.toBe(false);
  });

  it('reports failure when there is no document for the fallback', async () => {
    stubNavigator(undefined as never);
    vi.stubGlobal('document', undefined);

    await expect(copyText('nothing')).resolves.toBe(false);
  });
});
