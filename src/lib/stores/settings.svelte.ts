import { getLlamaServerInfo, getSettings, updateSettings } from '$lib/api';
import type { LlamaPathStatus, LlamaServerInfo, Settings } from '$lib/types';
import { errorMessage } from '$lib/utils/format';

const DEFAULT_SETTINGS: Settings = {
  host: '127.0.0.1',
  port: 8080,
  model_dir: '~/Models',
  llama_server_path: 'llama-server',
  api_key: '',
  ssl_cert_file: '',
  ssl_key_file: '',
  hf_token: '',
  webui_enabled: true,
  metrics_enabled: true,
  slots_enabled: true,
  api_prefix: '',
  timeout_secs: 600,
  log_verbosity: 3,
  keep_server_on_exit: false,
};

class SettingsStore {
  current = $state<Settings>({ ...DEFAULT_SETTINGS });
  llamaPathStatus = $state<LlamaPathStatus>('idle');
  llamaPathError = $state('');
  llamaServerInfo = $state<LlamaServerInfo | null>(null);

  async refresh(): Promise<void> {
    this.current = await getSettings();
  }

  async save(next: Settings): Promise<void> {
    this.current = await updateSettings(next);
  }

  async checkLlamaPath(): Promise<void> {
    if (!this.current.llama_server_path.trim()) {
      this.llamaPathStatus = 'idle';
      this.llamaPathError = '';
      this.llamaServerInfo = null;
      return;
    }
    this.llamaPathStatus = 'checking';
    this.llamaPathError = '';
    try {
      this.llamaServerInfo = await getLlamaServerInfo();
      this.llamaPathStatus = 'ok';
    } catch (e) {
      this.llamaPathStatus = 'error';
      this.llamaPathError = errorMessage(e);
      this.llamaServerInfo = null;
    }
  }

  resetLlamaPathStatus(): void {
    this.llamaPathStatus = 'idle';
    this.llamaPathError = '';
    this.llamaServerInfo = null;
  }
}

export const settingsStore = new SettingsStore();
