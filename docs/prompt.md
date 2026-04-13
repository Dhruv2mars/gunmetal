# Prompt

## Task
- Continue after Phase 1 by deepening Gunmetal's built-in toll-booth value: richer request inspection and usage summaries across the existing Web UI, CLI, and OpenTUI surfaces.

## Core Goal
- Keep the completed Phase 1 flow intact, then make traffic inspection materially more useful: answer what provider or model is being used, how requests are performing, where tokens are going, and which requests need attention.

## Goals
- Refresh durable memory to the actual active phase and product taxonomy.
- Preserve the completed Phase 1 golden path without reopening its architecture.
- Add provider- and model-level traffic summaries on top of existing request history.
- Make request history more navigable with lightweight filtering.
- Keep Web UI, CLI, and OpenTUI aligned around the same traffic concepts.
- Keep the SDK internal and the product local-first.

## Non-Goals
- Do not publish the SDK yet.
- Do not split the repo or product.
- Do not add team or multi-tenant concepts.
- Do not broaden the playground into a full chat product.
- Do not add a second architecture for traffic data; reuse daemon-backed state.

## Constraints
- Be concise.
- Verify work before reporting back when possible.
- Use the durable-memory files as the source of truth.
- Keep user-facing terminology simple: `provider`, `model`, `key`, `request`.
- Product is local-first and individual-first.
- Gunmetal now has two top-level mental buckets:
  - `Products`: consumer-facing Gunmetal apps and services, with the current super app as the active product
  - `Developer`: internal engines that later become public developer products, starting with the provider/extension SDK that powers the super app
- Web UI and OpenTUI should consume daemon/operator endpoints rather than duplicating business logic.
- CLI can use Rust service/core paths directly, but behavior should stay aligned with the daemon-backed surfaces.
- Keep the app powered by the internal SDK and first-party provider extensions.

## Deliverables
- Updated durable-memory docs for the post-Phase-1 slice
- Provider/model traffic summaries in operator state and user surfaces
- Better request-history navigation
- CLI traffic inspection improvements
- Verification notes for the new slice

## Process Requirements
- Update durable memory before broad implementation.
- Work in ordered milestones:
  - milestone 1: refresh durable memory for the post-Phase-1 traffic-inspection phase
  - milestone 2: audit current traffic/request surfaces and define the additive data model
  - milestone 3: implement daemon-backed provider/model summaries
  - milestone 4: expose the new traffic intelligence in Web UI and OpenTUI
  - milestone 5: improve CLI traffic inspection
  - milestone 6: verification, cleanup, and push checkpoints when ready
- Keep the docs current as decisions land.

## Done When
- Traffic summaries answer where requests and tokens are going by provider and by model.
- Request history is easier to inspect without adding surface clutter.
- Web UI, CLI, and OpenTUI stay aligned around the same traffic concepts.
- The product/developer split is captured in durable memory without changing the current repo split or product scope.
