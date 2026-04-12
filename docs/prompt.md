# Prompt

## Task
- Continue the first-principles refactor by improving actual usability on top of the completed copy pass. The cleanest next slice is the local browser UI and its state payload: make provider setup progress obvious, surface request/token stats clearly, and keep the operator experience clutter-free.

## Core Goal
- Make the local browser UI feel like the clearest control center for Gunmetal’s core flow: connect a provider, create a key, send traffic, inspect requests and tokens.

## Goals
- Update durable memory for the browser-UI usability scope.
- Improve the local `/app` operator flow without adding clutter.
- Extend the state payload just enough to support better request/token visibility and setup progress.
- Preserve command names, APIs, and storage contracts outside this slice unless a tiny UX-oriented addition is clearly justified.
- Keep the repo green after the UI/state improvements.

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
- Updated durable-memory docs for the usability pass
- Improved local browser UI layout/content for setup progress and traffic visibility
- Small `/app/api/state` additions needed to power that UI
- Verification notes for the UI/state changes

## Process Requirements
- Update durable memory before broad implementation.
- Work in four phases:
  - phase 1: durable-memory refresh
  - phase 2: inspect the current browser UI and state payload
  - phase 3: implement the clutter-free browser/UI state improvements
  - phase 4: verify and commit the refactor slice
- Keep the docs current as decisions land.

## Done When
- Durable memory reflects the browser-UI usability decisions.
- The local browser UI makes provider setup progress obvious.
- Request history and token stats are more visible without making the page noisy.
- The repo still builds/tests for the touched areas.
