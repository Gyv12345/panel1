# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace (`Cargo.toml`) with code split by responsibility under `crates/`:
- `panel-cli`: `panel1` binary entrypoint and CLI commands
- `panel-core`: system/process/service primitives
- `panel-service`: managed service lifecycle and registry logic
- `panel-ai`: AI providers, agents, and tooling
- `panel-tui`: terminal UI, theme, and panels

Build/release scripts live at the repo root (`build-release.sh`, `build-linux.sh`, `build-linux-docker.sh`). Reference docs live in `docs/`.

## Build, Test, and Development Commands
- `cargo run -p panel1 -- --help`: run the CLI locally.
- `cargo run -p panel1 -- tui`: start the TUI during development.
- `cargo check --all-targets`: fast compile checks across workspace targets.
- `cargo test --all`: run all unit/integration tests.
- `cargo fmt --all`: format all crates.
- `cargo clippy --all-targets -- -W clippy::all`: lint with CI-equivalent strictness.
- `bash -n install.sh`: validate one-line installer script syntax.
- `./build-release.sh`: build and package the current platform artifact into `dist/`.

## Coding Style & Naming Conventions
Use Rust 2021 idioms and keep formatting rustfmt-clean (4-space indentation, no manual alignment hacks).
- Files/modules/functions: `snake_case`
- Types/traits/enums: `PascalCase`
- Constants/statics: `SCREAMING_SNAKE_CASE`

Keep modules focused by crate boundary (for example, avoid TUI-specific logic in `panel-core`). Prefer explicit error propagation with `Result` and context-rich errors.

## Testing Guidelines
Place tests close to code with `#[cfg(test)] mod tests`, and use `#[tokio::test]` for async paths. Name tests by behavior, e.g. `parse_checksum_rejects_invalid_hex`.

Before opening a PR, run:
1. `cargo fmt --all`
2. `cargo clippy --all-targets -- -W clippy::all`
3. `cargo test --all`

CI enforces check/test/fmt/clippy, so local results should match CI.

## Commit & Pull Request Guidelines
Follow Conventional Commits, as seen in history: `feat:`, `fix:`, `docs:`, `style:`, `chore:`, `ci:`.

PRs should include:
- clear purpose and scope
- affected crates/modules
- verification steps and command results
- screenshots or terminal captures for TUI/UI changes
- linked issue(s) when applicable
