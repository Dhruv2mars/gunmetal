# Prompt

## Task
- Execute Phase 2 of the refactor: improve real-world superapp usability while hardening the internal architecture and extension SDK that power it.

## Core Goal
- Make Gunmetal feel better in actual use, not just in demos: the super app should guide users through the right providers, explain traffic clearly, and stay backed by cleaner internal contracts that can later be exposed as public developer products.

## Tracks
- Track A: superapp usability
  - remove unsupported provider affordances
  - make provider setup guided by real provider metadata
  - improve request inspection with better filtering and drill-down context
- Track B: internal architecture and system design
  - replace hardcoded provider behavior branches with shared provider definitions
  - keep daemon/operator state as the source of truth for Web UI and OpenTUI
  - tighten the boundary between product UI logic and internal provider/runtime logic
- Track C: extension SDK hardening
  - make provider capabilities explicit in the SDK
  - expose provider definitions through the internal hub/registry
  - prepare the internal extension layer for future publication without publishing yet

## Non-Goals
- Do not publish SDK packages yet.
- Do not add team or multi-tenant concepts.
- Do not split the repo into separate products.
- Do not broaden the playground into a full chat app.

## Constraints
- Be concise.
- Verify work before reporting back when possible.
- Use the durable-memory files as the source of truth.
- Keep user-facing terminology simple: `provider`, `model`, `key`, `request`.
- Product is still local-first and individual-first.
- Product taxonomy remains:
  - `Products`: consumer-facing Gunmetal products, with the super app as the current product
  - `Developer`: internal engines that later become developer products, starting with the provider/extension SDK

## Deliverables
- Updated durable memory for Phase 2
- Shared provider capability metadata in the internal SDK
- Web UI, OpenTUI, and CLI behavior driven by the same provider metadata
- Better request inspection and filtering in the super app
- Verification notes for all three tracks

## Done When
- The super app no longer exposes unsupported provider paths.
- Provider setup guidance comes from shared provider metadata rather than hardcoded UI branches.
- Request inspection is materially more useful in Web UI, OpenTUI, and CLI.
- The internal SDK/provider contracts are more explicit and more publication-ready than before.
