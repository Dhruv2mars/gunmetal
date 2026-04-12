# Prompt

## Task
- Continue the first-principles refactor by improving actual usability on top of the completed copy pass. The current slice is the local browser UI workflow itself: make provider setup adapt to the selected provider and make request history support a real drill-down without clutter.

## Core Goal
- Make the local browser UI feel like the clearest control center for Gunmetal’s core flow: connect the right provider faster, then inspect a selected request with useful latency and token detail.

## Goals
- Update durable memory for this workflow-polish pass.
- Make the provider form adapt to the selected provider type.
- Add a compact request-detail drill-down above the request-history table.
- Preserve command names, APIs, and storage contracts outside this slice unless a tiny UX-oriented addition is clearly justified.
- Keep the repo green after the UI improvements.

## Non-Goals
- Do not publish the SDK yet.
- Do not redesign the app from scratch in this pass.
- Do not split this into multiple repos.
- Do not broaden this pass into a full CLI/TUI overhaul.
- Do not change provider integration behavior unless needed for the browser-UI improvements.

## Constraints
- Be concise.
- Verify the work before reporting back when possible.
- Use the durable-memory files as the source of truth for subsequent work in this repo.
- Keep the product framing simple for users: user-facing noun is `provider`, internal noun is `extension`.
- The product is local-first and individual-first.
- Gunmetal remains a single product, not multiple separate apps/repos.
- Reinforce the first built-in toll-booth benefits: request history and token stats.
- Keep the browser UI visually intentional but uncluttered.
- Prefer small state-payload additions over large architectural changes.

## Deliverables
- Updated durable-memory docs for the workflow-polish pass
- Improved provider form behavior in the local browser UI
- Request-detail drill-down in the local browser UI
- Verification notes for the UI changes

## Process Requirements
- Update durable memory before broad implementation.
- Work in four phases:
  - phase 1: durable-memory refresh
  - phase 2: inspect the current browser UI workflow
  - phase 3: implement provider-form and request-detail improvements
  - phase 4: verify and decide whether to push this checkpoint to main
- Keep the docs current as decisions land.

## Done When
- Durable memory reflects the workflow-polish decisions.
- The local browser UI helps the user enter the right provider details faster.
- Request history supports a useful selected-request view.
- The repo still builds/tests for the touched areas.
