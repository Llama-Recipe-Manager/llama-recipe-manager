# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] — 2026-11-18

### Added

- **Redesigned UI**: new Graphite (dark) and Paper (light) themes built on
  Tailwind and shadcn-svelte, a macOS overlay title bar with live server
  status, and a tabbed recipe page (Overview, Live stats, Logs). (PR #90)
- **Command palette**: ⌘K / Ctrl+K to run actions, jump between sections and
  open recipes; ⌘, / Ctrl+, opens Settings. (PR #90)
- **Models section**: browse local GGUFs and download from Hugging Face in one
  place; downloads keep running while you navigate. (PR #90)
- **Structured flag editor**: fields for common llama-server flags that stay in
  sync with the raw argument string. (PR #90)
- **System tray** with server status and start/stop. (PR #85)
- **Pinned recipes** and sorting by last updated, last used or name. (PR #86)
- **Import/export** recipes as `.ripe` files. (PR #75)
- **Cancel Hugging Face downloads**. (PR #73)
- Keyboard shortcuts: ⌘N / Ctrl+N new recipe, ⌘F / Ctrl+F search, ⌘S / Ctrl+S
  save. (PRs #69, #74)

### Changed

- Settings are split into sections with a sub-nav and a save bar. (PR #90)
- Errors appear as toasts and confirmations use in-app dialogs. (PR #90)
- GGUF type detection is cached across scans. (PR #79)

### Security

- The API key and Hugging Face token are stored in the OS keychain. (PR #81)
- A content security policy is enabled. (PR #80)
- The API key is passed via `LLAMA_API_KEY` instead of the command line.
  (PR #71)

### Fixed

- A busy port is reported before llama-server is spawned. (PR #83)
- Unsaved edits prompt before import and quit. (PR #76)
- Switching recipes no longer clears the running server's logs. (PR #90)
- Log reading is lossy-safe, batched and coloured by level; copy respects the
  active filter. (PRs #66, #72)
- Recipes launch from the database and accept Windows absolute paths. (PR #65)
- Settings are saved in a single transaction. (PR #68)
- GGUF probing runs off the main thread, HF URLs are encoded and download
  progress is throttled. (PR #67)

## [0.4.0] — 2026-08-16

### Added

- **Model scanner pagination**: the model browser now loads `.gguf` results in
  pages as you scroll (with a "Load more" fallback button), keeps memory bounded
  on directories with thousands of models, and shows a loaded/total counter.
  (PR #48)
- **Copy-to-clipboard**: the recipe detail view lets you copy the endpoint URL
  and the full launched command with one click, using a clipboard fallback for
  older WebViews. (PR #47)

### Changed

- **Accessible dialogs**: the model browser and downloader are now proper modal
  dialogs (`role="dialog"`, `aria-modal`, initial-focus management), clearing
  the remaining Svelte accessibility warnings. (PR #47)

## [0.3.1] — 2026-07-04

### Added

- **mmproj file support**: scan and download now distinguish between model
  and mmproj `.gguf` files. (PR #39)
- **Scan & Download buttons for mmproj**: the vision section in recipe form
  now has its own Scan and Download buttons, opening a browser/downloader
  filtered to mmproj files.
- **"View all ggufs" toggle**: a checkbox in both the model browser and
  downloader lets you see all `.gguf` files regardless of type.
- **Kind badges**: scanned files show a `model` or `mmproj` badge next to
  their name.

## [0.3.0] — 2026-06-28

### Added

- **Scan models directory**: discover `.gguf` files from the configured model
  directory with a searchable modal browser. (PR #30)
- **HuggingFace model download**: browse and download `.gguf` files directly
  from HF Hub. Real-time progress events, `.part` temp file safety, and
  orphaned partial cleanup on startup. (PR #30)
- **Manual theme toggle**: System / Light / Dark selector in Settings,
  persisted to the database. (PR #30)
- Theme application extracted into a pure utility function.
- Rust tests for download filename sanitisation (path traversal prevention).
- New Rust dependencies: `reqwest` (stream + json), `futures-util`.

### Changed

- Model scanner rewritten from `std::fs` to `tokio::fs` for fully async I/O.
- Downloads use `.gguf.part` temp files with atomic rename on completion.
- Frontend tests added for `applyTheme`.

### Fixed

- Theme no longer uses default `"system"` on startup — `themeStore.subscribe()`
  now runs after `settingsStore.refresh()` finishes loading persisted settings.
- Orphaned `.gguf.part` files from interrupted downloads are cleaned up on restart.

### Security

- Download filename sanitisation strips directory components to prevent path
  traversal via malicious HF API filenames.

## [0.2.0] — 2026-04-26

### Added

- `ConfirmDialog` component: reusable confirmation dialog with ARIA support,
  keyboard navigation (Tab cycling, Enter to confirm, Escape to cancel),
  and matching existing dialog styling.

### Changed

- Replace browser `window.confirm()` in recipe delete with `ConfirmDialog`.
- Add confirmation dialog before clearing server logs.
- Update all GitHub URLs from `coder3101` to `Llama-Recipe-Manager` org.
- Git remote updated to `Llama-Recipe-Manager/llama-recipe-manager`.
- Homepage URL updated to https://llama-recipe-manager.github.io.

### Fixed

- LogsPanel now filters by `recipe_id` so each recipe's panel only shows its
  own logs instead of all recipes' logs.
- LogsPanel status dot and empty-state message now reflect the current
  recipe's server state instead of the global server state.
- Windows console window flash when spawning `llama-server` — added
  `CREATE_NO_WINDOW` flag to `tokio::process::Command` in both the server
  spawn path and the version probe.

## [0.1.1] — 2026-04-18

### Fixed

- fix: NVIDIA + Wayland Crash

### Changed

- chore: Icon update for homepage

## [0.1.0] — 2026-04-18

First public release. Native desktop launcher for `llama-server` with
recipe management, signed auto-updates, live metrics, and graceful
cross-platform process lifecycle.

### Added

- Recipe-command flag deny-list to prevent recipes from overriding
  app-managed settings.
- Recipe field length / charset validation (NUL / CR / LF rejection, length
  caps).
- Path canonicalisation for relative model and mmproj paths, with
  containment under the configured `model_dir`.
- Two-pane UI shell with a nav rail (My Recipes, Settings) and a
  contextual sub-sidebar.
- Vitest test suite for frontend utilities.
- Cargo unit + integration test suite for the validation and database
  layers.
- GitHub Actions CI for lint, format, type-check, and test on Linux,
  macOS, and Windows.
- GitHub Actions release workflow that builds per-OS Tauri bundles.

### Changed

- Settings page extended with Security (API key, TLS cert/key, HF token)
  and Server-behaviour (Web UI, metrics, slots, API prefix, timeout, log
  verbosity) sections.
- HuggingFace token is now passed via the `HF_TOKEN` environment variable
  to spawned `llama-server` processes instead of as a CLI argument.
