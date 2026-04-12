# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- This pass continues the first-principles refactor by improving actual usability on top of the completed copy pass. The repo layout and internal SDK boundary already exist. The current slice is intentionally narrow: improve the local browser UI workflow itself so provider setup adapts to the selected provider and request history supports a real drill-down.

## Current Status
- Durable memory exists and is updated for the browser-UI usability pass.
- Product thesis: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- User-facing term is `provider`; internal implementation term is `extension`.
- Current branch for this refactor: `refactor/internal-sdk`.
- The internal SDK extraction is already implemented and verified.
- Repo-structure refactor is implemented and verified.
- The first UX cleanup pass is implemented and verified.
- The deeper usability pass is now focused on the local browser UI workflow.
- The setup/traffic browser-UI/state implementation is already in place and verified.
- The workflow-polish implementation is complete and verified.

## Status By Milestone
- Milestone 1 complete: durable-memory refresh for the workflow-polish pass.
- Milestone 2 complete: inspected the local browser UI workflow.
- Milestone 3 complete: implemented adaptive provider setup and request drill-down.
- Milestone 4 complete: reran verification and confirmed this checkpoint is ready to move to `main`.

## Setup And Verification
- Repo state:
  - `git status --short --branch`
  - `find apps packages -maxdepth 2 -type f | sort`
  - targeted `sed -n` reads of docs, manifests, and key source files
- Validation commands for this pass:
  - `cargo test -p gunmetal-daemon`
  - `bun run test`
  - `cargo test --workspace`

## Completed Work
- Established the first-principles product decisions:
  - Gunmetal is one product, not three.
  - Gunmetal App is the local-first product for individuals.
  - An internal Gunmetal SDK powers the app’s multi-provider support.
  - First-party providers are implemented as extensions on top of that SDK.
  - The first built-in toll-booth benefits are request history and token stats.
- Implemented the internal SDK boundary already in the repo:
  - added `packages/sdk`
  - moved provider abstraction/runtime into `packages/sdk`
  - kept first-party provider clients and extension implementations in `packages/extensions`
  - rewired CLI, daemon, and TUI to use the SDK-backed provider system
- Restructured the repo layout to match the architecture:
  - `apps/gunmetal` now holds the native app entrypoint
  - `packages/extensions` holds first-party provider extensions
  - `packages/sdk-core` holds shared SDK-facing types
  - app-only Rust support crates now live under `packages/app-*`
- Began the UX pass with an audit of the main drift points:
  - older `switchboard` and `control plane` framing still appears in docs and web copy
  - `profile` still dominates several user-facing surfaces even though the chosen UX noun is `provider`
  - the browser UI, CLI, and TUI all need a first cleanup pass before deeper workflow refinement
- Completed the first UX cleanup pass:
  - updated `README.md` and the main web/docs copy to present Gunmetal as a local inference middle layer
  - shifted the browser UI labels and banners toward `provider`, `request history`, and local operation
  - updated CLI help/setup/output text to match the product thesis while leaving command names stable
  - updated TUI tabs, prompts, detail panes, and status messages to use provider-first language
  - updated daemon/API operator-facing error and action messages to remove older profile-centric phrasing where surfaced
- Current usability focus:
  - the browser UI should feel like the cleanest operator surface
  - setup progress should be explicit rather than inferred from raw counts
  - request history should carry more useful token/traffic context
- Browser-UI usability implementation:
  - added compact `setup` and `traffic` summary sections to `/app/api/state`
  - added per-log input/output token fields to the operator-state response
  - updated the browser UI to render a four-step setup checklist under the golden path
  - replaced the old service-only side panel with a traffic snapshot plus service details
  - updated request-history rows to show token breakdowns and localized timestamps
- Current workflow-polish implementation:
  - added a provider-form helper region to the browser shell
  - the provider form now adapts for browser-login vs API-key providers
  - added a selected-request detail region above request history
  - request rows are now selectable and render focused latency/token/endpoint detail

## Validation Results
- The SDK extraction pass was previously verified:
  - `cargo test -p gunmetal-sdk` passed
  - `cargo test -p gunmetal-providers` passed
  - `cargo test -p gunmetal-cli -p gunmetal-daemon -p gunmetal-tui` passed
  - `cargo test --workspace` passed
- Repo-structure verification also passed:
  - `node --test test/repo-structure.test.js` passed
  - `bun run test` passed
  - legacy path sweep across `README.md`, `AGENTS.md`, `Cargo.toml`, `test`, `apps`, and `packages` returned no stale old-layout matches
- UX audit completed by directly reviewing the touched surfaces:
  - `README.md`
  - `apps/web/src/app/page.tsx`
  - `apps/web/src/app/layout.tsx`
  - `apps/web/src/app/start-here/page.tsx`
  - `apps/web/src/app/web-ui/page.tsx`
  - `apps/web/src/components/site-shell.tsx`
  - `apps/web/src/lib/site-content.ts`
  - `packages/app-cli/src/lib.rs`
  - `packages/app-daemon/src/browser_app.html`
  - `packages/app-tui/src/lib.rs`
- UX verification completed:
  - `cargo run -p gunmetal -- --help` passed and showed the updated thesis text
  - `cargo test -p gunmetal-cli -p gunmetal-daemon -p gunmetal-tui` passed
  - `bun run test` passed
  - `cargo test --workspace` passed
- Usability inspection in progress:
  - reviewing `packages/app-daemon/src/browser_app.html`
  - reviewing `packages/app-daemon/src/lib.rs` state-response shaping
  - identifying the smallest additive payload fields needed for a better UI
- Focused usability verification completed:
  - `cargo test -p gunmetal-daemon` passed
- Repo-wide verification completed:
  - `bun run test` passed
  - `cargo test --workspace` passed
- Focused workflow-polish verification completed:
  - `cargo test -p gunmetal-daemon` passed
- Repo-wide workflow-polish verification completed:
  - `bun run test` passed
  - `cargo test --workspace` passed

## Decisions
- Product promise: use AI subscriptions and provider accounts through one local middle layer for inference.
- Gunmetal’s current OpenAI-compatible surface is the initial compatibility layer, not the permanent only abstraction.
- The SDK stays internal until the app architecture is tightly packed and stable.
- UX should stay extremely simple for normal users: connect providers, mint a key, make requests.
- The first UX cleanup pass should emphasize:
  - one local API
  - connected providers
  - request history
  - token stats
- Repo layout should communicate architecture directly:
  - app entrypoint under `apps/gunmetal`
  - SDK under `packages/sdk`
  - first-party extensions under `packages/extensions`
  - app-only Rust support packages under `packages/app-*`

## Next Steps
- Move the verified workflow-polish slice through to `main`.
- Use this cleaner browser UI as the base for deeper flow work later.
- Revisit SDK naming and public packaging only when the internal SDK surface is stable enough to publish confidently.

## Follow-Ups
- Later work should decide how much of the SDK becomes public API and how much remains internal.
- After this copy-and-surface pass, the next substantive phase should improve the actual usability and flow quality of the CLI, TUI, and web/browser UX instead of just the wording.
