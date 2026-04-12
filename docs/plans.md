# Plans

This document is the execution plan and ongoing notes for the current task. Keep it current as the work moves.

## Verification Checklist
- `test -f docs/prompt.md`
- `test -f docs/plans.md`
- `test -f docs/implement.md`
- `test -f docs/documentation.md`
- Review the new docs with `sed -n '1,220p' docs/*.md`
- Last verified: 2026-04-12

## Milestones
- `[done]` Milestone 1: inspect current repo state and history
  - Scope: repo guidance, workspace shape, product surfaces, and git progress history
  - Key areas: `README.md`, `AGENTS.md`, root workspace manifests, `apps/cli`, `apps/web`, `packages/*`, git log/tags
  - Acceptance criteria: current architecture and project arc are understood well enough to write durable memory without guessing
  - Verification steps: `git status --short --branch`, `git log --reverse --oneline --no-decorate`, targeted file reads
- `[done]` Milestone 2: create the durable-memory files
  - Scope: author the required `docs/` set for this prerequisite phase
  - Key areas: `docs/prompt.md`, `docs/plans.md`, `docs/implement.md`, `docs/documentation.md`
  - Acceptance criteria: each file is present, concise, and aligned with the current repo state
  - Verification steps: existence checks plus direct file review
- `[done]` Milestone 3: verify consistency
  - Scope: confirm the docs agree with the inspected repo state and task scope
  - Key areas: all new `docs/*.md`
  - Acceptance criteria: no missing file, stale placeholder text, or contradiction with the inspected code/history
  - Verification steps: re-read the files and compare against the gathered repo context

## Acceptance Checks
- The durable-memory set exists and is readable.
- The prompt describes this prerequisite task rather than a generic repo summary.
- The documentation records the current product, repo layout, release/history arc, and next-step posture.

## Validation
- `git status --short --branch`
- `git log --oneline --decorate --graph --all -n 80`
- `git log --reverse --oneline --no-decorate`
- `git rev-list --count --all`
- `git tag --sort=creatordate`
- `find apps packages -maxdepth 2 -type f | sort`
- targeted `sed -n` reads across root docs and key source files
- `test -f docs/prompt.md && test -f docs/plans.md && test -f docs/implement.md && test -f docs/documentation.md`

## Decisions
- Durable memory is initialized around the current prerequisite task: establish project baseline and context before future work.
- No product code changes are part of this pass.
- Future tasks in this repo should update these files first when scope changes materially.

## Implementation Notes
- No prior `docs/prompt.md`, `docs/plans.md`, `docs/implement.md`, or `docs/documentation.md` existed at the repo root before this pass.
- Current branch for this work: `docs/durable-memory-init`.
- Current visible release line: `v0.1.0` through `v0.1.8`.
- Current visible commit count across refs: 103.
- Recent arc after the initial build-out is mostly release hardening, web UI parity, and acceptance-loop stability.

## Risks
- Risk: the memory set becomes stale after the next task.
  - Mitigation: update `docs/prompt.md`, `docs/plans.md`, and `docs/documentation.md` before and during follow-on work.
- Risk: history summary hides nuance from individual fixes.
  - Mitigation: use git history directly when a later task depends on a specific regression or subsystem.

## Architecture
- Product: local-first AI switchboard that exposes an OpenAI-compatible local API at `http://127.0.0.1:4684/v1`.
- Runtime surfaces: Rust CLI, Rust TUI, Rust daemon/browser control plane, Next.js marketing/docs site, npm install wrapper.
- Core storage: local files under `~/.gunmetal`, with SQLite state, runtime files, helper directory, and logs.
- Provider model: explicit adapters for `codex`, `copilot`, `openrouter`, `zen`, and `openai`, with additional enum slots for `azure`, `nvidia`, and custom providers.
