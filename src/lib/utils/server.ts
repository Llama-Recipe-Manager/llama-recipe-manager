import type { Settings } from '$lib/types';

/**
 * Build the URL the running llama-server is reachable at, honouring TLS,
 * api-prefix, and the configured host/port.
 *
 * `0.0.0.0` is rewritten to `127.0.0.1` so the link is actually clickable
 * from the local machine.
 */
export function webuiUrl(settings: Settings): string {
  const scheme = settings.ssl_cert_file && settings.ssl_key_file ? 'https' : 'http';
  const host = settings.host === '0.0.0.0' ? '127.0.0.1' : settings.host || '127.0.0.1';
  const prefix = settings.api_prefix?.startsWith('/') ? settings.api_prefix : '';
  return `${scheme}://${host}:${settings.port}${prefix}`;
}

export function formatUptime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m ${String(s).padStart(2, '0')}s`;
  if (m > 0) return `${m}m ${String(s).padStart(2, '0')}s`;
  return `${s}s`;
}
