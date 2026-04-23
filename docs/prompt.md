# Prompt

## Task
- Remove the TUI surface completely.
- Make CLI plus Web UI the canonical control/setup path.
- Clean dead, obsolete, and misleading code/docs after TUI removal.
- Continue cleanup by removing dead web marketing scaffolding and unused dependencies.

## Core Goal
- Gunmetal ships as a local-first API daemon with:
  - CLI for install/setup/service/scripted control
  - Web UI for operator workflows
  - SDK/provider/storage internals behind those surfaces
- No `gunmetal tui`, OpenTUI package, TUI crate, TUI dependency, or TUI docs should remain.

## Tracks
- Track A: durable docs and product wording
  - replace old GA/TUI milestone language
  - document CLI + Web UI as the product surface
- Track B: Rust workspace cleanup
  - remove `packages/app-tui`
  - remove `gunmetal-tui` dependency and launch path
  - make no-command CLI behavior useful without the TUI
- Track C: JS/web/npm cleanup
  - remove OpenTUI workspace and lockfile entries
  - remove TUI copy from public site, README, npm metadata, and repo tests
- Track D: verification
  - run focused CLI/workspace checks
  - run full repo test/check when feasible
- Track E: post-TUI cleanup
  - remove placeholder web routes not part of current product
  - remove unused components, assets, helper modules, and JS deps
  - keep only real public routes: home, Web UI, start-here, docs, install

## Non-Goals
- Do not add new product surfaces.
- Do not publish packages.
- Do not change provider behavior unless cleanup requires it.

## Constraints
- Be concise.
- Verify before reporting back when possible.
- Use durable-memory files as source of truth.
- Keep user-facing terminology simple: `provider`, `model`, `key`, `request`.
- Product remains local-first and individual-first.

## Done When
- TUI code/package/crate/command/docs are gone.
- Default `gunmetal` invocation no longer launches a TUI.
- CLI help points users to setup, web, start, status, chat, and logs.
- Workspace manifests and tests match the smaller architecture.
- Verification passes or failures are clearly reported.
- Unused web scaffolding no longer ships or bloats lockfiles.
