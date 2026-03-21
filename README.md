# gunmetal

Local-first AI switchboard.

Connect providers you already use, create local API keys, point your apps at `http://127.0.0.1:4684/v1`.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

Install downloads the native `gunmetal` binary into `~/.gunmetal/bin/`.

## Quickstart

```bash
gunmetal setup
gunmetal start
gunmetal status
```

`gunmetal setup` is the golden path. It saves one profile, checks auth, syncs models, creates one key, and ends with a ready-to-run test command.

## Start Here

1. Install: `npm i -g @dhruv2mars/gunmetal`
2. Run `gunmetal setup`
3. Run `gunmetal start`
4. Call `GET /v1/models`
5. Call `POST /v1/chat/completions`

```bash
export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key

curl $OPENAI_BASE_URL/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

curl $OPENAI_BASE_URL/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "codex/gpt-5.4",
    "messages": [{"role":"user","content":"say ok"}]
  }'
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

Gunmetal works when the app talks to Gunmetal:

- app must let you set a custom base URL
- app must let you send a custom API key
- app must accept arbitrary model ids like `provider/model`
- if it hardcodes the upstream endpoint, Gunmetal cannot help there

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
