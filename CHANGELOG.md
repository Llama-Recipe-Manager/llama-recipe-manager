# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
