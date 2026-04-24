# Documentation

Updated continuously during the UX pass.

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
  - `gunmetal web`
  - `gunmetal doctor`
  - `gunmetal chat`
  - `gunmetal logs summary`

## Current Status
- TUI is out of scope and should not be referenced as an active public surface.
- Hosted truth remains `gunmetalapp.vercel.app`.
- Public Web UI marketing route remains `/webui`.
- Product route `/products/suite` explains providers, models, keys, requests, and local API flow.
- Developer route `/developer/sdk` explains extension surfaces and points to SDK/extension packages.
- Download route `/download` gives install command, setup, Web UI, start, status, and GitHub releases.
- Docs route `/docs` is a compact quick-start with sticky step navigation and API contract.
- Changelog routes `/changelogs` and `/changelog` load GitHub Releases with a local fallback state.
- Local browser UI remains `http://127.0.0.1:4684/app`.
- Local API remains `http://127.0.0.1:4684/v1`.
- Landing page and shared landing navbar were restored to pre-2026-04-23 state after scope correction.
- `DESIGN.md` now defines landing and subpage brand/design rules.
- Local browser Web UI now has a clearer next-action rail, refined mobile behavior, and safer narrow-screen containment.
- CLI now includes `gunmetal doctor` for setup diagnosis and next-command guidance.

## Validation Results
- `npx --yes bun@1.3.5 run --filter @gunmetal/web test`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web lint`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web build`
- `cargo fmt --all --check`
- `cargo test -p gunmetal-cli`
- `cargo test -p gunmetal-daemon`
- `cargo run -p gunmetal -- --help`
- `cargo run -p gunmetal -- doctor`
- `cargo run -p gunmetal -- web --no-open`
- agent-browser live check for hosted `/` and `/webui`
- agent-browser live check for local `/app` desktop and mobile
- `npx --yes bun@1.3.5 run --filter @gunmetal/web test`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web lint`
- `npx --yes bun@1.3.5 run --filter @gunmetal/web build`
