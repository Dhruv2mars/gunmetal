# gunmetal

Local-first AI switchboard.

Gunmetal lets you connect providers you already use, create Gunmetal-local API keys, and point your apps at one local API on `http://127.0.0.1:4684/v1`.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

The npm package downloads the native `gunmetal` binary into `~/.gunmetal/bin/` on first run.

## Quickstart

```bash
gunmetal setup
gunmetal start
```

What happens in setup:

1. connect a provider
2. sync models
3. create a Gunmetal key
4. print the base URL and first model

Then point any compatible app at:

- base URL: `http://127.0.0.1:4684/v1`
- API key: your Gunmetal key
- model: a provider-prefixed model like `openai/gpt-5.1` or `codex/gpt-5.4`

## Commands

```bash
gunmetal
gunmetal setup
gunmetal start
gunmetal status
gunmetal profiles list
gunmetal auth status <profile-name>
gunmetal models sync <profile-name>
gunmetal keys list
gunmetal logs list
```

## What Works

- local daemon, CLI, and TUI in one product
- explicit provider/model routing
- Gunmetal-local API keys
- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/responses`
- streaming on both API paths
- subscription providers: `codex`, `copilot`
- gateway providers: `openrouter`, `zen`
- direct-key providers: `openai`

## Monorepo

- `apps/site`: landing page, docs, install, changelog
- `packages/cli`: npm launcher and installer
- `crates/gunmetal-app`: native entry binary
- `crates/gunmetal-daemon`: local HTTP service
- `crates/gunmetal-cli`: commands and setup flow
- `crates/gunmetal-tui`: terminal dashboard
- `crates/gunmetal-storage`: local sqlite state
- `crates/gunmetal-providers`: provider adapters

## Local Dev

```bash
bun install
bun run test
bun run check
cargo run -p gunmetal-app -- --help
```
