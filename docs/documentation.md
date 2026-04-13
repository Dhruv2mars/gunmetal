# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- This is the GA shipping pass for the Gunmetal super app.
- The focus is now public readiness of the product surfaces, not broadening scope.

## Current Status
- Durable memory exists and is updated for the GA shipping pass.
- Product thesis remains: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- Product taxonomy remains explicit:
  - `Products`: consumer-facing Gunmetal products, with the super app as the current product
  - `Developer`: internal engines that later become public developer products, starting with the provider/extension SDK
- The current public-shipping order is fixed:
  - landing page
  - CLI
  - TUI
  - Web UI
- Hosted truth is now explicit:
  - canonical public host is `gunmetalapp.vercel.app`
  - public Web UI marketing route is `/webui`
  - `gunmetal.vercel.app` belongs to an unrelated old site and must not be used
- The current active milestone is complete for this shipping pass.

## Status By Milestone
- Milestone 1 complete: durable-memory refresh and shipping branch.
- Milestone 2 complete: public landing page polish.
- Milestone 3 complete: CLI shipping polish.
- Milestone 4 complete: TUI shipping polish.
- Milestone 5 complete: Web UI shipping polish.
- Milestone 6 complete: final GA verification and release cleanup.

## Current Implementation Focus
- The foundation from earlier phases remains in place:
  - shared provider metadata
  - aligned daemon, CLI, and OpenTUI provider behavior
  - stronger request inspection
- Shipping work completed so far in this pass:
  - landing page rewritten around the real super-app story
  - canonical public host corrected to `gunmetalapp.vercel.app`
  - public Web UI route moved to `/webui` with a redirect from `/web-ui`
  - CLI help and recovery paths tightened for first-time public users
  - OpenTUI tightened for public use, including clearer setup-to-playground flow and narrower 80-column terminal copy
  - browser Web UI tightened around the real golden path:
    - Gunmetal-key-only playground copy
    - provider-scoped playground model selection
    - calmer request-history range controls
    - narrower models-in-view presentation
    - streamed playground requests now persist into request history correctly
- The current task is the final GA verification pass.

## Why This Matters
- The public front door needs to explain the product cleanly before broader launch.
- The later CLI, TUI, and Web UI passes should inherit one clear product story rather than drifting.
- Public readiness depends on both presentation quality and real usability.

## Validation Results
- Last completed full-repo verification before this shipping pass:
  - `bun run test`
  - `bun run check`
  - `cargo test --workspace`
- Landing page verification completed:
  - `bun run --filter @gunmetal/web test`
  - `bun run --filter @gunmetal/web lint`
  - live browser checks on desktop and mobile with agent-browser
  - route verification for `/install`, `/start-here`, `/docs`, `/webui`, and redirect from `/web-ui`
- CLI verification completed:
  - `cargo test -p gunmetal-cli`
  - command help smokes for root, setup, web, start, status, logs list, providers list, and chat
  - live command smokes for `gunmetal status`, `gunmetal profiles list`, and `gunmetal providers list`
- TUI verification completed:
  - `cargo test -p gunmetal-tui`
  - live `gunmetal tui` launcher smokes, including a constrained `80x24` terminal check
- Web UI verification completed:
  - `cargo test -p gunmetal-daemon`
  - desktop and mobile screenshot captures against `/app`
  - live browser interaction against `/app` using a Gunmetal key
  - live browser verification of the current Codex upstream failure path and request-detail update
- Final GA verification completed:
  - `bun run test`
  - `bun run check`
  - `cargo run -p gunmetal -- status`
  - `cargo run -p gunmetal -- logs list`

## Decisions
- The Gunmetal super app is the only product in scope for this pass.
- Surface order is fixed and should not be skipped.
- Public polish should improve the real golden path, not just copy or visuals.

## Next Steps
- Clean the repo, commit, and move the checkpoint to `main`.
