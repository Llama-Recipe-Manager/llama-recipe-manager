export type ParamType = 'number' | 'text' | 'bool' | 'select';
export type ParamGroup = 'Context & GPU' | 'Performance' | 'Sampling';

export interface ParamSpec {
  key: string;
  label: string;
  flags: string[];
  type: ParamType;
  group: ParamGroup;
  options?: string[];
  placeholder?: string;
  step?: number;
}

export type ParamValue = string | boolean | null;

const KV_TYPES = ['f16', 'bf16', 'f32', 'q8_0', 'q5_1', 'q5_0', 'q4_1', 'q4_0', 'iq4_nl'];

export const PARAMS: ParamSpec[] = [
  {
    key: 'ctx',
    label: 'Context size',
    flags: ['-c', '--ctx-size'],
    type: 'number',
    group: 'Context & GPU',
    placeholder: '4096',
  },
  {
    key: 'ngl',
    label: 'GPU layers',
    flags: ['-ngl', '--gpu-layers', '--n-gpu-layers'],
    type: 'text',
    group: 'Context & GPU',
    placeholder: '99',
  },
  {
    key: 'fa',
    label: 'Flash attention',
    flags: ['-fa', '--flash-attn'],
    type: 'select',
    group: 'Context & GPU',
    options: ['auto', 'on', 'off'],
  },
  {
    key: 'np',
    label: 'Parallel slots',
    flags: ['-np', '--parallel'],
    type: 'number',
    group: 'Context & GPU',
    placeholder: '1',
  },
  {
    key: 'ctk',
    label: 'KV cache type (K)',
    flags: ['-ctk', '--cache-type-k'],
    type: 'select',
    group: 'Context & GPU',
    options: KV_TYPES,
  },
  {
    key: 'ctv',
    label: 'KV cache type (V)',
    flags: ['-ctv', '--cache-type-v'],
    type: 'select',
    group: 'Context & GPU',
    options: KV_TYPES,
  },
  {
    key: 'threads',
    label: 'Threads',
    flags: ['-t', '--threads'],
    type: 'number',
    group: 'Performance',
    placeholder: 'auto',
  },
  {
    key: 'batch',
    label: 'Batch size',
    flags: ['-b', '--batch-size'],
    type: 'number',
    group: 'Performance',
    placeholder: '2048',
  },
  {
    key: 'ubatch',
    label: 'Micro-batch size',
    flags: ['-ub', '--ubatch-size'],
    type: 'number',
    group: 'Performance',
    placeholder: '512',
  },
  {
    key: 'jinja',
    label: 'Jinja chat template',
    flags: ['--jinja'],
    type: 'bool',
    group: 'Performance',
  },
  {
    key: 'mlock',
    label: 'Lock model in RAM',
    flags: ['--mlock'],
    type: 'bool',
    group: 'Performance',
  },
  {
    key: 'nommap',
    label: 'Disable mmap',
    flags: ['--no-mmap'],
    type: 'bool',
    group: 'Performance',
  },
  {
    key: 'temp',
    label: 'Temperature',
    flags: ['--temp', '--temperature'],
    type: 'number',
    group: 'Sampling',
    placeholder: '0.8',
    step: 0.05,
  },
  {
    key: 'topP',
    label: 'Top P',
    flags: ['--top-p'],
    type: 'number',
    group: 'Sampling',
    placeholder: '0.95',
    step: 0.01,
  },
  {
    key: 'topK',
    label: 'Top K',
    flags: ['--top-k'],
    type: 'number',
    group: 'Sampling',
    placeholder: '40',
  },
  {
    key: 'minP',
    label: 'Min P',
    flags: ['--min-p'],
    type: 'number',
    group: 'Sampling',
    placeholder: '0.05',
    step: 0.01,
  },
  {
    key: 'repeat',
    label: 'Repeat penalty',
    flags: ['--repeat-penalty'],
    type: 'number',
    group: 'Sampling',
    placeholder: '1.0',
    step: 0.05,
  },
  {
    key: 'predict',
    label: 'Max tokens',
    flags: ['-n', '--n-predict', '--predict'],
    type: 'number',
    group: 'Sampling',
    placeholder: '-1',
  },
];

export const PARAM_GROUPS: ParamGroup[] = ['Context & GPU', 'Performance', 'Sampling'];

interface Token {
  value: string;
  start: number;
  end: number;
}

