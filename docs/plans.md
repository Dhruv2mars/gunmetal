# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,320p' docs/plans.md`
- `sed -n '1,220p' docs/implement.md`
- `sed -n '1,320p' docs/documentation.md`
- targeted daemon/web/tui/cli/sdk/provider tests as milestones land
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- `git status --short --branch`
- Last verified: 2026-04-13

## Milestones
- `[done]` Milestone 1: refresh durable memory and branch for Phase 2
  - Scope: replace the older traffic-summary slice with the three-track Phase 2 definition
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs reflect the Phase 2 tracks and product taxonomy
  - Verification steps: direct review of the updated docs
- `[done]` Milestone 2: audit the usability gaps and internal boundaries
  - Scope: inspect superapp provider setup, request inspection, daemon state shaping, and SDK/provider definitions
  - Key areas: `packages/app-daemon`, `packages/app-tui`, `packages/app-cli`, `packages/sdk`, `packages/extensions`
  - Acceptance criteria: the concrete Phase 2 work is pinned down before broad edits
  - Verification steps: targeted code reads and notes in docs
- `[done]` Milestone 3: harden the internal extension SDK
  - Scope: add explicit provider capabilities and UX hints to provider definitions and expose them through the internal hub/registry
  - Key areas: `packages/sdk`, `packages/extensions`
  - Acceptance criteria: provider behavior is explicit in the internal developer layer
  - Verification steps: `cargo test -p gunmetal-sdk`, `cargo test -p gunmetal-providers`
- `[done]` Milestone 4: improve real-world superapp usability
  - Scope: drive Web UI and OpenTUI setup from real provider metadata, remove unsupported provider affordances, and improve request drill-down/filtering
  - Key areas: `packages/app-daemon/src/browser_app.html`, `packages/app-tui/src/opentui.js`
  - Acceptance criteria: the super app guides real provider usage more clearly and request inspection is more actionable
  - Verification steps: focused daemon tests and TUI smoke checks
- `[done]` Milestone 5: align CLI and daemon architecture to the same provider and request concepts
  - Scope: remove stale hardcoded provider branching in CLI, improve provider listing/log inspection, and keep daemon state shaping aligned with the new metadata
  - Key areas: `packages/app-cli/src/lib.rs`, `packages/app-daemon/src/lib.rs`
  - Acceptance criteria: CLI is not second-class and the daemon uses the same provider metadata model as the rest of the app
  - Verification steps: `cargo test -p gunmetal-daemon -p gunmetal-cli -p gunmetal-tui`
- `[done]` Milestone 6: repo-wide verification, cleanup, and push
  - Scope: run full gates, smoke the important flows, keep the diff clean, and move the checkpoint to `main`
  - Key areas: touched packages, docs, git state
  - Acceptance criteria: repo is clean, tests are green, and the slice is coherent enough for `main`
  - Verification steps: `bun run test`, `bun run check`, `cargo test --workspace`, smokes, `git status`

## Acceptance Checks
- The super app no longer offers unsupported providers in the browser setup flow.
- Web UI and OpenTUI provider setup is driven by shared provider metadata.
- The internal SDK exposes explicit provider capabilities and UX hints.
- CLI provider and log inspection use the same provider model, not stale hardcoded rules.
- Request inspection across surfaces includes better filtering and richer context.
- The repo stays aligned with the `Products` vs `Developer` mental model without changing current product scope.

## Validation
- `git status --short --branch`
- targeted `sed -n` reads across docs and touched files
- focused crate/package tests for the milestone being worked
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- smoke `gunmetal logs summary`, `gunmetal tui`, and the local browser shell via daemon tests
- push to `origin/main` when the slice is coherent and verified

## Decisions
- Phase 2 is one integrated pass across usability, architecture, and SDK hardening rather than three disconnected tasks.
- The super app remains the product center of gravity.
- The internal SDK remains private, but its contracts should now look more like future public developer product surfaces.
- Shared provider metadata is the key leverage point:
  - it improves user setup flows now
  - it cleans daemon/CLI branching now
  - it hardens future SDK boundaries now

## Implementation Notes
- Active work should land through a feature branch and only move `main` once the slice is coherent and verified.
- Implemented in Phase 2:
  - `ProviderDefinition` now carries explicit capability and UX metadata
  - `ProviderHub` and `ProviderRegistry` now expose provider definitions
  - first-party extensions now publish real provider metadata through those definitions
  - browser provider selection is now driven by live provider definitions instead of hardcoded options
  - OpenTUI provider guidance is now driven by the same provider definitions
  - request inspection now includes richer filtering and more request context
  - CLI provider and log inspection now align better with the same provider model

## Risks
- Risk: Phase 2 adds more chrome without improving actual user outcomes.
  - Mitigation: keep the changes tied to provider setup correctness and request inspection clarity.
- Risk: SDK metadata becomes hand-wavy and not actually useful.
  - Mitigation: use the new metadata directly in daemon, browser UI, OpenTUI, and CLI behavior.
- Risk: daemon and UI layers drift again later.
  - Mitigation: keep the daemon/operator state as the shared source of truth and keep UI behavior driven by that shape.

## Architecture
- Product: local-first inference middle layer for individuals.
- Canonical flow: `app/tool -> Gunmetal key -> Gunmetal -> provider extension -> upstream provider`.
- Product taxonomy:
  - `Products`: super app now, standalone user products later
  - `Developer`: internal engines now, standalone SDKs later
