# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `sed -n '1,220p' docs/prompt.md`
- `sed -n '1,260p' docs/plans.md`
- `sed -n '1,220p' docs/implement.md`
- `sed -n '1,260p' docs/documentation.md`
- `rg -n "tui|TUI|OpenTUI|app-tui|gunmetal tui|@opentui|gunmetal-tui" . -S`
- `cargo test -p gunmetal-cli`
- `cargo test --workspace`
- `bun run test`
- `bun run check`
- `cargo run -p gunmetal -- --help`
- `cargo run -p gunmetal`
- `cargo run -p gunmetal -- status`
- `git status --short --branch`
- Last verified: 2026-04-23

## Milestones
- `[done]` Milestone 1: durable memory retarget
  - Scope: replace old GA/TUI plan with TUI-removal architecture.
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/documentation.md`
  - Acceptance criteria: docs say CLI + Web UI are canonical; TUI is out.
- `[done]` Milestone 2: remove TUI code and command surface
  - Scope: delete `packages/app-tui`, remove workspace/dependency entries, remove `gunmetal tui`.
  - Key areas: `Cargo.toml`, `Cargo.lock`, `apps/gunmetal`, `packages/app-cli`
  - Acceptance criteria: Rust workspace builds without any TUI crate or command.
- `[done]` Milestone 3: cleanup docs/site/npm/tests
  - Scope: remove public TUI references and OpenTUI JS workspace metadata.
  - Key areas: `README.md`, `AGENTS.md`, `packages/npm/package.json`, `test/repo-structure.test.js`, `bun.lock`
  - Acceptance criteria: repo copy and tests reflect CLI + Web UI only.
- `[done]` Milestone 4: dead-reference scan and verification
  - Scope: search for stale TUI/dead references, run focused and full verification.
  - Key areas: full repo
  - Acceptance criteria: no TUI references remain except historical lock/test output if tool-generated; tests/checks pass or blockers are documented.
- `[done]` Milestone 5: web scaffold cleanup
  - Scope: remove placeholder marketing routes, unused UI components, unused assets, unused site content module, unused JS deps, unused Rust workspace deps, and stale CLI public helpers.
  - Key areas: `apps/web`, `apps/web/package.json`, `bun.lock`, `Cargo.toml`, `packages/app-cli`, `test/repo-structure.test.js`
  - Acceptance criteria: web app exposes only current real routes and tests/checks pass.

## Acceptance Checks
- No TUI package, Rust crate, command, docs, npm keyword, or web copy remains.
- `gunmetal` with no subcommand shows help instead of launching a UI.
- `gunmetal web` remains the canonical graphical setup/control surface.
- Repo-structure guard expects the reduced workspace.
- Lockfiles no longer include OpenTUI workspace/dependencies.
- Web app no longer carries unused motion/component/template scaffolding.
- Placeholder `Coming soon` product/developer/download/changelog pages are gone.

## Decisions
- TUI is removed, not hidden.
- CLI handles setup, service control, auth, keys, models, chat, and logs.
- Web UI handles graphical control and setup.
- `apps/gunmetal` is now the native CLI entrypoint.

## Validation
- `rg -n "packages/app-tui|@gunmetal/tui|gunmetal-tui|GUNMETAL_TUI|OpenTUI|@opentui|gunmetal tui|terminal UI|terminal experience|\\bTUI\\b|\\btui\\b" . -S --glob '!docs/**'`
  - clean except one negative CLI help assertion.
- `cargo run -p gunmetal -- --help`
  - no `tui` command; root invocation is CLI help.
- `cargo run -p gunmetal`
  - prints CLI help instead of launching a TUI.
- `cargo run -p gunmetal -- status`
  - stopped recovery says `gunmetal start` or `gunmetal web`.
- `npm exec --yes bun@1.3.5 -- run test`
  - pass.
- `npm exec --yes bun@1.3.5 -- run check`
  - pass.
- `npm exec --yes bun@1.3.5 -- run --filter @gunmetal/web build`
  - pass; static routes are `/`, `/webui`, `/start-here`, `/docs`, and `/install`.
- `cargo test -p gunmetal-cli`
  - pass.
- `rg -n "framer-motion|clsx|tailwind-merge|@/lib/utils|tokio-stream|tokio_stream|arboard|tower-http|tower_http|tracing-subscriber|tracing_subscriber|gunmetal-tui|app-tui|@opentui" apps packages Cargo.toml Cargo.lock bun.lock -S`
  - clean except `tower-http` remains as a transitive lockfile dependency.
- `rg -n "products/suite|developer/sdk|/download|/changelogs|Coming soon|site-content|components/sections|file\\.svg|globe\\.svg|next\\.svg|vercel\\.svg|window\\.svg" apps/web test README.md docs -S`
  - clean outside this cleanup documentation.

## Risks
- Risk: stale TUI references linger in public docs or tests.
  - Mitigation: run full-repo `rg` scans with TUI/OpenTUI/package names.
- Risk: lockfiles retain removed packages.
  - Mitigation: regenerate Bun and Cargo lockfiles after manifest edits.
- Risk: web nav links drift to deleted routes.
  - Mitigation: nav is flat and points only at real routes.
