<script lang="ts">
  import { untrack } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import CircleCheck from '@lucide/svelte/icons/circle-check';
  import CircleX from '@lucide/svelte/icons/circle-x';
  import FolderOpen from '@lucide/svelte/icons/folder-open';
  import LoaderCircle from '@lucide/svelte/icons/loader-circle';
  import Monitor from '@lucide/svelte/icons/monitor';
  import Moon from '@lucide/svelte/icons/moon';
  import Sun from '@lucide/svelte/icons/sun';
  import { toast } from 'svelte-sonner';

  import Field from '$lib/components/common/Field.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import type { Settings } from '$lib/types';
  import { cn } from '$lib/utils/cn';
  import { notifyError } from '$lib/utils/notify';

  const SECTIONS = [
    { id: 'server', label: 'llama-server' },
    { id: 'network', label: 'Network' },
    { id: 'security', label: 'Security' },
    { id: 'models', label: 'Models' },
    { id: 'behavior', label: 'Behavior' },
    { id: 'appearance', label: 'Appearance' },
  ] as const;

  const VERBOSITY = [
    { value: '0', label: '0 — generic' },
    { value: '1', label: '1 — error' },
    { value: '2', label: '2 — warning' },
    { value: '3', label: '3 — info (default)' },
    { value: '4', label: '4 — debug' },
  ];

  const THEMES = [
    { value: 'system', label: 'System', icon: Monitor },
    { value: 'light', label: 'Paper', icon: Sun },
    { value: 'dark', label: 'Graphite', icon: Moon },
  ] as const;

  const clone = (s: Settings): Settings => ({ ...s });
  let base = $state(clone(settingsStore.current));
  let draft = $state(clone(settingsStore.current));
  let saving = $state(false);
  let active = $state<string>('server');
  let scroller = $state<HTMLDivElement | undefined>(undefined);

  const dirty = $derived(JSON.stringify(draft) !== JSON.stringify(base));

  $effect(() => {
    const next = settingsStore.current;
    untrack(() => {
      if (!dirty) draft = clone(next);
      base = clone(next);
    });
  });

  async function save() {
    saving = true;
    try {
      await settingsStore.save({
        ...draft,
        port: Number(draft.port),
        timeout_secs: Number(draft.timeout_secs),
      });
      await settingsStore.checkLlamaPath();
      void themeStore.subscribe();
      toast.success('Settings saved');
    } catch (e) {
      notifyError(e);
    } finally {
      saving = false;
    }
  }

  async function pickDir() {
    try {
      const dir = await openDialog({ directory: true, defaultPath: draft.model_dir || undefined });
      if (typeof dir === 'string') draft.model_dir = dir;
    } catch (e) {
      notifyError(e);
    }
  }

  async function pickFile(key: 'ssl_cert_file' | 'ssl_key_file', extensions: string[]) {
    try {
      const f = await openDialog({ multiple: false, filters: [{ name: 'PEM', extensions }] });
      if (typeof f === 'string') draft[key] = f;
    } catch (e) {
      notifyError(e);
    }
  }

  function jump(id: string) {
    active = id;
    document
      .getElementById(`settings-${id}`)
      ?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  function onScroll() {
    if (!scroller) return;
    const top = scroller.getBoundingClientRect().top + 24;
    for (const s of [...SECTIONS].reverse()) {
      const el = document.getElementById(`settings-${s.id}`);
      if (el && el.getBoundingClientRect().top <= top) {
        active = s.id;
        return;
      }
    }
    active = SECTIONS[0].id;
  }

  const pathStatus = $derived(settingsStore.llamaPathStatus);
</script>

{#snippet toggle(
  id: string,
  label: string,
  hint: string,
  key: 'webui_enabled' | 'metrics_enabled' | 'slots_enabled' | 'keep_server_on_exit',
)}
  <div class="flex items-start justify-between gap-6 py-3">
    <Label for={id} class="flex flex-col items-start gap-0.5">
      <span>{label}</span>
      <span class="text-xs font-normal text-faint">{hint}</span>
    </Label>
    <Switch {id} bind:checked={draft[key]} />
  </div>
{/snippet}

