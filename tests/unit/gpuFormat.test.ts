import { describe, expect, it } from 'vitest';

import { formatGpuDevices, formatVram } from '../../src/lib/utils/gpuFormat';
import type { GpuDevice } from '../../src/lib/types';

function dev(name: string, vramMib = 0, computeCapability = ''): GpuDevice {
  return { name, vram_mib: vramMib, compute_capability: computeCapability };
}

describe('formatVram', () => {
  it('returns empty for 0 or negative', () => {
    expect(formatVram(0)).toBe('');
    expect(formatVram(-1)).toBe('');
  });

  it('renders sub-GiB values in MB', () => {
    expect(formatVram(512)).toBe('512 MB');
  });

  it('renders GiB values, rounding whole numbers ≥ 16', () => {
    expect(formatVram(24576)).toBe('24 GB');
    expect(formatVram(8 * 1024)).toBe('8.0 GB');
  });
});

describe('formatGpuDevices', () => {
  it('returns empty for no devices', () => {
    expect(formatGpuDevices([])).toBe('');
  });

  it('formats a single device with VRAM', () => {
    expect(formatGpuDevices([dev('NVIDIA RTX 4090', 24576)])).toBe('NVIDIA RTX 4090 24 GB');
  });

  it('groups identical devices and prefixes a count', () => {
    const two = [dev('NVIDIA RTX 4090', 24576), dev('NVIDIA RTX 4090', 24576)];
    expect(formatGpuDevices(two)).toBe('2 × NVIDIA RTX 4090 24 GB');
  });

  it('treats case-variant duplicates as the same device', () => {
    const two = [dev('NVIDIA RTX 4090', 24576), dev('nvidia rtx 4090', 24576)];
    expect(formatGpuDevices(two)).toBe('2 × NVIDIA RTX 4090 24 GB');
  });

  it('joins distinct devices with a comma', () => {
    const mixed = [dev('Apple M4 Max', 128 * 1024), dev('Apple M4 Max', 128 * 1024), dev('CPU')];
    expect(formatGpuDevices(mixed)).toBe('2 × Apple M4 Max 128 GB, CPU');
  });

  it('omits VRAM when a device reports none', () => {
    expect(formatGpuDevices([dev('CPU')])).toBe('CPU');
  });

  it('appends compute capability when present', () => {
    expect(formatGpuDevices([dev('NVIDIA RTX 4090', 24576, '8.9')])).toBe(
      'NVIDIA RTX 4090 24 GB (CC 8.9)',
    );
  });
});
