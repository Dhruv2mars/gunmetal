# Prompt

## Task
- Execute the GA shipping pass for the Gunmetal super app.
- Work in strict surface order:
  - landing page
  - CLI
  - TUI
  - Web UI

## Core Goal
- Make the Gunmetal super app public-usable and polished enough for a broad public launch.
- Treat this as shipping work, not exploratory refactor work:
  - product story must be clear
  - install and operator flows must feel finished
  - copy, UX, errors, and defaults must be fit for public use
  - final work must be tested before GA

## Tracks
- Track A: public landing page
  - sharpen the super-app story
  - present installation, supported providers, and surfaces clearly
  - make the hosted front door feel finished rather than internal
- Track B: CLI shipping polish
  - tighten help text, golden paths, recovery paths, and command ergonomics
  - make the CLI feel reliable for first-time public users
- Track C: TUI shipping polish
  - make the terminal experience feel production-ready
  - preserve parity with the browser flow where intended
- Track D: Web UI shipping polish
  - make the local browser operator flow clear, stable, and demo-safe
  - keep it clean enough for public screenshots and real use
- Track E: final GA verification
  - verify all four public surfaces
  - run the relevant tests and live checks before reporting GA readiness

## Non-Goals
- Do not publish SDK packages yet.
- Do not add team or multi-tenant concepts.
- Do not split the repo into separate products.
- Do not widen scope beyond the super app during this pass.

## Constraints
- Be concise.
- Verify work before reporting back when possible.
- Use the durable-memory files as the source of truth.
- Keep user-facing terminology simple: `provider`, `model`, `key`, `request`.
- Product is still local-first and individual-first.
- Focus only on the Gunmetal super app for now.
- Product taxonomy remains:
  - `Products`: consumer-facing Gunmetal products, with the super app as the current product
  - `Developer`: internal engines that later become developer products, starting with the provider/extension SDK

## Deliverables
- Updated durable memory for the GA shipping pass
- A public-facing landing page for the Gunmetal super app
- Shipping-level CLI polish
- Shipping-level TUI polish
- Shipping-level Web UI polish
- Final verification notes for the full public surface area

## Done When
- The landing page can act as the public front door for the super app.
- CLI, TUI, and Web UI each feel complete enough for first-time public users.
- The full golden path is clean across the public surfaces.
- The product is verified before being treated as GA-ready.
