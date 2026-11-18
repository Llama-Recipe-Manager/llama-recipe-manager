<div align="center">

<img src="docs/assets/icon.png" alt="Llama Recipe Manager" width="160" height="160" />

# Llama Recipe Manager

**A native desktop launcher for `llama-server`.**

Save your `llama.cpp` invocations as named _recipes_, switch between them in one click, stop juggling flags.

[![CI](https://github.com/Llama-Recipe-Manager/llama-recipe-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/Llama-Recipe-Manager/llama-recipe-manager/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)

</div>

---

## Highlights

- **Recipe-first** — named bundles of `llama-server` flags, switch models and GPU layouts in one click
- **Local-first** — no account, no telemetry, no network dependency. Recipes stored in SQLite
- **Preset & INI Export** — export recipes directly to `llama-server --models-preset` compatible `.ini` files or `.ripe` bundles
- **Safety rails** — flag deny-list, path confinement, input validation. See [SECURITY.md](./SECURITY.md)
- **Cross-platform** — macOS, Linux, Windows with native theming

## Export & Presets

Llama Recipe Manager supports exporting your configurations for direct use with `llama-server`:

- **Single Preset Export**: Right-click any recipe in the sidebar or open the 3-dot menu in Recipe Detail, then select **Export as INI preset…** (or **Export .ripe…** for app-to-app sharing).
- **Export All to INI**: Click the **Export All** button in the Recipes sidebar header or run `export all recipes ini` from the Command Palette (`Ctrl+K` / `Cmd+K`) to generate a complete multi-model preset file with shared global `[*]` defaults.
- **Run Directly with `llama-server`**:
  ```bash
  llama-server --models-preset ./llama_models.ini
  ```
- **CLI Batch Export Tool**:
  Export directly from the local SQLite database without opening the GUI:
  ```bash
  python3 scripts/export-recipes-ini.py [path/to/recipes.db] [output.ini]
  ```

## Install

Pre-built bundles ship on the [Releases](https://github.com/Llama-Recipe-Manager/llama-recipe-manager/releases) page.

```bash
git clone https://github.com/Llama-Recipe-Manager/llama-recipe-manager.git
cd llama-recipe-manager
bun install
bun run tauri dev      # development
bun run tauri build    # produces .app / .deb / .AppImage / .msi
```

Full build instructions: [`docs/Development.md`](./docs/Development.md)

## Documentation

- [Development guide](./docs/Development.md) · [Security policy](./SECURITY.md) · [Contributing](./CONTRIBUTING.md)
- [Changelog](./CHANGELOG.md) · [Code of Conduct](./CODE_OF_CONDUCT.md)

## License

[MIT](./LICENSE) © 2026 Mohammad Ashar Khan

Built on [llama.cpp](https://github.com/ggml-org/llama.cpp), [Tauri](https://tauri.app), [SvelteKit](https://kit.svelte.dev).
