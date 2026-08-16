import type { GpuDevice } from '$lib/types';

/**
 * Format a VRAM value (reported in MiB by llama.cpp) into a compact human
 * string, e.g. `24564` → `"24 GB"`. Values under 1 GiB fall back to MB.
 */
export function formatVram(vramMib: number): string {
  if (vramMib <= 0) return '';
  const gib = vramMib / 1024;
  if (gib < 1) return `${vramMib} MB`;
  // Whole GiB when >= 16 so "24 GB" reads cleanly instead of "24.0 GB".
  return gib >= 16 ? `${Math.round(gib)} GB` : `${gib.toFixed(1)} GB`;
}

/**
 * Build a single, human-readable GPU summary from a list of detected devices.
 *
 * - Groups identical devices (case-insensitive name) and counts them, so a
 *   multi-GPU machine reads as `"2 × NVIDIA RTX 4090 24 GB"`.
 * - Different devices are joined with a comma.
 * - Omits VRAM when a device reports none (e.g. CPU-only / unknown).
 */
export function formatGpuDevices(devices: GpuDevice[]): string {
  if (!devices || devices.length === 0) return '';

  const groups = new Map<string, { name: string; count: number; vramMib: number; cc: string }>();
  for (const d of devices) {
    const key = (d.name || '').toLowerCase();
    const existing = groups.get(key);
    if (existing) {
      existing.count += 1;
    } else {
      groups.set(key, {
        name: d.name,
        count: 1,
        vramMib: d.vram_mib,
        cc: d.compute_capability,
      });
    }
  }

  const parts: string[] = [];
  for (const g of groups.values()) {
    const base = g.name.trim();
    if (!base) continue;
    const label = [base, formatVram(g.vramMib)].filter(Boolean).join(' ');
    const withCount = g.count > 1 ? `${g.count} × ${label}` : label;
    parts.push(g.cc ? `${withCount} (CC ${g.cc})` : withCount);
  }

  return parts.filter(Boolean).join(', ');
}
