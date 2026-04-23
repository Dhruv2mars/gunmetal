# Documentation

Updated continuously during the UX pass.

## What This Work Is
- First-principles UX/frontend pass for the refactored Gunmetal product.
- Active surfaces are:
  - hosted web site in `apps/web`
  - local browser Web UI served from `packages/app-daemon`
  - CLI in `packages/app-cli`

## Product Thesis
- Gunmetal is a local-first inference middle layer for individuals.
- It turns provider accounts and AI subscriptions into one OpenAI-compatible local API.
- Canonical flow:
  - `app/tool -> Gunmetal key -> Gunmetal local API -> provider extension -> upstream provider`

## Current UX Model
- First screen should answer:
  - what Gunmetal is
  - how to install it
  - where the local API lives
  - what command gets the browser UI open
  - what to do when setup is incomplete
- Local browser Web UI should prioritize:
  - setup readiness
  - provider auth/sync
  - key creation
  - playground test
  - request history and recovery details
- CLI should prioritize:
  - `gunmetal setup`
  - `gunmetal web`
  - `gunmetal doctor`
  - `gunmetal chat`
  - `gunmetal logs summary`

## Current Status
- TUI is out of scope and should not be referenced as an active public surface.
- Hosted truth remains `gunmetalapp.vercel.app`.
- Public Web UI marketing route remains `/webui`.
- Local browser UI remains `http://127.0.0.1:4684/app`.
- Local API remains `http://127.0.0.1:4684/v1`.

## Validation Results
- Current UX pass validation pending.
