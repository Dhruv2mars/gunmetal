# Prompt

## Task
- Execute Phase 1 of the first-principles refactor: perfect the single-user golden path across Web UI, CLI, and TUI while rewriting the TUI in OpenTUI and adding a minimal playground.

## Core Goal
- Make Gunmetal feel complete for one local user: connect a provider, finish auth, sync models, mint a Gunmetal key, make a real request, and verify it through a playground and request history on every first-class surface.

## Goals
- Refresh durable memory to the actual active phase.
- Treat the browser UI as the primary guided flow without making CLI or TUI second-class.
- Keep CLI fully capable in Rust.
- Rewrite the TUI in OpenTUI as a terminal-native mirror of the Web UI.
- Add a simple playground to Web UI, TUI, and CLI.
- Support both `chat/completions` and `responses` in the playground flow.
- Use only Gunmetal keys in the playground.
- Use synced models for model selection.
- Strengthen request history and token stats as the built-in toll-booth value.

## Non-Goals
- Do not publish the SDK yet.
- Do not split the repo or product.
- Do not add team or multi-tenant concepts.
- Do not expose internal `extension` terminology in normal user UX.
- Do not broaden the playground into stored chats, tools, attachments, or prompt libraries in Phase 1.

## Constraints
- Be concise.
- Verify work before reporting back when possible.
- Use the durable-memory files as the source of truth.
- Keep user-facing terminology simple: `provider`, `model`, `key`, `request`.
- Product is local-first and individual-first.
- Web UI and OpenTUI should consume daemon/operator endpoints rather than duplicating business logic.
- CLI can use Rust service/core paths directly, but behavior should stay aligned with the daemon-backed surfaces.
- Keep the app powered by the internal SDK and first-party provider extensions.

## Deliverables
- Updated durable-memory docs for Phase 1
- A concrete implementation plan for backend, Web UI, OpenTUI, and CLI
- Golden-path improvements across all three surfaces
- A minimal Gunmetal-key playground on all three surfaces
- Verification notes for each milestone

## Process Requirements
- Update durable memory before broad implementation.
- Work in ordered milestones:
  - milestone 1: phase contract and backend capability audit
  - milestone 2: daemon/operator API alignment for golden path and playground
  - milestone 3: Web UI golden path and playground
  - milestone 4: OpenTUI rewrite and parity
  - milestone 5: CLI parity including playground/testing flow
  - milestone 6: verification, cleanup, and push checkpoints when ready
- Keep the docs current as decisions land.

## Done When
- A single user can complete the full local Gunmetal flow from any first-class surface.
- Web UI, CLI, and TUI share the same nouns and core capabilities.
- The TUI runs on OpenTUI without owning business logic.
- Playground works with Gunmetal keys and synced models and can exercise both `chat/completions` and `responses`.
- Request history and token stats clearly confirm what happened after a real request.
