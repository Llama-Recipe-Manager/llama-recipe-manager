export type LogLevel = 'error' | 'warn' | 'info';

const PREFIX_RE = /^\d+\.\d+\.\d+\.\d+ ([EWID]) /;
const ERROR_RE = /\b(error|failed|fatal|abort(ed)?)\b/i;
const WARN_RE = /\bwarn(ing)?\b/i;

/** llama-server writes everything to stderr, so the stream can't be used as a severity signal. */
export function logLevel(line: string): LogLevel {
  const prefix = PREFIX_RE.exec(line)?.[1];
  if (prefix === 'E') return 'error';
  if (prefix === 'W') return 'warn';
  if (prefix) return 'info';
  if (ERROR_RE.test(line)) return 'error';
  if (WARN_RE.test(line)) return 'warn';
  return 'info';
}
