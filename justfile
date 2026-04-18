# Llama Recipe Manager — justfile
# Run with `just <recipe>`. See `just --list` for all recipes.

default: fmt lint check test

fmt:
	bun run format
	cargo fmt --manifest-path src-tauri/Cargo.toml --all

lint:
	bun run lint
	cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

check:
	bun run check
	cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
	cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check

test:
	bun run test
	cargo test --manifest-path src-tauri/Cargo.toml

bump level:
	python scripts/bump-version.py --{{level}}
