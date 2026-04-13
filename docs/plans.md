# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,260p' docs/plans.md`
- `sed -n '1,220p' docs/implement.md`
- `sed -n '1,260p' docs/documentation.md`
- targeted daemon/web/tui/cli tests as milestones land
- `bun run test`
- `cargo test --workspace`
- `git status --short --branch`
- Last verified: 2026-04-13

## Milestones
- `[done]` Milestone 1: refresh durable memory for Phase 1
  - Scope: replace the stale browser-only workflow pass with the full Phase 1 contract
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs reflect current product decisions and implementation order
  - Verification steps: direct review of the updated docs
- `[done]` Milestone 2: define the shared flow contract and capability audit
  - Scope: pin down the exact golden path, playground behavior, and daemon/API gaps across all three surfaces
  - Key areas: daemon endpoints, app state payloads, current CLI commands, current TUI surface
  - Acceptance criteria: concrete contract exists for provider connect -> model sync -> key mint -> playground -> request inspection
  - Verification steps: targeted code reads and notes in docs
- `[done]` Milestone 3: daemon/operator API alignment
  - Scope: add or tighten the operator endpoints and payloads needed by Web UI, OpenTUI, and CLI parity
  - Key areas: `packages/app-daemon`, possibly `packages/app-storage`, shared request/playground handling
  - Acceptance criteria: all surfaces can rely on the same provider/model/key/request/playground data model
  - Verification steps: focused daemon tests
- `[done]` Milestone 4: Web UI golden path and playground
  - Scope: finish the primary guided flow in the local browser UI and add a simple playground
  - Key areas: `packages/app-daemon/src/browser_app.html`, daemon operator handlers
  - Acceptance criteria: one user can complete the whole flow and test a key from the browser UI
  - Verification steps: focused daemon tests plus manual inspection
- `[done]` Milestone 5: OpenTUI rewrite and parity
  - Scope: replace the Rust TUI with an OpenTUI implementation that mirrors the Web UI flow
  - Key areas: TUI package/runtime, launch wiring, daemon-backed data fetching, playground
  - Acceptance criteria: TUI is OpenTUI-based, feature-parallel for Phase 1, and does not own business logic
  - Verification steps: TUI-specific tests or smoke checks plus repo verification
- `[done]` Milestone 6: CLI parity and playground
  - Scope: make the Rust CLI fully capable for the same flow, including interactive chat after the one-shot baseline
  - Key areas: `packages/app-cli`, app entrypoint wiring
  - Acceptance criteria: everything meaningful in Phase 1 can be completed from CLI
  - Verification steps: CLI smoke tests and targeted Rust tests
- `[done]` Milestone 7: cleanup, verification, and push checkpoint(s)
  - Scope: keep diffs clean, rerun verification, and push when the checkpoint is strong enough
  - Key areas: touched packages, docs, git state
  - Acceptance criteria: repo is clean, tests are green, and `main` only moves when the slice is coherent
  - Verification steps: `bun run test`, `cargo test --workspace`, `git status`

## Acceptance Checks
- Durable memory states the Phase 1 decisions.
- Web UI, CLI, and TUI all support the single-user golden path.
- Playground uses only Gunmetal keys and synced models.
- Playground can exercise both `chat/completions` and `responses`.
- OpenTUI TUI rewrite is in place without duplicating app logic.
- Request history and token stats clearly validate playground and normal traffic.
- The workspace still tests green after each checkpoint.

## Validation
- `git status --short --branch`
- targeted `sed -n` reads across docs and touched files
- focused crate/package tests for the milestone being worked
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- `cargo run -p gunmetal -- --help`
- push to `origin/main` when the current slice is coherent and verified

## Decisions
- Product thesis: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- User-facing noun is `provider`; internal implementation noun is `extension`.
- The app is the product. The SDK is internal first and powers the app.
- Request history and token stats are the first built-in toll-booth benefits to preserve and strengthen later.
- Web UI, CLI, and TUI are all first-class surfaces.
- Web UI is the primary guided surface.
- CLI stays in Rust.
- TUI will be rewritten in OpenTUI.
- OpenTUI should behave like the Web UI in terminal form and consume daemon/operator endpoints rather than owning business logic.
- Playground is part of Phase 1, not a separate future phase.
- Playground will use only Gunmetal keys.
- Playground will use synced models for model selection.
- Playground should support both `chat/completions` and `responses`.
- First request verification should happen through the playground rather than a separate pre-playground test step.
- Start simple where needed, but Phase 1 still aims for full parity rather than leaving the TUI or CLI half-finished.

