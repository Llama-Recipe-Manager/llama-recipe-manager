<script lang="ts">
  import { getTauriVersion, getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { onMount } from 'svelte';

  import CopyButton from './CopyButton.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Progress } from '$lib/components/ui/progress';
  import { appStore } from '$lib/stores/app.svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import { formatBytes } from '$lib/utils/model';
  import { notifyError } from '$lib/utils/notify';

  const REPO_URL = 'https://github.com/Llama-Recipe-Manager/llama-recipe-manager';
  const LINKS = [
    { label: 'GitHub', url: REPO_URL },
    { label: 'Report an issue', url: `${REPO_URL}/issues/new/choose` },
    { label: 'Changelog', url: 'https://llama-recipe-manager.github.io/changelog.html' },
  ];

  let appVersion = $state('');
  let tauriVersion = $state('');
  let lastCheckedAt = $state<number | null>(null);

  onMount(async () => {
    try {
      [appVersion, tauriVersion] = await Promise.all([getVersion(), getTauriVersion()]);
    } catch {
      /* only available inside the Tauri runtime */
    }
  });

  $effect(() => {
    if (updaterStore.checkedOnce && updaterStore.status !== 'checking') lastCheckedAt = Date.now();
  });

  const pct = $derived.by(() => {
    const p = updaterStore.progress;
    return p?.total ? Math.min(100, (p.downloaded / p.total) * 100) : null;
  });

  function ago(ms: number | null): string {
    if (ms === null) return 'never';
    const sec = Math.round(Math.max(0, Date.now() - ms) / 1000);
    if (sec < 60) return 'just now';
    const min = Math.round(sec / 60);
    if (min < 60) return `${min}m ago`;
    const hr = Math.round(min / 60);
    return hr < 24 ? `${hr}h ago` : `${Math.round(hr / 24)}d ago`;
  }

  async function open(url: string) {
    try {
      await openUrl(url);
    } catch (e) {
      notifyError(e);
    }
  }

  const busy = $derived(
    updaterStore.status === 'checking' || updaterStore.status === 'downloading',
  );
  const versionText = $derived(
    `Llama Recipe Manager v${appVersion} (Tauri v${tauriVersion}, ${typeof navigator !== 'undefined' ? navigator.platform : ''})`,
  );
</script>

<Dialog.Root bind:open={appStore.aboutOpen}>
  <Dialog.Content class="sm:max-w-md">
    <div class="flex items-center gap-3">
      <img src="/icon.png" alt="" class="size-12 rounded-xl" />
      <div class="min-w-0 flex-1">
        <Dialog.Title class="text-base">Llama Recipe Manager</Dialog.Title>
        <Dialog.Description class="flex items-center gap-1 font-mono text-xs">
          v{appVersion || '—'}
          <CopyButton text={versionText} label="Copy version info" />
        </Dialog.Description>
      </div>
      {#if updaterStore.status === 'available'}
        <Badge>Update v{updaterStore.info?.version}</Badge>
      {:else if updaterStore.status === 'error'}
        <Badge variant="destructive">Check failed</Badge>
      {:else if updaterStore.checkedOnce && updaterStore.status === 'idle'}
        <Badge variant="secondary">Up to date</Badge>
      {/if}
    </div>

    <dl class="grid grid-cols-2 gap-3 rounded-lg bg-code p-3 text-xs">
      <div>
        <dt class="text-faint">Runtime</dt>
        <dd>{tauriVersion ? `Tauri ${tauriVersion}` : '—'}</dd>
      </div>
      <div>
        <dt class="text-faint">Updates</dt>
        <dd>GitHub Releases (signed)</dd>
      </div>
      <div>
        <dt class="text-faint">Last checked</dt>
        <dd>{ago(lastCheckedAt)}</dd>
      </div>
      <div>
        <dt class="text-faint">License</dt>
        <dd>MIT</dd>
      </div>
    </dl>

    {#if updaterStore.status === 'downloading'}
      <div class="flex flex-col gap-1.5 text-xs">
        <div class="flex justify-between text-muted-foreground">
          <span>Downloading update…</span>
          <span class="font-mono">
            {formatBytes(updaterStore.progress?.downloaded ?? 0)}
            {#if updaterStore.progress?.total}/ {formatBytes(updaterStore.progress.total)}{/if}
          </span>
        </div>
        <Progress value={pct ?? 0} class="h-1.5" />
      </div>
    {/if}

    {#if updaterStore.error}
      <p class="text-xs text-destructive">{updaterStore.error}</p>
    {/if}

    <div class="flex items-center gap-2">
      {#if updaterStore.status === 'available'}
        <Button onclick={() => updaterStore.install()}>
          Install v{updaterStore.info?.version} &amp; restart
        </Button>
      {:else}
        <Button variant="outline" onclick={() => updaterStore.check()} disabled={busy}>
          {updaterStore.status === 'checking' ? 'Checking…' : 'Check for updates'}
        </Button>
      {/if}
    </div>

    <div class="flex items-center justify-between border-t pt-3 text-xs text-faint">
      <span>© 2026 Mohammad Ashar Khan</span>
      <span class="flex gap-3">
        {#each LINKS as link (link.url)}
          <button type="button" class="hover:text-foreground" onclick={() => open(link.url)}>
            {link.label}
          </button>
        {/each}
      </span>
    </div>
  </Dialog.Content>
</Dialog.Root>
