# Plans

Keep this current while the UX pass moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,260p' docs/plans.md`
- `sed -n '1,220p' docs/implement.md`
- `sed -n '1,260p' docs/documentation.md`
- `bun run --filter @gunmetal/web test`
- `bun run --filter @gunmetal/web lint`
- `cargo test -p gunmetal-cli`
- `cargo test -p gunmetal-daemon`
- `cargo run -p gunmetal -- --help`
- `cargo run -p gunmetal -- doctor`
- live browser check for hosted pages or local `/app` when server checks are available
- `git status --short --branch`
- Last verified: 2026-04-23

## Milestones
- `[done]` Milestone 1: reset durable memory and UX direction
  - Scope: remove stale TUI/GA language and define the new Web UI + CLI target.
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance: docs match the refactored product surface.
- `[done]` Milestone 2: hosted site UX foundation
  - Scope: rebuild marketing/docs/install/start/webui routes around the golden path.
  - Key areas: `apps/web/src/app`, shared layout/components/styles
  - Acceptance: pages explain the product, setup path, and local URLs without generic SaaS copy.
- `[done]` Milestone 3: local browser Web UI polish
  - Scope: improve operator hierarchy, empty states, controls, and mobile resilience.
  - Key areas: `packages/app-daemon/src/browser_app.html`
  - Acceptance: first action is visible, playground is clear, request history is readable.
- `[done]` Milestone 4: CLI UX polish
  - Scope: improve first-run diagnosis and command recovery.
  - Key areas: `packages/app-cli`
  - Acceptance: CLI has a concise “what next?” surface and better help language.
- `[done]` Milestone 5: verification
  - Scope: targeted web, daemon, CLI tests; broader checks if touched surface risk warrants.
  - Key areas: repo test scripts and command smokes
  - Acceptance: touched surface tests pass, live checks recorded when possible.

## Decisions
- TUI is removed from active product scope.
- Browser Web UI and CLI are now primary local operator surfaces.
- Marketing site supports the local product, not a separate hosted app.
- Public copy must say what users do with real commands and URLs.
- Add only UX-supporting functionality; avoid new product scope.
- Landing page and shared landing navbar are frozen to pre-2026-04-23 state unless user explicitly expands scope.

## Risks
- Risk: visual polish hides unclear product mechanics.
  - Mitigation: keep golden path and API contract visible on first screen.
- Risk: CLI grows noisy.
  - Mitigation: add one diagnostic path instead of many overlapping commands.
- Risk: local Web UI becomes too dashboard-heavy.
  - Mitigation: keep primary action and current setup state above dense tables.

## Active Notes
- Current branch: `ux-web-cli-first-principles`.
- Current source-of-truth: Web UI + CLI UX, no TUI.
- Hosted site live-checked at `http://localhost:3000`.
- Local browser UI live-checked at `http://127.0.0.1:4684/app`.
- Screenshot evidence saved under `.codex/screenshots/`.
- Landing design source-of-truth is now `DESIGN.md`.
- Added hosted subpages for `/products/suite`, `/developer/sdk`, `/download`, `/docs`, `/changelogs`, and `/changelog`.
- Changelog implementation follows Offdex's GitHub Releases fetch/normalize/fallback model, restyled for Gunmetal.
