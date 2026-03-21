# Repository Guidelines

## Project Structure & Module Organization
`apps/cli/` is the native Gunmetal app entrypoint; `apps/web/` holds the Next.js site and docs. Internal Rust modules live in `packages/`: `packages/cli` handles commands, `packages/daemon` serves HTTP, `packages/providers` integrates upstreams, `packages/storage` owns SQLite, `packages/core` shares types, and `packages/tui` renders the terminal UI. `packages/npm/` is the npm distribution wrapper; executable code is in `packages/npm/bin`, tests in `packages/npm/test`.

## Build, Test, and Development Commands
Use Bun at the repo root.

- `bun install` installs JS workspace deps.
- `bun run dev` starts the web app locally.
- `bun run build` builds web, npm wrapper, and Rust workspace.
- `bun run test` runs the repo-structure guard, web type checks, npm wrapper tests, and `cargo test --workspace`.
- `bun run check` runs web lint, CLI checks, `cargo fmt --check`, and strict Clippy.
- `cargo run -p gunmetal -- --help` smoke-tests the Rust entrypoint.

## Coding Style & Naming Conventions
Rust follows `rustfmt` defaults and `snake_case` module names. Keep types `PascalCase` and prefer small focused modules. TS/JS/TSX uses 2-space indentation, semicolons, and `PascalCase` React components; route files stay in Next.js app-router form such as `src/app/install/page.tsx`. Run `bun run fmt:web` for web fixes and `cargo fmt --all` for Rust. If working in `apps/web`, read `apps/web/AGENTS.md` first; it has extra Next.js-specific rules.

## Testing Guidelines
Follow test-first work. Start with a failing test, then implement. npm wrapper changes should begin with `packages/npm/test/*.test.js`; Rust changes should add or extend nearby `#[cfg(test)]` modules. Run `bun run test` before opening a PR. Target full coverage for touched code; do not merge untested paths.

## Commit & Pull Request Guidelines
Create a branch for every change. Keep commits atomic and use prefixes seen in history: `feat:`, `fix:`, `test:`. Use GitHub CLI for repo flow: `gh pr create`, `gh pr merge`. PRs should say what changed, why, how it was verified, and include screenshots for web UI changes. Merge to `main`, then delete the branch.

## Security & Configuration Tips
Do not commit secrets. `.env*`, `node_modules/`, `coverage/`, and `target/` are ignored already; keep local runtime state there or under temp files, not in tracked source.
