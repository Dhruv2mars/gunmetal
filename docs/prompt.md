# Prompt

## Task
- Establish durable memory for `gunmetal` before follow-on work. Inspect the current codebase and git history, capture the product vision and present architecture, and seed the required `docs/` files so future tasks can use them as the source of truth.

## Core Goal
- Create a concise, current durable-memory baseline for the project and this prerequisite phase.

## Goals
- Capture the product shape, workspace structure, and main runtime surfaces as they exist on `main`.
- Summarize the major implementation and release history visible in git.
- Record the commands and checks used to inspect and verify the repo state.
- Create `docs/prompt.md`, `docs/plans.md`, `docs/implement.md`, and `docs/documentation.md`.

## Non-Goals
- Do not change product behavior, APIs, UI, or release metadata.
- Do not rewrite repository docs outside the new durable-memory set unless later work requires it.
- Do not infer roadmap details that are not supported by the current codebase or git history.

## Constraints
- Be concise.
- Verify the work before reporting back when possible.
- Use the durable-memory files as the source of truth for subsequent work in this repo.
- Keep the docs grounded in the actual repository state, not generic project summaries.

## Deliverables
- `docs/prompt.md`
- `docs/plans.md`
- `docs/implement.md`
- `docs/documentation.md`

## Process Requirements
- Read repo guidance and local durable-memory guidance before substantial work.
- Inspect the current workspace and git history before writing the docs.
- Keep the docs current if scope changes in later work.

## Done When
- All four durable-memory files exist in `docs/`.
- The docs reflect the current product architecture and visible work history.
- The docs record how this prerequisite pass was verified.