## Implementation Notes
- Active work should land through a feature branch and only move `main` once the slice is coherent and verified.
- The SDK boundary already exists and is green.
- Repo-structure pass is already done.
- Product-language cleanup is already done.
- Browser-UI workflow polish is already done and should now be treated as the base layer, not the active phase.
- Implemented layout remains:
  - `apps/gunmetal`
  - `packages/sdk`
  - `packages/sdk-core`
  - `packages/extensions`
  - `packages/app-cli`
  - `packages/app-daemon`
  - `packages/app-storage`
  - `packages/app-tui`
- Existing verified UX base:
  - web/docs copy centers the local inference middle-layer thesis
  - browser UI already has setup checklist, traffic snapshot, adaptive provider form, request-history token breakdowns, and selected-request detail
  - CLI help/setup output uses provider-first language
  - existing Rust TUI wording is aligned, but the implementation is not the final terminal architecture
- Phase 1 build order:
  - shared flow contract and capability audit
  - daemon/operator API alignment
  - Web UI golden path and playground
  - OpenTUI rewrite
  - CLI parity and interactive playground
- Playground rollout intent:
  - one-shot/simple first where needed
  - interactive chat included within Phase 1, not deferred to a later phase
- Canonical Phase 1 object flow:
  - provider -> synced models -> Gunmetal key -> playground/request -> request history/token stats
- Capability audit findings:
  - existing `/v1/chat/completions` and `/v1/responses` already provide the real execution path, streaming behavior, and request logging needed for playgrounds
  - existing `/app/api/state` already exposes providers, synced models, keys, logs, setup state, and traffic summaries for operator surfaces
  - browser UI can add playground behavior without a separate backend execute endpoint
  - CLI can add an interactive `chat` flow by talking to the running local daemon
  - current Rust TUI is featureful enough to mine for flow ideas but should now be replaced by a thin launcher plus an OpenTUI app
  - daemon/API alignment in Phase 1 should stay small unless a concrete surface gap appears during implementation
- Implemented in Milestone 3:
  - kept the existing `/v1/chat/completions` and `/v1/responses` routes as the single playground execution path
  - kept request logging and token accounting on those real inference routes instead of adding a fake operator-side execute path
  - refreshed browser-shell coverage tests to include the new playground markers
- Implemented in Milestone 4:
  - local browser UI now includes a Gunmetal-key playground
  - playground supports both `chat/completions` and `responses`
  - playground uses synced-model selection and conversation-or-single-message modes
  - successful playground requests flow back into request history and token stats by reloading local state
- Implemented in Milestone 5:
  - `packages/app-tui` now contains a Bun/OpenTUI app entry at `src/opentui.js`
  - Rust `gunmetal-tui` is now a thin launcher that starts the OpenTUI surface
  - OpenTUI surface includes setup, playground, and request-inspection views backed by daemon endpoints
  - provider actions, key creation, request inspection, and playground execution all stay daemon-backed
- Implemented in Milestone 6:
  - added `gunmetal chat`
  - CLI chat supports interactive use and one-shot `--prompt` mode
  - CLI chat supports both `chat` and `responses` modes
  - CLI chat streams deltas from the same local `/v1/...` routes as the browser and TUI playgrounds

## Risks
- Risk: TUI rewrite drags core product work into UI-framework churn.
  - Mitigation: keep OpenTUI purely as a daemon-backed surface and move product logic only in Rust backend paths.
- Risk: all-three-surfaces parity balloons scope.
  - Mitigation: keep one shared flow contract and deliver in the fixed milestone order.
- Risk: playground scope expands into a full chat product.
  - Mitigation: Phase 1 playground stays focused on test/demo of Gunmetal keys and models.
- Risk: inbound request-mode support gets fragmented.
  - Mitigation: use one simple UX with explicit mode selection for `chat/completions` and `responses`.
- Risk: OpenTUI currently depends on Bun at runtime.
  - Mitigation: keep the Rust side as a launcher boundary so packaging can be improved later without redoing the TUI itself.

## Architecture
- Product: local-first inference middle layer for individuals.
- Canonical flow: `app/tool -> Gunmetal key -> Gunmetal -> provider extension -> upstream provider`.
- Core product nouns:
  - provider accounts
  - models
  - Gunmetal keys
  - requests
  - usage stats
- Internal architecture target:
  - inbound compatibility layer
  - Gunmetal core request/control layer
  - Gunmetal SDK for provider extensions
  - first-party provider extensions
