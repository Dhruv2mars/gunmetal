# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,260p' docs/plans.md`
- `sed -n '1,220p' docs/implement.md`
- `sed -n '1,320p' docs/documentation.md`
- targeted daemon/web/tui/cli tests as milestones land
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- `git status --short --branch`
- Last verified: 2026-04-13

## Milestones
- `[done]` Milestone 1: refresh durable memory for the post-Phase-1 traffic-inspection slice
  - Scope: replace the completed Phase 1 execution plan with the next additive slice
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs reflect the new product taxonomy and the next implementation target
  - Verification steps: direct review of the updated docs
- `[done]` Milestone 2: audit current traffic/request surfaces and define the additive data model
  - Scope: inspect current operator state, browser request history, OpenTUI request view, and CLI logs
  - Key areas: `packages/app-daemon`, `packages/app-cli`, `packages/app-tui`
  - Acceptance criteria: exact next slice is pinned down before editing code
  - Verification steps: targeted code reads and notes in docs
- `[done]` Milestone 3: implement daemon-backed provider/model summaries
  - Scope: add summary data that explains requests and token usage by provider and by model
  - Key areas: `packages/app-daemon/src/lib.rs`
  - Acceptance criteria: operator state exposes coherent summary rows without changing the core request path
  - Verification steps: focused daemon tests
- `[done]` Milestone 4: expose the new traffic intelligence in Web UI and OpenTUI
  - Scope: surface summaries and lightweight request navigation improvements in the existing UIs
  - Key areas: `packages/app-daemon/src/browser_app.html`, `packages/app-tui/src/opentui.js`
  - Acceptance criteria: request history becomes navigable and the summary layer is visible without clutter
  - Verification steps: targeted daemon/TUI smoke checks
- `[done]` Milestone 5: improve CLI traffic inspection
  - Scope: give the Rust CLI better traffic inspection via summaries and/or filters
  - Key areas: `packages/app-cli/src/lib.rs`
  - Acceptance criteria: CLI can answer the same basic traffic questions as the UI surfaces
  - Verification steps: targeted CLI tests and help smokes
- `[in_progress]` Milestone 6: verification, cleanup, and push checkpoint(s)
  - Scope: keep diffs clean, rerun verification, and push when the slice is coherent
  - Key areas: touched packages, docs, git state
  - Acceptance criteria: repo is clean, tests are green, and `main` only moves when the slice is coherent
  - Verification steps: `bun run test`, `bun run check`, `cargo test --workspace`, `git status`

## Acceptance Checks
- Durable memory states the new `Products` vs `Developer` mental model.
- Gunmetal super app remains the current product focus.
- Developer-facing SDK framing stays internal-first while being captured as future public developer product surface.
- Operator state exposes useful provider/model traffic summaries.
- Request history is easier to inspect than a flat log list.
- Web UI, CLI, and OpenTUI stay aligned around the same traffic concepts.

## Validation
- `git status --short --branch`
- targeted `sed -n` reads across docs and touched files
- focused crate/package tests for the milestone being worked
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- push to `origin/main` when the current slice is coherent and verified

## Decisions
- Product thesis remains: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- Product taxonomy is now:
  - `Products`: consumer-facing Gunmetal apps and services, with the super app as the active product
  - `Developer`: the internal engines that later become public developer products, starting with the provider/extension SDK
- The app is still the product. The SDK is still internal first and powers the app.
- Post-Phase-1 work should deepen Gunmetal's built-in toll-booth value instead of adding another surface or broadening scope.
- The next additive slice is request intelligence: summary and navigation, not new inference execution modes.

## Implementation Notes
- Active work should land through a feature branch and only move `main` once the slice is coherent and verified.
- Phase 1 is complete and should be treated as stable base functionality.
- Current operator state already exposes:
  - providers
  - models
  - keys
  - logs
  - setup state
  - top-line traffic totals
- Current gap:
  - request history is still mostly a flat recent log list
  - there is no first-class provider/model summary layer
  - CLI logs are too raw for quick inspection
- Implemented in this slice:
  - daemon operator state now exposes provider-level traffic summaries
  - daemon operator state now exposes model-level traffic summaries
  - browser request history now includes provider/status filters plus top-provider/top-model summary cards
  - OpenTUI request view now includes provider/status filters and summary text for top providers and models
  - CLI `logs` now supports provider/model/status filtering
  - CLI now has `gunmetal logs summary`

## Risks
- Risk: the new inspection layer turns into generic dashboard clutter.
  - Mitigation: keep summaries compact and directly tied to requests, latency, and tokens.
- Risk: Web UI and OpenTUI drift.
  - Mitigation: keep both surfaces backed by the same daemon state shape.
- Risk: CLI becomes a second-class surface again.
  - Mitigation: improve `logs` instead of leaving traffic inspection UI-only.

## Architecture
- Product: local-first inference middle layer for individuals.
- Canonical flow: `app/tool -> Gunmetal key -> Gunmetal -> provider extension -> upstream provider`.
- Product taxonomy:
  - `Products`: super app now, standalone user products later
  - `Developer`: internal engines now, standalone SDKs later
