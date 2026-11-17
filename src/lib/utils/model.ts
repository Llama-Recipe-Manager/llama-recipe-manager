export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

const QUANT_RE = /(?:^|[-_.])((?:I?Q\d(?:_[0-9A-Z]+)*)|BF16|F16|F32|FP16|MXFP4)(?=[-_.]|$)/i;

export function quantOf(path: string): string | null {
  const name = fileName(path).replace(/\.gguf$/i, '');
  const m = QUANT_RE.exec(name);
  return m ? m[1].toUpperCase() : null;
}

export function recipeMeta(r: { model_path: string; gpu_info: string; tags: string }): string {
  const parts = [quantOf(r.model_path), r.gpu_info.trim()].filter(Boolean);
  return parts.length ? parts.join(' · ') : r.tags;
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 100 ? 0 : 1)} ${units[i]}`;
}
