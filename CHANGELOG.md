# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