<div class="flex min-h-0 flex-1">
  <nav class="w-48 shrink-0 border-r px-3 py-6">
    <h1 class="mb-3 px-2 text-base font-semibold">Settings</h1>
    {#each SECTIONS as s (s.id)}
      <button
        type="button"
        class={cn(
          'block w-full rounded-md px-2 py-1.5 text-left text-muted-foreground hover:bg-accent hover:text-foreground',
          active === s.id && 'bg-accent font-medium text-foreground',
        )}
        onclick={() => jump(s.id)}
      >
        {s.label}
      </button>
    {/each}
  </nav>

  <div class="flex min-w-0 flex-1 flex-col">
    <div bind:this={scroller} onscroll={onScroll} class="min-h-0 flex-1 overflow-y-auto">
      <div class="mx-auto flex max-w-[720px] flex-col gap-8 px-8 py-6 pb-24">
        {#if serverStore.anyRunning()}
          <p
            class="rounded-lg border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning"
          >
            A server is running. Changes apply to the next launch.
          </p>
        {/if}

        <section id="settings-server" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">llama-server</h2>
          <Card.Root class="py-5">
            <Card.Content class="flex flex-col gap-4 px-5">
              <Field
                label="Executable"
                for="s-llama-path"
                hint="Absolute path, or just llama-server if it is on your PATH."
                error={pathStatus === 'error'
                  ? settingsStore.llamaPathError || 'Not found'
                  : undefined}
              >
                <div class="relative">
                  <Input
                    id="s-llama-path"
                    bind:value={draft.llama_server_path}
                    oninput={() => settingsStore.resetLlamaPathStatus()}
                    placeholder="llama-server"
                    class="pr-24 font-mono text-xs"
                    spellcheck="false"
                    aria-invalid={pathStatus === 'error' || undefined}
                  />
                  <span
                    class="pointer-events-none absolute top-1/2 right-2.5 flex -translate-y-1/2 items-center gap-1 text-[11px] font-medium"
                  >
                    {#if pathStatus === 'checking'}
                      <LoaderCircle class="size-3.5 animate-spin text-faint" />
                    {:else if pathStatus === 'ok'}
                      <CircleCheck class="size-3.5 text-success" /><span class="text-success"
                        >Found</span
                      >
                    {:else if pathStatus === 'error'}
                      <CircleX class="size-3.5 text-destructive" />
                    {/if}
                  </span>
                </div>
              </Field>
              {#if pathStatus === 'ok' && settingsStore.llamaServerInfo}
                {@const info = settingsStore.llamaServerInfo}
                <dl
                  class="grid grid-cols-[90px_1fr] gap-x-3 gap-y-1.5 rounded-md bg-code p-3 text-xs"
                >
                  <dt class="text-faint">Version</dt>
                  <dd class="font-mono">{info.version || 'unknown'}</dd>
                  {#if info.compiler}
                    <dt class="text-faint">Build</dt>
                    <dd class="truncate font-mono" title={info.compiler}>{info.compiler}</dd>
                  {/if}
                  {#each info.gpu_devices as gpu, i (i)}
                    <dt class="text-faint">
                      {i === 0 ? `GPU${info.gpu_devices.length > 1 ? 's' : ''}` : ''}
                    </dt>
                    <dd>
                      {gpu.name}
                      {#if gpu.vram_mib > 0}
                        <span class="text-faint">· {(gpu.vram_mib / 1024).toFixed(1)} GiB</span>
                      {/if}
                      {#if gpu.compute_capability}
                        <span class="text-faint">· CC {gpu.compute_capability}</span>
                      {/if}
                    </dd>
                  {/each}
                </dl>
              {/if}
              <Field label="Log verbosity" for="s-verbosity">
                <Select.Root
                  type="single"
                  value={String(draft.log_verbosity)}
                  onValueChange={(v) => (draft.log_verbosity = Number(v))}
                >
                  <Select.Trigger id="s-verbosity" class="w-60">
                    {VERBOSITY.find((v) => v.value === String(draft.log_verbosity))?.label}
                  </Select.Trigger>
                  <Select.Content>
                    {#each VERBOSITY as v (v.value)}
                      <Select.Item value={v.value}>{v.label}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
              </Field>
            </Card.Content>
          </Card.Root>
        </section>

        <section id="settings-network" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">Network</h2>
          <Card.Root class="py-5">
            <Card.Content class="grid grid-cols-[1fr_140px] gap-4 px-5">
              <Field label="Host" for="s-host" hint="Use 0.0.0.0 to listen on all interfaces.">
                <Input
                  id="s-host"
                  bind:value={draft.host}
                  placeholder="127.0.0.1"
                  class="font-mono text-xs"
                />
              </Field>
              <Field label="Port" for="s-port">
                <Input
                  id="s-port"
                  type="number"
                  min={1}
                  max={65535}
                  bind:value={draft.port}
                  class="font-mono text-xs"
                />
              </Field>
              <Field label="API prefix" for="s-prefix" hint="e.g. /llama, no trailing slash.">
                <Input
                  id="s-prefix"
                  bind:value={draft.api_prefix}
                  placeholder="(none)"
                  class="font-mono text-xs"
                />
              </Field>
              <Field label="Timeout (s)" for="s-timeout">
                <Input
                  id="s-timeout"
                  type="number"
                  min={1}
                  max={86400}
                  bind:value={draft.timeout_secs}
                  class="font-mono text-xs"
                />
              </Field>
            </Card.Content>
          </Card.Root>
        </section>

        <section id="settings-security" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">Security</h2>
          <Card.Root class="py-5">
            <Card.Content class="flex flex-col gap-4 px-5">
              <Field
                label="API key"
                for="s-api-key"
                hint="Passed to llama-server via LLAMA_API_KEY. Stored in the system keychain."
              >
                <Input
                  id="s-api-key"
                  type="password"
                  bind:value={draft.api_key}
                  placeholder="(none)"
                  autocomplete="off"
                />
              </Field>
              <Field
                label="TLS certificate"
                for="s-cert"
                hint="PEM. Set together with the key to serve HTTPS."
              >
                <div class="flex gap-2">
                  <Input
                    id="s-cert"
                    bind:value={draft.ssl_cert_file}
                    placeholder="(none — HTTP only)"
                    class="font-mono text-xs"
                  />
                  <Button
                    variant="outline"
                    onclick={() => pickFile('ssl_cert_file', ['pem', 'crt', 'cer'])}>Browse</Button
                  >
                </div>
              </Field>
              <Field label="TLS private key" for="s-key">
                <div class="flex gap-2">
                  <Input
                    id="s-key"
                    bind:value={draft.ssl_key_file}
                    placeholder="(none — HTTP only)"
                    class="font-mono text-xs"
                  />
                  <Button variant="outline" onclick={() => pickFile('ssl_key_file', ['pem', 'key'])}
                    >Browse</Button
                  >
                </div>
              </Field>
            </Card.Content>
          </Card.Root>
        </section>

        <section id="settings-models" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">Models</h2>
          <Card.Root class="py-5">
            <Card.Content class="flex flex-col gap-4 px-5">
              <Field
                label="Model directory"
                for="s-model-dir"
                hint="Scanned for .gguf files; downloads are saved here."
              >
                <div class="flex gap-2">
                  <Input
                    id="s-model-dir"
                    bind:value={draft.model_dir}
                    placeholder="~/Models"
                    class="font-mono text-xs"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label="Choose folder"
                    onclick={pickDir}
                  >
                    <FolderOpen />
                  </Button>
                </div>
              </Field>
              <Field
                label="Hugging Face token"
                for="s-hf"
                hint="Used for gated downloads and set as HF_TOKEN when launching."
              >
                <Input
                  id="s-hf"
                  type="password"
                  bind:value={draft.hf_token}
                  placeholder="(none)"
                  autocomplete="off"
                />
              </Field>
            </Card.Content>
          </Card.Root>
        </section>

        <section id="settings-behavior" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">Behavior</h2>
          <Card.Root class="py-2">
            <Card.Content class="divide-y px-5">
              {@render toggle(
                's-webui',
                'Built-in Web UI',
                'Serve the llama.cpp chat UI at the server root.',
                'webui_enabled',
              )}
              {@render toggle(
                's-metrics',
                'Prometheus metrics',
                'Required for throughput charts and token totals.',
                'metrics_enabled',
              )}
              {@render toggle(
                's-slots',
                'Slots endpoint',
                'Shows per-slot activity in Live stats.',
                'slots_enabled',
              )}
              {@render toggle(
                's-keep',
                'Keep server running on quit',
                'Leave llama-server reachable after closing the app.',
                'keep_server_on_exit',
              )}
            </Card.Content>
          </Card.Root>
        </section>

        <section id="settings-appearance" class="scroll-mt-6">
          <h2 class="mb-3 text-sm font-semibold">Appearance</h2>
          <div class="grid grid-cols-3 gap-3">
            {#each THEMES as t (t.value)}
              <button
                type="button"
                class={cn(
                  'flex flex-col items-center gap-2 rounded-lg border bg-card p-4 transition-colors hover:border-input',
                  draft.theme === t.value && 'border-primary ring-2 ring-primary/25',
                )}
                aria-pressed={draft.theme === t.value}
                onclick={() => (draft.theme = t.value)}
              >
                <t.icon class="size-5 text-muted-foreground" />
                <span class="text-xs font-medium">{t.label}</span>
              </button>
            {/each}
          </div>
        </section>
      </div>
    </div>

    {#if dirty}
      <div class="flex items-center gap-3 border-t bg-sidebar px-8 py-3">
        <span class="flex-1 text-xs text-muted-foreground">You have unsaved changes.</span>
        <Button variant="ghost" onclick={() => (draft = clone(base))}>Discard</Button>
        <Button onclick={save} disabled={saving}>{saving ? 'Saving…' : 'Save changes'}</Button>
      </div>
    {/if}
  </div>
</div>
