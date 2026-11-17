import { defaultWindowIcon } from '@tauri-apps/api/app';
import { Menu, PredefinedMenuItem } from '@tauri-apps/api/menu';
import { TrayIcon } from '@tauri-apps/api/tray';
import { getCurrentWindow } from '@tauri-apps/api/window';

const TRAY_ID = 'main';

export interface TrayState {
  status: string;
  action: { text: string; enabled: boolean; run: () => void } | null;
}

export async function showMainWindow(): Promise<void> {
  const win = getCurrentWindow();
  await win.show();
  await win.unminimize();
  await win.setFocus();
}

async function buildMenu(state: TrayState): Promise<Menu> {
  const items = [
    { id: 'status', text: state.status, enabled: false },
    ...(state.action
      ? [
          {
            id: 'action',
            text: state.action.text,
            enabled: state.action.enabled,
            action: state.action.run,
          },
        ]
      : []),
    await PredefinedMenuItem.new({ item: 'Separator' }),
    { id: 'show', text: 'Show window', action: () => void showMainWindow() },
    {
      id: 'quit',
      text: 'Quit',
      action: async () => {
        await showMainWindow();
        await getCurrentWindow().close();
      },
    },
  ];
  return Menu.new({ items });
}

export async function createTray(): Promise<{
  update: (state: TrayState) => void;
  dispose: () => Promise<void>;
}> {
  await TrayIcon.removeById(TRAY_ID).catch(() => {});
  const tray = await TrayIcon.new({
    id: TRAY_ID,
    icon: (await defaultWindowIcon()) ?? undefined,
    tooltip: 'Llama Recipe Manager',
  });

  let queue = Promise.resolve();
  return {
    update(state) {
      queue = queue
        .then(async () => {
          await tray.setMenu(await buildMenu(state));
          await tray.setTooltip(`Llama Recipe Manager — ${state.status}`);
        })
        .catch(() => {});
    },
    dispose: () => TrayIcon.removeById(TRAY_ID).catch(() => {}),
  };
}
