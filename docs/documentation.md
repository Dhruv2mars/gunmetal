# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- Phase 2 is a combined superapp and platform pass: improve real-world usability in the Gunmetal super app while also hardening the internal architecture and extension SDK that power it.

## Current Status
- Durable memory exists and is updated for Phase 2.
- Product thesis remains: Gunmetal is the local-first inference middle layer that turns AI subscriptions and provider accounts into one programmable API.
- Product taxonomy remains explicit:
  - `Products`: consumer-facing Gunmetal products, with the super app as the current product
  - `Developer`: internal engines that later become public developer products, starting with the provider/extension SDK
- Phase 1 remains the stable base.
- The active work is now deeper than traffic summaries alone:
  - shared provider metadata
  - provider-guided superapp flows
  - cleaner internal provider/runtime boundaries

## Status By Milestone
- Milestone 1 complete: durable-memory refresh and Phase 2 branch.
- Milestone 2 complete: usability and architecture audit.
- Milestone 3 complete: extension SDK hardening.
- Milestone 4 complete: superapp usability improvements.
- Milestone 5 complete: CLI and daemon alignment.
- Milestone 6 complete: repo-wide verification, cleanup, and push.

## Implemented In Phase 2
- `packages/sdk/src/lib.rs`
  - added explicit provider capability metadata
  - added provider UX hints
  - exposed provider definitions through the internal registry and hub
- `packages/extensions/src/lib.rs`
  - first-party providers now declare real capability and UX metadata
- `packages/app-daemon/src/lib.rs`
  - daemon now uses shared provider definitions instead of local hardcoded provider branching
  - operator state now includes richer provider and request metadata
- `packages/app-daemon/src/browser_app.html`
  - browser provider setup is now driven by live provider definitions
  - unsupported provider options are no longer hardcoded into the setup form
  - request filtering and request detail are more useful in actual use
- `packages/app-tui/src/opentui.js`
  - OpenTUI provider setup uses the same provider metadata model
  - request inspection has richer filters and context
- `packages/app-cli/src/lib.rs`
  - CLI provider behavior now relies on the same provider definition model
  - provider listing is more informative
  - log inspection is more useful with richer filters and request context

## Why This Matters
- The super app is now less misleading:
  - it no longer advertises unsupported provider paths in the browser
  - setup guidance is based on actual provider capabilities
- The internal architecture is cleaner:
  - provider behavior is declared once in the SDK layer
  - daemon, browser UI, OpenTUI, and CLI consume that shared definition model
- The internal extension SDK is more publication-ready:
  - capabilities are explicit
  - UX hints are explicit
  - registry and hub expose the provider catalog directly

## Validation Results
- Focused verification completed:
  - `cargo test -p gunmetal-sdk`
  - `cargo test -p gunmetal-providers`
  - `cargo test -p gunmetal-daemon -p gunmetal-cli -p gunmetal-tui`
- Repo-wide verification completed:
  - `bun run test`
  - `bun run check`
  - `cargo test --workspace`
- Important smokes completed:
  - `cargo run -p gunmetal -- logs summary --limit 10`
  - `cargo run -p gunmetal -- providers list`
  - `cargo run -p gunmetal -- tui`

## Decisions
- Shared provider metadata is now the main system-design primitive for user-facing provider behavior.
- Phase 2 deliberately improves the current product rather than adding another product surface.
- The internal SDK is still not public, but its contracts are now being shaped as if they will eventually need to be public.

## Next Steps
- After Phase 2, the next likely phase should deepen request drill-down, longer-window usage summaries, and local control policies on top of this cleaner provider contract.
