import { describe, expect, it } from 'vitest';

import { PARAMS, readParam, summarizeCommand, tokenize, writeParam } from '$lib/utils/flags';

const spec = (key: string) => PARAMS.find((p) => p.key === key)!;

describe('tokenize', () => {
  it('unquotes values but keeps source positions', () => {
    const cmd = `--alias "my model" -c 4096`;
    const tokens = tokenize(cmd);
    expect(tokens.map((t) => t.value)).toEqual(['--alias', 'my model', '-c', '4096']);
    expect(cmd.slice(tokens[1].start, tokens[1].end)).toBe('"my model"');
  });
});

describe('readParam', () => {
  it('reads short, long and inline forms', () => {
    expect(readParam('-c 8192', spec('ctx'))).toBe('8192');
    expect(readParam('--ctx-size 16384', spec('ctx'))).toBe('16384');
    expect(readParam('--ctx-size=2048', spec('ctx'))).toBe('2048');
  });

  it('uses the last occurrence, like llama-server', () => {
    expect(readParam('-c 1024 -ngl 99 -c 4096', spec('ctx'))).toBe('4096');
  });

  it('treats negative numbers as values', () => {
    expect(readParam('-n -1 --jinja', spec('predict'))).toBe('-1');
  });

  it('handles booleans and bare flash attention', () => {
    expect(readParam('--jinja -c 10', spec('jinja'))).toBe(true);
    expect(readParam('-c 10', spec('jinja'))).toBe(false);
    expect(readParam('-fa -c 10', spec('fa'))).toBe('on');
    expect(readParam('-fa off', spec('fa'))).toBe('off');
    expect(readParam('-c 10', spec('ctx'))).toBe('10');
    expect(readParam('', spec('ctx'))).toBeNull();
  });
});

describe('writeParam', () => {
  it('replaces a value in place without touching other text', () => {
    const cmd = '-ngl 99\n  -c 4096 --temp 0.6';
    expect(writeParam(cmd, spec('ctx'), '8192')).toBe('-ngl 99\n  -c 8192 --temp 0.6');
  });

  it('appends missing flags using the first alias', () => {
    expect(writeParam('-ngl 99', spec('ctx'), '8192')).toBe('-ngl 99 -c 8192');
    expect(writeParam('', spec('jinja'), true)).toBe('--jinja');
  });

  it('removes a flag and its value', () => {
    expect(writeParam('-ngl 99 -c 4096 --temp 0.6', spec('ctx'), '')).toBe('-ngl 99 --temp 0.6');
    expect(writeParam('-c 4096 --jinja', spec('ctx'), null)).toBe('--jinja');
    expect(writeParam('-c 4096 --jinja', spec('jinja'), false)).toBe('-c 4096');
  });

  it('updates inline and bare forms', () => {
    expect(writeParam('--ctx-size=2048 -t 4', spec('ctx'), '4096')).toBe('--ctx-size=4096 -t 4');
    expect(writeParam('-fa -t 4', spec('fa'), 'off')).toBe('-fa off -t 4');
  });

  it('quotes values that need it', () => {
    expect(writeParam('', spec('ngl'), 'all of them')).toBe(`-ngl 'all of them'`);
  });

  it('round-trips with readParam', () => {
    let cmd = '';
    for (const [key, value] of [
      ['ctx', '32768'],
      ['fa', 'on'],
      ['temp', '0.7'],
    ] as const) {
      cmd = writeParam(cmd, spec(key), value);
      expect(readParam(cmd, spec(key))).toBe(value);
    }
    expect(cmd).toBe('-c 32768 -fa on --temp 0.7');
  });
});

describe('summarizeCommand', () => {
  it('splits known params from everything else', () => {
    const { params, other } = summarizeCommand(
      'llama-server -c 8192 --jinja --rope-scaling yarn -ngl 99',
    );
    expect(params.map((p) => [p.spec.key, p.value])).toEqual([
      ['ctx', '8192'],
      ['ngl', '99'],
      ['jinja', 'on'],
    ]);
    expect(other).toEqual(['--rope-scaling', 'yarn']);
  });
});
