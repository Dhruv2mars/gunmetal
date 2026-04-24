# Prompt

## Task
- Rebuild Gunmetal UX quality from first principles after the refactor that removed TUI and dead surfaces.
- Current target surfaces:
  - hosted docs/product routes in `apps/web`, but keep landing page + shared landing navbar on pre-2026-04-23 state unless user explicitly asks
  - local browser Web UI served by `packages/app-daemon`
  - CLI in `packages/app-cli`

## Core Goal
- Make Gunmetal feel obvious, calm, and trustworthy for a first-time individual user.
- The product story must be concrete:
  - install Gunmetal
  - connect one provider
  - sync models
  - mint one local key
  - point apps at the OpenAI-compatible local API
  - inspect traffic when something succeeds or fails

## UX Principles
- Local-first stays central. No team, cloud account, or hosted control-plane framing.
- Use simple terms: `provider`, `model`, `key`, `request`.
- Prefer guided next steps over feature lists.
- Show real commands and real URLs before abstract copy.
- Treat empty states and error recovery as primary UX, not fallback text.
- Keep the Web UI dense enough for operators, but not visually noisy.
- Keep the CLI readable in normal terminals and useful when copied into logs.

## Taste Direction
- Technical software UI: no serif, no Inter, no purple/blue AI-gradient look.
- Use restrained neutral surfaces with one calibrated accent.
- Avoid centered generic hero patterns and equal three-card marketing rows.
- Motion must be purposeful, transform/opacity-only, and reduced-motion aware.
- UI controls need stable dimensions and no mobile text overlap.

## Non-Goals
- Do not reintroduce TUI.
- Do not publish SDK packages.
- Do not add auth, accounts, teams, hosted sync, or multi-tenant concepts.
- Do not widen beyond the super-app Web UI and CLI path.
- Do not redesign or tweak landing page/navbar unless user explicitly asks.

## Deliverables
- Updated durable memory for this UX pass.
- Stronger hosted marketing/docs routes.
- Stronger local browser Web UI hierarchy, empty states, and action flow.
- Stronger CLI first-run and diagnosis ergonomics.
- Verification notes from tests and live checks where possible.

## Done When
- A new user can understand what Gunmetal is and how to try it from the site.
- The local Web UI makes the next setup/action obvious at every state.
- The CLI can answer “what should I do next?” without making users inspect storage manually.
- Tests pass for touched surfaces.
