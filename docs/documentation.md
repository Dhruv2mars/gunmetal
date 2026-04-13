# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- This phase continues the first-principles refactor by turning the new architecture into a complete single-user product flow. The repo layout and internal SDK boundary already exist. The active phase is broader than the last browser-only pass: finish the golden path across Web UI, CLI, and TUI, rewrite the TUI in OpenTUI, and add a minimal playground.

## Current Status
- Durable memory exists and is updated for Phase 1.
- Product thesis: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- User-facing term is `provider`; internal implementation term is `extension`.
- Active work lands through a feature branch and only moves `main` once the slice is coherent and verified.
- The internal SDK extraction is already implemented and verified.
- Repo-structure refactor is implemented and verified.
- The first UX cleanup pass is implemented and verified.
- The browser-UI workflow polish pass is implemented and verified.
- The active phase is now full Phase 1 execution rather than another browser-only slice.

## Status By Milestone
- Milestone 1 complete: durable-memory refresh for Phase 1.
- Milestone 2 complete: shared flow contract and capability audit.
- Milestone 3 complete: daemon/operator API alignment.
- Milestone 4 complete: Web UI golden path and playground.
- Milestone 5 complete: OpenTUI rewrite and parity.
- Milestone 6 complete: CLI parity and playground.
- Milestone 7 complete: cleanup, verification, and push checkpoints.

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
- Phase 1 decisions now locked:
  - Web UI, CLI, and TUI are all first-class surfaces
  - Web UI is the primary guided surface
  - CLI stays in Rust
  - TUI will be rewritten in OpenTUI
  - OpenTUI should consume daemon/operator endpoints and not own business logic
  - Playground is part of Phase 1
  - Playground uses only Gunmetal keys
  - Playground uses synced models for model selection
  - Playground supports both `chat/completions` and `responses`
  - First-request verification happens through the playground
- Capability audit completed:
  - `/app/api/state` already gives the operator surfaces enough provider/model/key/log context to drive the golden path
  - `/v1/chat/completions` and `/v1/responses` already cover the execution path the playground should use
  - request logging and token accounting already happen on those inference routes, so playground traffic automatically feeds request history
  - the main remaining work is surface implementation, not a new inference backend abstraction
- Phase 1 implementation now landed:
  - browser UI includes a real Gunmetal-key playground on top of the existing provider/key/request surfaces
  - CLI includes `gunmetal chat` for interactive or one-shot playground use
  - Rust TUI has been replaced with a Bun/OpenTUI surface launched through a thin Rust wrapper
  - OpenTUI now exposes setup, playground, and request-inspection views against daemon-backed APIs

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
- Phase 1 planning verification completed:
  - `sed -n '1,220p' docs/prompt.md`
  - `sed -n '1,260p' docs/plans.md`
  - `sed -n '1,220p' docs/implement.md`
  - `sed -n '1,260p' docs/documentation.md`
  - `git status --short --branch`
- Phase 1 implementation verification completed:
  - `cargo test -p gunmetal-cli -p gunmetal-daemon -p gunmetal-tui`
  - `bun run test`
  - `bun run check`
  - `cargo run -p gunmetal -- --help`
  - `cargo run -p gunmetal -- chat --help`
  - Bun/OpenTUI smoke run via `perl -e 'alarm 3; exec @ARGV' bun run packages/app-tui/src/opentui.js`

## Decisions
- Product promise: use AI subscriptions and provider accounts through one local middle layer for inference.
- Gunmetal’s current OpenAI-compatible surface is the initial compatibility layer, not the permanent only abstraction.
- The SDK stays internal until the app architecture is tightly packed and stable.
- UX should stay extremely simple for normal users: connect providers, mint a key, make requests.
- The first built-in toll-booth value stays request history plus token stats.
- Web UI, CLI, and TUI are all first-class in Phase 1.
- Web UI is the canonical guided flow.
- CLI remains the exact operational and scripting surface.
- TUI should feel like the Web UI in terminal form.
- The Phase 1 playground exists to test/demo Gunmetal keys and models directly, not to become a full chat product.
- Repo layout should communicate architecture directly:
  - app entrypoint under `apps/gunmetal`
  - SDK under `packages/sdk`
  - first-party extensions under `packages/extensions`
  - app-only Rust support packages under `packages/app-*`

## Next Steps
- Finish repo cleanup for the Phase 1 branch, then commit and push the verified checkpoint.
- After Phase 1, move into deeper traffic inspection and control features rather than more surface churn.
- Revisit SDK naming and public packaging only when the internal SDK surface is stable enough to publish confidently.

## Follow-Ups
- Later work should decide how much of the SDK becomes public API and how much remains internal.
- After Phase 1, the next substantive phase should deepen the toll-booth value itself: richer request inspection, better usage summaries, and tighter control surfaces without introducing team/platform complexity.