export function tokenize(cmd: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < cmd.length) {
    while (i < cmd.length && /\s/.test(cmd[i])) i++;
    if (i >= cmd.length) break;
    const start = i;
    let value = '';
    while (i < cmd.length && !/\s/.test(cmd[i])) {
      const ch = cmd[i];
      if (ch === "'" || ch === '"') {
        const close = cmd.indexOf(ch, i + 1);
        const end = close === -1 ? cmd.length : close;
        value += cmd.slice(i + 1, end);
        i = end + 1;
      } else if (ch === '\\' && i + 1 < cmd.length) {
        value += cmd[i + 1];
        i += 2;
      } else {
        value += ch;
        i++;
      }
    }
    tokens.push({ value, start, end: Math.min(i, cmd.length) });
  }
  return tokens;
}

const isValueToken = (t: Token | undefined) =>
  !!t && (!t.value.startsWith('-') || /^-\d/.test(t.value));

interface Match {
  flag: Token;
  value: Token | null;
  inline: boolean;
}

function findLast(tokens: Token[], spec: ParamSpec): Match | null {
  for (let i = tokens.length - 1; i >= 0; i--) {
    const v = tokens[i].value;
    if (spec.flags.includes(v)) {
      const next = tokens[i + 1];
      return {
        flag: tokens[i],
        value: spec.type !== 'bool' && isValueToken(next) ? next : null,
        inline: false,
      };
    }
    const eq = v.indexOf('=');
    if (eq > 0 && v.startsWith('--') && spec.flags.includes(v.slice(0, eq))) {
      return { flag: tokens[i], value: null, inline: true };
    }
  }
  return null;
}

export function readParam(cmd: string, spec: ParamSpec): ParamValue {
  const m = findLast(tokenize(cmd), spec);
  if (spec.type === 'bool') return m !== null;
  if (!m) return null;
  if (m.inline) return m.flag.value.slice(m.flag.value.indexOf('=') + 1);
  if (m.value) return m.value.value;
  return spec.key === 'fa' ? 'on' : '';
}

function quote(v: string): string {
  return /^[\w@%+=:,./-]+$/.test(v) ? v : `'${v.replace(/'/g, `'\\''`)}'`;
}

function append(cmd: string, piece: string): string {
  const base = cmd.trimEnd();
  return base ? `${base} ${piece}` : piece;
}

function remove(cmd: string, start: number, end: number): string {
  let s = start;
  while (s > 0 && /[ \t]/.test(cmd[s - 1])) s--;
  const out = cmd.slice(0, s) + cmd.slice(end);
  return s === 0 ? out.replace(/^\s+/, '') : out;
}

export function writeParam(cmd: string, spec: ParamSpec, value: ParamValue): string {
  const m = findLast(tokenize(cmd), spec);
  const clear = value === null || value === false || value === '';
  if (clear) return m ? remove(cmd, m.flag.start, (m.value ?? m.flag).end) : cmd;

  const flag = spec.flags[0];
  if (spec.type === 'bool') return m ? cmd : append(cmd, flag);

  const v = quote(String(value));
  if (!m) return append(cmd, `${flag} ${v}`);
  if (m.inline) {
    const name = m.flag.value.slice(0, m.flag.value.indexOf('='));
    return cmd.slice(0, m.flag.start) + `${name}=${v}` + cmd.slice(m.flag.end);
  }
  if (m.value) return cmd.slice(0, m.value.start) + v + cmd.slice(m.value.end);
  return cmd.slice(0, m.flag.end) + ` ${v}` + cmd.slice(m.flag.end);
}

export interface ParamSummary {
  spec: ParamSpec;
  value: string;
}

/** Known params set in `cmd`, plus every other token in the order it appears. */
export function summarizeCommand(cmd: string): { params: ParamSummary[]; other: string[] } {
  const tokens = tokenize(cmd);
  const used = new Set<Token>();
  const params: ParamSummary[] = [];
  for (const spec of PARAMS) {
    const m = findLast(tokens, spec);
    if (!m) continue;
    used.add(m.flag);
    if (m.value) used.add(m.value);
    const value = readParam(cmd, spec);
    params.push({ spec, value: value === true ? 'on' : String(value) });
  }
  const other = tokens
    .filter((t, i) => !used.has(t) && !(i === 0 && /(^|\/)llama-server$/.test(t.value)))
    .map((t) => t.value);
  return { params, other };
}
