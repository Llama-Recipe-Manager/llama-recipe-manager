import type { ServerPhase } from '$lib/stores/server.svelte';

export function trayStatus(phase: ServerPhase, recipeName: string | null): string {
  const name = recipeName ?? 'server';
  switch (phase) {
    case 'starting':
      return `Starting ${name}…`;
    case 'ready':
      return `Running ${name}`;
    case 'stopping':
      return `Stopping ${name}…`;
    default:
      return 'No server running';
  }
}
