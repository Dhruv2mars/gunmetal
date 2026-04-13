# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- Phase 1 is complete. The active slice now deepens Gunmetal's built-in toll-booth value instead of adding another surface: richer request inspection and usage summaries on top of the completed Web UI, CLI, and OpenTUI flow.

## Current Status
- Durable memory exists and is updated for the post-Phase-1 slice.
- Product thesis remains: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- Product taxonomy is now explicit:
  - `Products`: consumer-facing Gunmetal products, with the super app as the active product
  - `Developer`: internal engines that later become public developer products, starting with the provider/extension SDK
- The internal SDK extraction and repo-structure refactor remain complete and verified.
- Phase 1 remains the stable base:
  - Web UI, CLI, and OpenTUI are all first-class
  - playground exists across all three surfaces
  - request history and token stats already exist

## Status By Milestone
- Milestone 1 complete: durable-memory refresh for the post-Phase-1 slice.
- Milestone 2 complete: audit current traffic/request surfaces and define the additive data model.
- Milestone 3 complete: implement daemon-backed provider/model summaries.
- Milestone 4 complete: expose the summary layer in Web UI and OpenTUI.
- Milestone 5 complete: improve CLI traffic inspection.
- Milestone 6 in progress: verification, cleanup, and push checkpoints.

## Completed Context
- The current product is the Gunmetal super app.
- Future standalone user products are allowed, but they are not the current task.
- Future developer-facing products will be carved out of the internal engines that already power the super app.
- The provider/extension SDK remains internal-first and is still the likely first developer-facing product later.
- Current operator-state strengths:
  - provider/model/key/log visibility already exists
  - top-line traffic totals already exist
  - playground traffic already feeds the same request logs as normal inference traffic
- Current operator-state weakness:
  - request history is still mostly a flat recent log list
  - there is no first-class provider/model summary layer yet
  - CLI log inspection is still raw

## Validation Results
- Phase 1 verification already passed before this slice:
  - `cargo test -p gunmetal-cli -p gunmetal-daemon -p gunmetal-tui`
  - `bun run test`
  - `bun run check`
  - `cargo run -p gunmetal -- --help`
  - `cargo run -p gunmetal -- chat --help`
  - `cargo run -p gunmetal -- tui` smoke via the Rust launcher
- Current audit reads for the new slice:
  - `sed -n '1,220p' docs/prompt.md`
  - `sed -n '1,280p' docs/plans.md`
  - `sed -n '1,320p' docs/documentation.md`
  - targeted reads of:
    - `packages/app-daemon/src/lib.rs`
    - `packages/app-daemon/src/browser_app.html`
    - `packages/app-cli/src/lib.rs`
    - `packages/app-tui/src/opentui.js`
- Implementation verification for the new slice:
  - `cargo test -p gunmetal-daemon`
  - `cargo test -p gunmetal-cli`
  - `bun run test`
  - `bun run check`
  - `cargo run -p gunmetal -- logs summary --limit 10`
  - `cargo run -p gunmetal -- tui` smoke via `perl -e 'alarm 3; exec @ARGV' ...`

## Decisions
- The next slice should add traffic intelligence, not a new product surface.
- The new taxonomy is mental-model guidance for the repo and product story, not a repo split or a new publish step.
- The summary layer should answer:
  - which providers are carrying traffic
  - which models are consuming tokens
  - which requests are succeeding or failing
  - where latency is accumulating
- The summary layer should stay daemon-backed so Web UI and OpenTUI remain aligned.
- CLI should improve in the same slice so traffic inspection does not become UI-only.

## Implemented In This Slice
- `packages/app-daemon/src/lib.rs`
  - added provider-level and model-level traffic summaries to `/app/api/state`
- `packages/app-daemon/src/browser_app.html`
  - added top-provider and top-model summary cards
  - added provider/status request filters in the browser request view
- `packages/app-tui/src/opentui.js`
  - added provider/status request filters in the terminal request view
  - added compact top-provider and top-model summaries
- `packages/app-cli/src/lib.rs`
  - added `gunmetal logs summary`
  - added `--provider`, `--model`, and `--status` filters to `gunmetal logs list`

## Next Steps
- Finish cleanup and push the verified checkpoint if the slice still looks coherent after final review.
- After this, keep deepening Gunmetal's control value rather than adding more operator chrome:
  - richer request drill-down
  - stronger usage summaries over longer windows
  - local policy/control hooks
