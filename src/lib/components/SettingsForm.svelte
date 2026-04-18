<script lang="ts">
  import FilePickerInput from './FilePickerInput.svelte';
  import ValidatedPathInput from './ValidatedPathInput.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { errorMessage } from '$lib/utils/format';

  let {
    onBack,
    onError,
  }: {
    onBack: () => void;
    onError: (msg: string) => void;
  } = $props();

  let host = $state(settingsStore.current.host);
  let port = $state(settingsStore.current.port);
  let modelDir = $state(settingsStore.current.model_dir);
  let llamaPath = $state(settingsStore.current.llama_server_path);
  let apiKey = $state(settingsStore.current.api_key);
  let sslCertFile = $state(settingsStore.current.ssl_cert_file);
  let sslKeyFile = $state(settingsStore.current.ssl_key_file);
  let hfToken = $state(settingsStore.current.hf_token);
  let webuiEnabled = $state(settingsStore.current.webui_enabled);
  let metricsEnabled = $state(settingsStore.current.metrics_enabled);
  let slotsEnabled = $state(settingsStore.current.slots_enabled);
  let apiPrefix = $state(settingsStore.current.api_prefix);
  let timeoutSecs = $state(settingsStore.current.timeout_secs);
  let logVerbosity = $state(settingsStore.current.log_verbosity);
  let keepServerOnExit = $state(settingsStore.current.keep_server_on_exit);
  let saved = $state(false);

  $effect(() => {
    void settingsStore.current.llama_server_path;
    settingsStore.checkLlamaPath();
  });

  async function save() {
    try {
      await settingsStore.save({
        host,
        port,
        model_dir: modelDir,
        llama_server_path: llamaPath,
        api_key: apiKey,
        ssl_cert_file: sslCertFile,
        ssl_key_file: sslKeyFile,
        hf_token: hfToken,
        webui_enabled: webuiEnabled,
        metrics_enabled: metricsEnabled,
        slots_enabled: slotsEnabled,
        api_prefix: apiPrefix,
        timeout_secs: timeoutSecs,
        log_verbosity: logVerbosity,
        keep_server_on_exit: keepServerOnExit,
      });
      saved = true;
      setTimeout(() => (saved = false), 2000);
      await settingsStore.checkLlamaPath();
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  function onLlamaPathInput() {
    settingsStore.resetLlamaPathStatus();
  }
</script>

<div class="form-view">
  <div class="form-header">
    <h2>Settings</h2>
    <div class="form-actions">
      <button class="btn secondary" onclick={onBack}>Back</button>
      <button class="btn primary" onclick={save}>{saved ? 'Saved' : 'Save'}</button>
    </div>
  </div>

  <div class="form-body">
    <p class="settings-desc">
      These settings control how llama-server is launched. Host and port are always injected into
      the command at runtime.
    </p>

    <div class="form-group">
      <label for="s-llama-path">llama-server Path</label>
      <ValidatedPathInput
        id="s-llama-path"
        bind:value={llamaPath}
        placeholder="llama-server"
        status={settingsStore.llamaPathStatus}
        errorMessage={settingsStore.llamaPathError}
        onInput={onLlamaPathInput}
      />
      {#if settingsStore.llamaPathStatus !== 'error'}
        <span class="form-hint">Absolute path or just "llama-server" if it's in your PATH</span>
      {/if}

      {#if settingsStore.llamaPathStatus === 'ok' && settingsStore.llamaServerInfo}
        {@const info = settingsStore.llamaServerInfo}
        <div class="server-info">
          <div class="server-info-row">
            <span class="server-info-label">Version</span>
            <span class="server-info-value">{info.version || 'unknown'}</span>
          </div>
          {#if info.compiler}
            <div class="server-info-row">
              <span class="server-info-label">Build</span>
              <span class="server-info-value">{info.compiler}</span>
            </div>
          {/if}
          {#if info.gpu_devices.length > 0}
            <div class="server-info-row">
              <span class="server-info-label">GPU{info.gpu_devices.length > 1 ? 's' : ''}</span>
              <div class="server-info-value">
                {#each info.gpu_devices as gpu}
                  <div class="gpu-line">
                    {gpu.name}
                    {#if gpu.vram_mib > 0}
                      <span class="gpu-meta">— {(gpu.vram_mib / 1024).toFixed(1)} GiB VRAM</span>
                    {/if}
                    {#if gpu.compute_capability}
                      <span class="gpu-meta">· CC {gpu.compute_capability}</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="form-row">
      <div class="form-group flex-1">
        <label for="s-host">Host</label>
        <input id="s-host" type="text" bind:value={host} placeholder="127.0.0.1" />
      </div>
      <div class="form-group" style="width: 140px">
        <label for="s-port">Port</label>
        <input id="s-port" type="number" bind:value={port} min="1" max="65535" />
      </div>
    </div>

    <div class="form-group">
      <label for="s-model-dir">Default Model Directory</label>
      <input id="s-model-dir" type="text" bind:value={modelDir} placeholder="~/Models" />
      <span class="form-hint"
        >Relative model paths in recipes will be resolved against this directory</span
      >
    </div>

    <h3 class="section-heading">Security</h3>

    <div class="form-group">
      <label for="s-api-key">API Key</label>
      <input
        id="s-api-key"
        type="password"
        bind:value={apiKey}
        placeholder="(none)"
        autocomplete="off"
      />
      <span class="form-hint">Injected as --api-key. Leave empty to disable authentication.</span>
    </div>

    <div class="form-group">
      <label for="s-hf-token">Hugging Face Token</label>
      <input
        id="s-hf-token"
        type="password"
        bind:value={hfToken}
        placeholder="(none)"
        autocomplete="off"
      />
      <span class="form-hint"
        >Set as HF_TOKEN env var when launching. Used for HF model downloads.</span
      >
    </div>

    <div class="form-group">
      <label for="s-ssl-cert">TLS Certificate</label>
      <FilePickerInput
        bind:value={sslCertFile}
        placeholder="(none — HTTP only)"
        extensions={['pem', 'crt', 'cer']}
        filterName="TLS certificate"
      />
      <span class="form-hint"
        >PEM-encoded certificate. Required together with TLS key for HTTPS.</span
      >
    </div>

    <div class="form-group">
      <label for="s-ssl-key">TLS Private Key</label>
      <FilePickerInput
        bind:value={sslKeyFile}
        placeholder="(none — HTTP only)"
        extensions={['pem', 'key']}
        filterName="TLS private key"
      />
      <span class="form-hint">PEM-encoded private key.</span>
    </div>

    <h3 class="section-heading">Server Behavior</h3>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={webuiEnabled} />
        <span>Enable built-in Web UI</span>
      </label>
    </div>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={metricsEnabled} />
        <span>Expose Prometheus metrics endpoint</span>
      </label>
    </div>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={slotsEnabled} />
        <span>Expose slots monitoring endpoint</span>
      </label>
    </div>

    <div class="form-row">
      <div class="form-group flex-1">
        <label for="s-api-prefix">API Prefix</label>
        <input id="s-api-prefix" type="text" bind:value={apiPrefix} placeholder="(none)" />
        <span class="form-hint">e.g. /llama (no trailing slash)</span>
      </div>
      <div class="form-group" style="width: 160px">
        <label for="s-timeout">Timeout (sec)</label>
        <input id="s-timeout" type="number" bind:value={timeoutSecs} min="1" max="86400" />
      </div>
    </div>

    <h3 class="section-heading">Lifecycle</h3>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={keepServerOnExit} />
        <span>Keep llama-server running after closing the app</span>
      </label>
      <span class="form-hint"
        >Off by default — the server is terminated when you quit. Turn this on if you want to leave
        the server reachable to other tools after closing the window.</span
      >
    </div>

    <h3 class="section-heading">Diagnostics</h3>

    <div class="form-group" style="max-width: 240px">
      <label for="s-log-verbosity">Log Verbosity</label>
      <select id="s-log-verbosity" bind:value={logVerbosity}>
        <option value={0}>0 — generic</option>
        <option value={1}>1 — error</option>
        <option value={2}>2 — warning</option>
        <option value={3}>3 — info (default)</option>
        <option value={4}>4 — debug</option>
      </select>
    </div>

    {#if serverStore.anyRunning()}
      <div class="settings-warning">
        Changes will apply to the next server launch. A server is currently running.
      </div>
    {/if}
  </div>
</div>

<style>
  .form-view {
    padding: 24px 32px;
    max-width: 700px;
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }

  .form-header h2 {
    font-size: 20px;
    font-weight: 700;
  }

  .form-actions {
    display: flex;
    gap: 8px;
  }

  .form-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-group input {
    width: 100%;
  }

  .form-hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .form-row {
    display: flex;
    gap: 12px;
  }

  .flex-1 {
    flex: 1;
  }

  .settings-desc {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .server-info {
    margin-top: 8px;
    padding: 12px 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .server-info-row {
    display: flex;
    gap: 12px;
    font-size: 12px;
    align-items: baseline;
  }

  .server-info-label {
    width: 56px;
    flex-shrink: 0;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-size: 10px;
    font-weight: 600;
  }

  .server-info-value {
    color: var(--text-primary);
    font-family: var(--font-mono);
    word-break: break-word;
    flex: 1;
  }

  .gpu-line {
    line-height: 1.5;
  }

  .gpu-meta {
    color: var(--text-tertiary);
    font-size: 11px;
  }

  .section-heading {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--text-tertiary);
    margin-top: 12px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .checkbox-label input[type='checkbox'] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }

  .settings-warning {
    font-size: 13px;
    color: var(--warning);
    padding: 10px 14px;
    background: rgba(255, 159, 10, 0.08);
    border: 1px solid rgba(255, 159, 10, 0.3);
    border-radius: var(--radius-sm);
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover {
    background: var(--accent-hover);
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover {
    background: var(--border);
  }
</style>
