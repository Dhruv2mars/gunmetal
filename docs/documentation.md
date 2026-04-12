# Documentation

This document is updated continuously as milestones land so it reflects reality.

## What This Work Is
- This pass initializes durable memory for `gunmetal` before any follow-on implementation. It captures the current product shape, repository layout, work-history arc, and the operating rules future tasks should follow.

## Current Status
- Durable memory is now initialized at the repo root in `docs/`.
- The current product is a local-first AI switchboard: one local OpenAI-compatible API, one local browser control plane, one CLI, and one TUI over shared local state.
- Current branch for this docs work: `docs/durable-memory-init`.

## Status By Milestone
- Milestone 1 complete: repo guidance, workspace shape, and git history reviewed.
- Milestone 2 complete: durable-memory docs created.
- Milestone 3 complete: docs reviewed for consistency against the inspected repo state.

## Setup And Verification
- Repo state:
  - `git status --short --branch`
  - `git log --oneline --decorate --graph --all -n 80`
  - `git log --reverse --oneline --no-decorate`
  - `git rev-list --count --all`
  - `git tag --sort=creatordate`
- Workspace inspection:
  - `find apps packages -maxdepth 2 -type f | sort`
  - targeted `sed -n` reads of `README.md`, workspace manifests, and key source files
- Durable-memory verification:
  - `test -f docs/prompt.md && test -f docs/plans.md && test -f docs/implement.md && test -f docs/documentation.md`
  - direct review of the new docs

## Completed Work
- Captured the current repo organization:
  - `apps/cli`: Rust entrypoint that boots the daemon for default/TUI usage or dispatches CLI commands.
  - `apps/web`: Next.js hosted front door with install/docs/changelog/web-ui pages.
  - `packages/cli`: command parsing and operational flows such as setup, web, start, status, profiles, auth, models, keys, and logs.
  - `packages/daemon`: local HTTP service exposing `/health`, browser control-plane endpoints under `/app/api/*`, and `/v1/models`, `/v1/chat/completions`, `/v1/responses`.
  - `packages/providers`: provider adapters and registry/hub logic.
  - `packages/storage`: local app-path resolution and SQLite-backed persistence.
  - `packages/core`: shared domain types and API contracts.
  - `packages/tui`: terminal dashboard/operator surface.
  - `packages/npm`: npm distribution wrapper that installs the native binary.
- Captured the visible work-history arc from git:
  - foundation and bootstrap
  - provider rollout for codex, copilot, openrouter, zen, and openai
  - responses API support
  - first-run and TUI hardening
  - monorepo restructure
  - npm packaging and release pipeline
  - hosted site plus local browser UI
  - several release/CI/acceptance stabilization cycles
  - latest visible release tag: `v0.1.8`

## Validation Results
- The repo was clean before this task except for branch state.
- The new durable-memory files now exist in `docs/`.
- The docs match the inspected workspace and history and do not claim unverified roadmap work.

## Decisions
- Use the durable-memory set for future project work in this repo.
- Keep the memory anchored to the current task and actual repo state instead of turning it into generic project notes.
- Treat git history as the source for fine-grained progress details when later work needs subsystem-level context.

## Next Steps
- Start the next requested task using `docs/prompt.md` and `docs/plans.md` as the baseline.
- Refresh the durable-memory files at the start of the next substantive task so the active scope is explicit.

## Follow-Ups
- If the next task targets `apps/web`, read `apps/web/AGENTS.md` before making web changes.
- If the next task depends on a specific regression or release, inspect the relevant commits directly instead of relying only on this summary.
