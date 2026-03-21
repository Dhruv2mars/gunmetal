# gunmetal

Local-first AI switchboard.

Connect providers you already use, create local API keys, point your apps at `http://127.0.0.1:4684/v1`.

## Install

The npm wrapper exists in-repo, but it is not published yet.

For now, run Gunmetal from source:

```bash
bun install
cargo run -p gunmetal --
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

Gunmetal is a normalized gateway by default.

- normalized mode keeps one clean contract across providers
- passthrough mode is opt-in through `gunmetal.mode = "passthrough"` plus `provider_options`
- benchmarks should use normalized mode unless you explicitly want provider-native behavior

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
apps/cli/       # native CLI/TUI entrypoint
apps/web/       # landing page, docs
packages/cli/   # CLI command layer
packages/core/  # shared types + contracts
packages/daemon/# local OpenAI-compatible API server
packages/npm/   # npm install wrapper for the native binary
packages/providers/ # upstream provider adapters
packages/storage/   # sqlite + local state
packages/tui/       # terminal UI
```

## Development

```bash
bun install
bun run dev      # start web dev server
bun run test     # repo structure + all tests
bun run check    # lint + fmt + clippy
cargo run -p gunmetal -- --help
```

## License

MIT
