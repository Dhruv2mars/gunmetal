# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,260p' docs/plans.md`
- `sed -n '1,260p' docs/documentation.md`
- `cargo test -p gunmetal-daemon`
- `bun run test`
- `cargo test --workspace`
- `git status --short --branch`
- Last verified: 2026-04-12

## Milestones
- `[done]` Milestone 1: refresh durable memory for the workflow-polish pass
  - Scope: rewrite the active task around the next local browser-UI flow improvements
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs reflect the narrowed workflow scope
  - Verification steps: direct review of the updated docs
- `[done]` Milestone 2: inspect the local browser UI workflow
  - Scope: identify the smallest changes that materially improve provider setup and request inspection
  - Key areas: `packages/app-daemon/src/browser_app.html`
  - Acceptance criteria: a tight implementation target exists before editing
  - Verification steps: targeted reads of the UI
- `[done]` Milestone 3: implement the workflow polish
  - Scope: adaptive provider form plus selected-request detail view
  - Key areas: `packages/app-daemon/src/browser_app.html`, `packages/app-daemon/src/lib.rs`
  - Acceptance criteria: the browser UI is more useful without feeling denser
  - Verification steps: daemon tests plus local UI source review
- `[done]` Milestone 4: verify and decide on main
  - Scope: rerun verification and decide whether to merge/push this checkpoint to `main`
  - Key areas: touched daemon/UI files plus branch status
  - Acceptance criteria: tests pass and the branch state is clear
  - Verification steps: `bun run test`, `cargo test --workspace`, `git status`

## Acceptance Checks
- Durable memory states the workflow-polish decisions.
- The provider form adapts to the selected provider.
- Request history has a useful selected-request view.
- The workspace still tests green after the pass.

## Validation
- `git status --short --branch`
- targeted `sed -n` reads across docs and the touched daemon/UI files
- `cargo test -p gunmetal-daemon`
- `bun run test`
- `cargo test --workspace`
- optional `git push origin main` if this checkpoint is clean enough

## Decisions
- Product thesis: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- User-facing noun is `provider`; internal implementation noun is `extension`.
- The app is the product. The SDK is internal first and powers the app.
- Request history and token stats are the first built-in toll-booth benefits to preserve and strengthen later.
- The current pass is usability-first on top of the already-finished app/sdk/extensions layout.
- Users should connect providers, mint a key, make requests, and then inspect request history and token stats.
- Keep command names and storage contracts stable in this pass unless a small UI/state addition is clearly worth it.
- The next workflow-polish slice should stay browser-UI-only unless a tiny daemon/API adjustment is required.

## Implementation Notes
- Current branch for this work: `refactor/internal-sdk`.
- The SDK boundary already exists and is green.
- The repo-structure pass is already done.
- The wording pass is already done.
- This pass should stay centered on the local browser UI rather than trying to fix every surface at once.
- Keep crate names and internal API routes stable where possible to avoid unnecessary churn.
- Implemented layout remains:
  - `apps/gunmetal`
  - `packages/sdk`
  - `packages/sdk-core`
  - `packages/extensions`
  - `packages/app-cli`
  - `packages/app-daemon`
  - `packages/app-storage`
  - `packages/app-tui`
- Audit findings:
  - web/docs copy still says `switchboard`, `control plane`, and `profile`
  - browser UI headings and banners still center `profile`
  - TUI tabs and details still center `Profiles`
  - CLI help/setup/output still uses older product wording
- Implemented in this pass:
  - web/docs copy now centers the local inference middle-layer thesis
  - browser UI headings and banners now say `provider` and `request history`
  - CLI help and setup output now describe connecting providers rather than saving profiles
  - TUI tabs and detail copy now present `Providers` and `Requests`
  - daemon/API recovery messages now use provider-first language where surfaced to operators
- Verification completed:
  - `cargo run -p gunmetal -- --help`
  - `cargo test -p gunmetal-cli -p gunmetal-daemon -p gunmetal-tui`
  - `bun run test`
  - `cargo test --workspace`
- Current usability target:
  - keep the browser UI visually clean
  - show setup progress without asking the user to infer it from counts alone
  - expose request/token stats more directly from `/app/api/state`
- Implemented in this pass:
  - `/app/api/state` now returns `setup` and `traffic` summary objects
  - per-request log rows now expose input/output token counts in addition to total tokens
  - the local browser UI now renders a compact setup checklist under the golden path
  - the local browser UI now renders a compact traffic snapshot with latency, success/error, and token totals
  - request-history rows now show token breakdowns and localized timestamps
- Focused verification completed:
  - `cargo test -p gunmetal-daemon`
- Repo-wide verification completed:
  - `bun run test`
  - `cargo test --workspace`
- Current workflow-polish target:
  - make the provider form adapt to the selected provider so users see only relevant fields and guidance
  - make request history support a selected-request detail view
- Implemented in this pass:
  - browser shell now includes a provider-form helper region
  - provider form now adapts for browser-sign-in vs API-key providers
  - browser shell now includes a selected-request detail region
  - request rows are selectable and render focused latency/token/endpoint detail
- Focused verification completed for this pass:
  - `cargo test -p gunmetal-daemon`
- Repo-wide verification completed for this pass:
  - `bun run test`
  - `cargo test --workspace`

## Risks
- Risk: the browser UI gains too much density while trying to show more interaction.
  - Mitigation: keep the detail view compact and reuse the existing panel instead of adding new columns.
- Risk: UX edits drift away from the product thesis.
  - Mitigation: keep the docs current and use them as the source of truth during the pass.
- Risk: browser-only behavior is weakly tested.
  - Mitigation: keep adding shell markers and rerun daemon plus repo-wide verification.

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
