# Documentation

Last updated: 2026-05-05.

## What This Work Is
- First-principles UX/frontend pass for the refactored Gunmetal product.
- Active surfaces are:
  - hosted web site in `apps/web`
  - local browser Web UI served from `packages/app-daemon`
  - CLI in `packages/app-cli`

## Product Thesis
- Gunmetal is a local-first inference middle layer for individuals.
- It turns provider accounts and AI subscriptions into one OpenAI-compatible local API.
- Canonical flow:
  - `app/tool -> Gunmetal key -> Gunmetal local API -> provider extension -> upstream provider`

## Current UX Model
- First screen should answer:
  - what Gunmetal is
  - how to install it
  - where the local API lives
  - what command gets the browser UI open
  - what to do when setup is incomplete
- Local browser Web UI should prioritize:
  - setup readiness
  - provider auth/sync
  - key creation
  - playground test
  - request history and recovery details
- CLI should prioritize:
  - `gunmetal setup`
  - `gunmetal start`
  - `gunmetal doctor`
  - `gunmetal chat`
  - `gunmetal logs summary`

## Current Status
- TUI is out of scope and should not be referenced as an active public surface.
- Hosted truth remains `gunmetalapp.vercel.app`.
- The local Dashboard is served by the daemon at `/`; `/webui` is not a public user-facing route.
- Product route `/products/suite` explains providers, models, keys, requests, and local API flow.
- Developer route `/developer/sdk` explains extension surfaces and points to SDK/extension packages.
- Provider SDK crates are prepared as public crates: `gunmetal-core`, `gunmetal-storage`, `gunmetal-sdk`, and `gunmetal-providers`.
- The release workflow publishes SDK crates in dependency order before publishing the npm CLI package.
- Download route `/download` gives install command, setup, Web UI, start, status, and GitHub releases.
- Docs route `/docs` is a compact quick-start with sticky step navigation and API contract.
- Changelog routes `/changelogs` and `/changelog` load GitHub Releases with a local fallback state.
- Local browser UI is `http://127.0.0.1:4684/`.
- Local API remains `http://127.0.0.1:4684/v1`.
- Landing page and shared landing navbar were restored to pre-2026-04-23 state after scope correction.
- `DESIGN.md` now defines landing and subpage brand/design rules.
- Local browser Web UI now uses the `/` route, a calm Gunmetal-branded operator shell, a clearer setup/action flow, clipboard fallback handling, and safer narrow-screen containment.
- CLI now includes `gunmetal doctor` for setup diagnosis and next-command guidance.
- Zen free-model E2E is covered through the local API path; the Zen adapter handles OpenRouter-style SSE payloads when a free upstream model streams despite a chat-completion request.

## Validation Results
- `npx --yes bun@1.3.5 run --filter @gunmetal/web test`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web lint`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web build`
- `bun run test`
- `bun run check`
- `bun run build`
- `cargo publish -p gunmetal-core --dry-run --allow-dirty`
- real Zen E2E: isolated `GUNMETAL_HOME`, profile create, model sync, key create, `gunmetal start --no-open`, `/v1/models`, `gunmetal chat` with `zen/hy3-preview-free`, and logs summary
- Browser Use check for `http://localhost:3000/`, `/products/suite`, `/developer/sdk`, `/download`, `/docs`
- Browser Use check for local `http://127.0.0.1:4684/`
- `cargo fmt --all --check`
- `cargo test -p gunmetal-cli`
- `cargo test -p gunmetal-daemon`
- `cargo run -p gunmetal -- --help`
- `cargo run -p gunmetal -- doctor`
- `cargo run -p gunmetal -- start --no-open`
- agent-browser live check for hosted `/`
- agent-browser live check for local `/` desktop and mobile
- `npx --yes bun@1.3.5 run --filter @gunmetal/web test`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web lint`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web build`
