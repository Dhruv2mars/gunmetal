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
- `[done]` Milestone 1: refresh durable memory and branch for the GA shipping pass
  - Scope: replace the older Phase 2 wording with shipping-mode goals and the user-specified surface order
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs reflect landing page -> CLI -> TUI -> Web UI -> GA verification
  - Verification steps: direct review of the updated docs
- `[done]` Milestone 2: public landing page polish
  - Scope: rewrite the hosted front door so the super app story, install path, and supported surfaces are clear and public-ready
  - Key areas: `apps/web/src/app/page.tsx`, related marketing copy/tests/styles
  - Acceptance criteria: the landing page feels like a public product page, not internal project scaffolding
  - Verification steps: web tests, lint, browser check
- `[done]` Milestone 3: CLI shipping polish
  - Scope: tighten first-run help, command ergonomics, and recovery/error copy for public users
  - Key areas: `packages/app-cli`
  - Acceptance criteria: the CLI is clear and reliable for first-time public use
  - Verification steps: targeted CLI tests and command smokes
- `[done]` Milestone 4: TUI shipping polish
  - Scope: tighten the terminal UX, empty states, navigation, and parity with the browser flow
  - Key areas: `packages/app-tui`
  - Acceptance criteria: the TUI feels production-ready and coherent with the rest of the super app
  - Verification steps: TUI smoke and targeted tests
- `[done]` Milestone 5: Web UI shipping polish
  - Scope: tighten the local browser operator flow for screenshots, demos, and real use
  - Key areas: `packages/app-daemon` browser shell and operator state shaping
  - Acceptance criteria: the Web UI feels finished and public-usable
  - Verification steps: daemon tests and live browser checks
- `[done]` Milestone 6: final GA verification and release cleanup
  - Scope: run the full public-surface test pass, clean the repo, and move the checkpoint to `main`
  - Key areas: web, CLI, TUI, Web UI, docs, git state
  - Acceptance criteria: the full super app is coherent enough for public release
  - Verification steps: `bun run test`, `bun run check`, `cargo test --workspace`, live smokes, `git status`

## Acceptance Checks
- The landing page clearly sells the Gunmetal super app and its local-first value.
- The CLI is understandable and trustworthy for first-time public users.
- The TUI is polished enough to stand beside the Web UI rather than feel secondary.
- The Web UI is clean enough for public demos and real operation.
- The full install-to-first-request path is strong across the public surfaces.
- The repo stays focused on the Gunmetal super app during this pass.

## Validation
- `git status --short --branch`
- targeted `sed -n` reads across docs and touched files
- focused crate/package tests for the milestone being worked
- `bun run test`
- `bun run check`
- `cargo test --workspace`
- smoke `gunmetal logs summary`, `gunmetal tui`, and the local browser shell via daemon tests
- push to `origin/main` when the slice is coherent and verified
- final verification completed with:
  - `bun run test`
  - `bun run check`
  - live desktop and mobile browser captures for `/app`
  - live Web UI request check showing the current upstream Codex usage-limit failure and the matching request-history entry

## Decisions
- This pass is shipping work, not open-ended product expansion.
- The Gunmetal super app is the only product in scope for now.
- Work order is fixed by surface:
  - landing page
  - CLI
  - TUI
  - Web UI
- Public usability matters more than adding new features during this pass.
- The TUI pass must hold up at a normal 80-column terminal width, not just wide desktop terminals.
- The Web UI pass must keep the first-run path visible even when the local state already contains many old models and requests.

## Implementation Notes
- Active work should land through a feature branch and only move `main` once the slice is coherent and verified.
- Start with the hosted landing page because it becomes the public front door.
- Each later surface pass should keep the same product story and golden path.
- Canonical hosted truth for this product:
  - public landing page host is `gunmetalapp.vercel.app`
  - public Web UI marketing route is `/webui`
  - `gunmetal.vercel.app` is not this product and must not be used in copy or metadata

## Risks
- Risk: the landing page sounds clever but still leaves the product unclear.
  - Mitigation: keep the copy concrete: install, connect provider, mint key, use anywhere.
- Risk: surface-by-surface polish drifts into different product stories.
  - Mitigation: keep one canonical super-app thesis and reuse it across surfaces.
- Risk: public polish work stops at visuals and misses operational usability.
  - Mitigation: verify the real golden path, not just static rendering.

## Architecture
- Product: local-first inference middle layer for individuals.
- Canonical flow: `app/tool -> Gunmetal key -> Gunmetal -> provider extension -> upstream provider`.
- Product taxonomy:
  - `Products`: super app now, standalone user products later
  - `Developer`: internal engines now, standalone SDKs later
