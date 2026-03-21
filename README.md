# gunmetal

Local-first AI switchboard.

Connect providers you already use, create local API keys, point your apps at `http://127.0.0.1:4684/v1`.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

## Quickstart

```bash
gunmetal setup   # connect provider, sync models, create key
gunmetal start   # start daemon
```

Then configure any OpenAI-compatible app:

| Setting  | Value                           |
| -------- | ------------------------------- |
| Base URL | `http://127.0.0.1:4684/v1`      |
| API Key  | your gunmetal key               |
| Model    | `openai/gpt-5.1`, `codex/gpt-5.4`, etc. |

## Providers

| Type         | Providers              |
| ------------ | ---------------------- |
| Subscription | `codex`, `copilot`     |
| Gateway      | `openrouter`, `zen`    |
| Direct       | `openai`               |

## API

```
GET  /v1/models
POST /v1/chat/completions
POST /v1/responses
```

Streaming supported on both POST endpoints.

## Commands

```bash
gunmetal setup
gunmetal start
gunmetal status
gunmetal profiles list
gunmetal auth status <profile>
gunmetal models sync <profile>
gunmetal keys list
gunmetal logs list
```

## Structure

```
apps/web/       # landing page, docs
packages/cli/   # npm wrapper
crates/         # rust workspace
  gunmetal-app/       # binary entry
  gunmetal-cli/       # commands
  gunmetal-core/      # shared types
  gunmetal-daemon/    # http server
  gunmetal-providers/ # provider adapters
  gunmetal-storage/   # sqlite
  gunmetal-tui/       # terminal ui
```

## Development

```bash
bun install
bun run dev      # start web dev server
bun run test     # all tests
bun run check    # lint + fmt + clippy
cargo run -p gunmetal-app -- --help
```

## License

MIT
