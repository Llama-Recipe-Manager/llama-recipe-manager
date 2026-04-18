/**
 * Tiny HTTP client for the running llama-server. Uses the WebView's `fetch`
 * directly (no Tauri command needed) and attaches an Authorization header
 * when an API key is configured.
 */

import type { Settings } from '$lib/types';
import { webuiUrl } from '$lib/utils/server';

function headers(settings: Settings): HeadersInit {
  return settings.api_key ? { Authorization: `Bearer ${settings.api_key}` } : {};
}

async function fetchJson<T>(url: string, settings: Settings, signal: AbortSignal): Promise<T> {
  const res = await fetch(url, { headers: headers(settings), signal });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return (await res.json()) as T;
}

export async function fetchMetricsText(settings: Settings, signal: AbortSignal): Promise<string> {
  const res = await fetch(`${webuiUrl(settings)}/metrics`, {
    headers: headers(settings),
    signal,
  });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return await res.text();
}

export function fetchProps<T>(settings: Settings, signal: AbortSignal): Promise<T> {
  return fetchJson<T>(`${webuiUrl(settings)}/props`, settings, signal);
}

export function fetchSlots<T>(settings: Settings, signal: AbortSignal): Promise<T> {
  return fetchJson<T>(`${webuiUrl(settings)}/slots`, settings, signal);
}

export async function fetchHealthOk(settings: Settings, signal: AbortSignal): Promise<boolean> {
  try {
    const res = await fetch(`${webuiUrl(settings)}/health`, {
      headers: headers(settings),
      signal,
    });
    return res.ok;
  } catch {
    return false;
  }
}
