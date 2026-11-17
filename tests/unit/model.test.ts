import { describe, expect, it } from 'vitest';

import { fileName, formatBytes, quantOf, recipeMeta } from '../../src/lib/utils/model';

describe('model utils', () => {
  it('extracts file names across separators', () => {
    expect(fileName('/a/b/model.gguf')).toBe('model.gguf');
    expect(fileName('C:\\m\\x.gguf')).toBe('x.gguf');
  });

  it('detects quantization tags', () => {
    expect(quantOf('/m/Qwen3-30B-A3B-Q4_K_M.gguf')).toBe('Q4_K_M');
    expect(quantOf('llama-3.1-8b-instruct.IQ3_XXS.gguf')).toBe('IQ3_XXS');
    expect(quantOf('gemma-3-27b-it-bf16.gguf')).toBe('BF16');
    expect(quantOf('gpt-oss-20b-mxfp4.gguf')).toBe('MXFP4');
    expect(quantOf('mystery.gguf')).toBeNull();
  });

  it('builds sidebar meta from quant and gpu, falling back to tags', () => {
    expect(recipeMeta({ model_path: 'a-Q8_0.gguf', gpu_info: 'RTX 4090', tags: 'x' })).toBe(
      'Q8_0 · RTX 4090',
    );
    expect(recipeMeta({ model_path: 'a.gguf', gpu_info: '', tags: 'chat' })).toBe('chat');
  });

  it('formats byte sizes', () => {
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(18.6 * 1024 ** 3)).toBe('18.6 GB');
  });
});
