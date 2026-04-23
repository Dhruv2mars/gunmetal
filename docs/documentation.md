# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- This is the TUI removal and repo cleanup pass.
- Gunmetal now concentrates on two user-facing control surfaces:
  - CLI for setup, service control, auth, keys, models, chat, logs, and scripted use
  - Web UI for graphical setup and operator workflows

## Current Status
- Durable memory has been retargeted from the older GA/TUI pass to TUI removal.
- Product thesis remains: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- The TUI is no longer a product surface.
- The native app entrypoint behaves like a CLI, not a terminal UI launcher.
- Canonical public host remains `gunmetalapp.vercel.app`.
- Public Web UI marketing route remains `/webui`.
- `gunmetal.vercel.app` belongs to an unrelated old site and must not be used.

## Current Implementation Focus
- `packages/app-tui` has been removed.
- `gunmetal tui` has been removed.
- `gunmetal-tui`, OpenTUI, and TUI lockfile dependencies have been removed.
- Stale TUI copy has been removed from README, site content, npm metadata, repo tests, and repo guidance.
- Daemon, storage, CLI, SDK, providers, web, and npm wrapper remain intact.
- Web lint cleanup removed an unused package-manager array and converted logo `<img>` tags to `next/image`.
- Post-TUI cleanup removed unused web scaffold:
  - placeholder product/developer/download/changelog pages
  - dropdown navbar complexity
  - unused section/UI components
  - unused demo SVG assets
  - unused `framer-motion`, `clsx`, and `tailwind-merge` deps
  - unused Rust workspace deps from older app scaffolding
  - no-longer-public CLI daemon helper types/functions

## Architecture
- Product: local-first inference middle layer for individuals.
- Canonical flow: `app/tool -> Gunmetal key -> Gunmetal -> provider extension -> upstream provider`.
- Control surfaces:
  - CLI: `gunmetal setup`, `gunmetal web`, `gunmetal start`, `gunmetal status`, `gunmetal chat`, `gunmetal logs ...`
- Web UI: local browser app at `http://127.0.0.1:4684/app`
- Public web routes:
  - `/`
  - `/webui`
  - `/start-here`
  - `/docs`
  - `/install`
- Internal layers:
  - daemon: local OpenAI-compatible API and Web UI shell
  - storage: SQLite/local state
  - SDK/core/extensions: provider contracts and built-in providers
  - npm package: native binary install wrapper

## Validation Results
- `rg -n "packages/app-tui|@gunmetal/tui|gunmetal-tui|GUNMETAL_TUI|OpenTUI|@opentui|gunmetal tui|terminal UI|terminal experience|\\bTUI\\b|\\btui\\b" . -S --glob '!docs/**'`
  - clean except one negative CLI help assertion.
- `cargo run -p gunmetal -- --help`
  - no TUI command.
- `cargo run -p gunmetal`
  - prints CLI help instead of launching a TUI.
- `cargo run -p gunmetal -- status`
  - stopped recovery points to `gunmetal start` or `gunmetal web`.
- `npm exec --yes bun@1.3.5 -- run test`
  - pass.
- `npm exec --yes bun@1.3.5 -- run check`
  - pass.
- `npm exec --yes bun@1.3.5 -- run --filter @gunmetal/web build`
  - pass; static routes are `/`, `/webui`, `/start-here`, `/docs`, and `/install`.
- `cargo test -p gunmetal-cli`
  - pass.
- Dead-code scans for removed web scaffold, placeholder routes, direct JS deps, app TUI crates, and OpenTUI packages:
  - clean, except `tower-http` remains as a transitive lockfile dependency.

## Decisions
- TUI is removed instead of kept as an alternate setup path.
- No-command `gunmetal` should show CLI help.
- `gunmetal web` is the graphical surface.
- `gunmetal start` is the API-only service path.
