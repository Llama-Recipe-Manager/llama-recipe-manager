import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'svelte-sonner';

import { deleteRecipe, duplicateRecipe } from '$lib/api';
import { confirm } from '$lib/stores/confirm.svelte';
import { recipesStore } from '$lib/stores/recipes.svelte';
import { serverStore } from '$lib/stores/server.svelte';
import { settingsStore } from '$lib/stores/settings.svelte';
import type { Recipe, Section } from '$lib/types';
import { notifyError } from '$lib/utils/notify';
import { exportAllRecipesToIni, exportRecipeToIni } from '$lib/utils/ini';
import { exportRecipeToFile, importRecipeFromFile } from '$lib/utils/ripe';
import { webuiUrl } from '$lib/utils/server';

export type RecipeView = 'detail' | 'new' | 'edit';
export type RecipeTab = 'overview' | 'stats' | 'logs';

class AppStore {
  section = $state<Section>('recipes');
  view = $state<RecipeView>('detail');
  tab = $state<RecipeTab>('overview');
  formDirty = $state(false);
  aboutOpen = $state(false);
  paletteOpen = $state(false);
  /** Model path to prefill when a new recipe is started from the Models section. */
  draftModel = $state<string | null>(null);

  get editing(): boolean {
    return this.section === 'recipes' && (this.view === 'new' || this.view === 'edit');
  }

  async confirmDiscard(): Promise<boolean> {
    if (!this.formDirty || !this.editing) return true;
    const ok = await confirm({
      title: 'Discard changes?',
      message: 'Your unsaved recipe changes will be lost.',
      confirmLabel: 'Discard',
      destructive: true,
    });
    if (ok) this.formDirty = false;
    return ok;
  }

  async goSection(next: Section): Promise<void> {
    if (next === this.section) return;
    if (!(await this.confirmDiscard())) return;
    this.section = next;
    if (next === 'recipes') this.view = 'detail';
    if (next === 'settings') void settingsStore.checkLlamaPath();
  }

  async selectRecipe(id: string, tab: RecipeTab = 'overview'): Promise<void> {
    if (!(await this.confirmDiscard())) return;
    this.section = 'recipes';
    this.view = 'detail';
    recipesStore.select(id);
    this.tab = tab;
  }

  async startNew(modelPath: string | null = null): Promise<void> {
    if (!(await this.confirmDiscard())) return;
    this.section = 'recipes';
    this.draftModel = modelPath;
    recipesStore.select(null);
    this.view = 'new';
  }

  startEdit(): void {
    if (recipesStore.selected) this.view = 'edit';
  }

  finishEdit(savedId?: string): void {
    this.formDirty = false;
    this.draftModel = null;
    if (savedId) recipesStore.select(savedId);
    this.view = 'detail';
  }

  async jumpToRunning(): Promise<void> {
    const id = serverStore.runningRecipeId();
    if (id) await this.selectRecipe(id);
  }

  async start(id = recipesStore.selectedId): Promise<void> {
    if (!id) return;
    try {
      await serverStore.start(id);
    } catch (e) {
      notifyError(e);
    }
  }

  async stop({ ask = true } = {}): Promise<void> {
    if (!serverStore.anyActive()) return;
    const starting = serverStore.phase === 'starting';
    if (
      ask &&
      !(await confirm(
        starting
          ? {
              title: 'Cancel start?',
              message: 'llama-server is still loading. The process will be terminated.',
              confirmLabel: 'Cancel start',
              cancelLabel: 'Keep loading',
              destructive: true,
            }
          : {
              title: 'Stop server?',
              message:
                'In-flight requests get up to 8 seconds to drain before llama-server is terminated.',
              confirmLabel: 'Stop server',
              destructive: true,
            },
      ))
    )
      return;
    try {
      await serverStore.stop();
    } catch (e) {
      notifyError(e);
    }
  }

  async openWebUi(): Promise<void> {
    try {
      await openUrl(webuiUrl(settingsStore.current));
    } catch (e) {
      notifyError(e);
    }
  }

  async importRecipe(): Promise<void> {
    if (!(await this.confirmDiscard())) return;
    try {
      const imported = await importRecipeFromFile();
      if (!imported) return;
      await recipesStore.refresh();
      await this.selectRecipe(imported.id);
      toast.success(`Imported “${imported.name}”`);
    } catch (e) {
      notifyError(e);
    }
  }

  async exportRecipe(recipe: Recipe): Promise<void> {
    try {
      await exportRecipeToFile(recipe);
    } catch (e) {
      notifyError(e);
    }
  }

  async exportRecipeIni(recipe: Recipe): Promise<void> {
    try {
      await exportRecipeToIni(recipe);
    } catch (e) {
      notifyError(e);
    }
  }

  async exportAllRecipesIni(): Promise<void> {
    try {
      await exportAllRecipesToIni();
    } catch (e) {
      notifyError(e);
    }
  }

  async togglePin(recipe: Recipe): Promise<void> {
    try {
      await recipesStore.setPinned(recipe.id, !recipe.pinned);
    } catch (e) {
      notifyError(e);
    }
  }

  async duplicate(recipe: Recipe): Promise<void> {
    try {
      const copy = await duplicateRecipe(recipe.id);
      await recipesStore.refresh();
      await this.selectRecipe(copy.id);
    } catch (e) {
      notifyError(e);
    }
  }

  async remove(recipe: Recipe): Promise<void> {
    if (serverStore.isActive(recipe.id)) {
      toast.error('Stop the server before deleting this recipe.');
      return;
    }
    const ok = await confirm({
      title: `Delete “${recipe.name}”?`,
      message: 'This cannot be undone.',
      confirmLabel: 'Delete',
      destructive: true,
    });
    if (!ok) return;
    try {
      await deleteRecipe(recipe.id);
      await recipesStore.refresh();
      if (recipesStore.selectedId === recipe.id) recipesStore.select(null);
    } catch (e) {
      notifyError(e);
    }
  }
}

export const appStore = new AppStore();
