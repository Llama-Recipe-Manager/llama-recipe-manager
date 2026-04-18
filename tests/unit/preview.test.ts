import { describe, expect, it } from 'vitest';

import type { Settings } from '../../src/lib/types';
import { previewCommand } from '../../src/lib/utils/preview';

const baseSettings: Settings = {
  host: '127.0.0.1',
  port: 8080,
  model_dir: '/models',
  llama_server_path: '/usr/local/bin/llama-server',
  api_key: '',
  ssl_cert_file: '',
  ssl_key_file: '',
  hf_token: '',
  webui_enabled: true,
  metrics_enabled: false,
  slots_enabled: true,
  api_prefix: '',
  timeout_secs: 600,
  log_verbosity: 3,
  keep_server_on_exit: false,
};

describe('previewCommand', () => {
  it('builds a minimal preview', () => {
    const out = previewCommand(baseSettings, '--ctx-size 8192', '/models/m.gguf');
    expect(out).toContain('/usr/local/bin/llama-server');
    expect(out).toContain('--ctx-size 8192');
    expect(out).toContain('--host 127.0.0.1');
    expect(out).toContain('--port 8080');
    expect(out).toContain('-m /models/m.gguf');
  });

  it('strips a leading "llama-server " token from recipe args', () => {
    const out = previewCommand(baseSettings, 'llama-server --ctx-size 8192');
    expect(out).not.toMatch(/llama-server\s+llama-server/);
    expect(out).toContain('--ctx-size 8192');
  });

  it('appends --mmproj when provided', () => {
    const out = previewCommand(baseSettings, '', '/m.gguf', '/proj.gguf');
    expect(out).toContain('--mmproj /proj.gguf');
  });

  it('redacts api_key in output', () => {
    const out = previewCommand({ ...baseSettings, api_key: 'super-secret-token' }, '');
    expect(out).toContain('--api-key');
    expect(out).not.toContain('super-secret-token');
  });

  it('redacts very short api keys with stars only', () => {
    const out = previewCommand({ ...baseSettings, api_key: 'abc' }, '');
    expect(out).toContain('--api-key ***');
  });

  it('emits TLS pair only when both cert and key are set', () => {
    const noKey = previewCommand({ ...baseSettings, ssl_cert_file: '/c.pem' }, '');
    expect(noKey).not.toContain('--ssl-cert-file');
    const both = previewCommand(
      { ...baseSettings, ssl_cert_file: '/c.pem', ssl_key_file: '/k.pem' },
      '',
    );
    expect(both).toContain('--ssl-cert-file /c.pem');
    expect(both).toContain('--ssl-key-file /k.pem');
  });

  it('adds --no-webui when webui is disabled', () => {
    const off = previewCommand({ ...baseSettings, webui_enabled: false }, '');
    expect(off).toContain('--no-webui');
    const on = previewCommand(baseSettings, '');
    expect(on).not.toContain('--no-webui');
  });

  it('adds --metrics only when enabled, --no-slots only when disabled', () => {
    const out = previewCommand(
      { ...baseSettings, metrics_enabled: true, slots_enabled: false },
      '',
    );
    expect(out).toContain('--metrics');
    expect(out).toContain('--no-slots');
  });

  it('omits non-default flags when at defaults', () => {
    const out = previewCommand(baseSettings, '');
    expect(out).not.toContain('--api-prefix');
    expect(out).not.toContain('--timeout');
    expect(out).not.toContain('--log-verbosity');
  });

  it('emits non-default timeout and log-verbosity', () => {
    const out = previewCommand({ ...baseSettings, timeout_secs: 30, log_verbosity: 5 }, '');
    expect(out).toContain('--timeout 30');
    expect(out).toContain('--log-verbosity 5');
  });

  it('falls back to "llama-server" when no path configured', () => {
    const out = previewCommand({ ...baseSettings, llama_server_path: '' }, '');
    expect(out.startsWith('llama-server ')).toBe(true);
  });
});
