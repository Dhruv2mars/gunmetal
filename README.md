# gunmetal

Local-first AI access as one Local API.

Gunmetal turns your AI subscriptions and upstream provider access into a local API. Connect provider access you already use, create local Gunmetal keys, point your apps at `http://127.0.0.1:4684/v1`, choose provider-qualified model IDs, and inspect request history for debugging.

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

Install downloads the native `gunmetal` binary into `~/.gunmetal/bin/`.

## Quickstart

```bash
gunmetal setup
gunmetal web
gunmetal start
gunmetal status
```

`gunmetal setup` is the golden path. It creates one provider connection, checks auth, syncs models, creates one Gunmetal key, and ends with a ready-to-run Local API request.

`gunmetal web` opens the local Dashboard at `http://127.0.0.1:4684/webui`. `gunmetal start` keeps the local OpenAI-compatible API running at `http://127.0.0.1:4684/v1`.

## Start Here

1. Install: `npm i -g @dhruv2mars/gunmetal`
2. Run `gunmetal setup`
3. Run `gunmetal web` for the Dashboard, or `gunmetal start` for the API only
4. Open `http://127.0.0.1:4684/webui` if you want the Dashboard
5. Call `GET /v1/models`
6. Call `POST /v1/chat/completions`

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
| API Key  | your Gunmetal key               |
| Model    | `openai/gpt-5.1`, `codex/gpt-5.4`, etc. |

## Providers

| Connection type          | Upstream providers      |
| ------------------------ | ----------------------- |
| Subscription connection  | `codex`, `copilot`      |
| API-key connection       | `openrouter`, `zen`, `openai` |

## API

```
GET  /v1/models
POST /v1/chat/completions
POST /v1/responses
```

Streaming supported on both POST endpoints.

Gunmetal is a normalized Local API by default, with local request history and token usage built into the debugging path.

- normalized mode keeps one clean contract across providers
- passthrough mode is opt-in through `gunmetal.mode = "passthrough"` plus `provider_options`
- benchmarks should use normalized mode unless you explicitly want provider-native behavior

Gunmetal works when the app talks to Gunmetal:

- app must let you set a custom base URL
- app must let you send a custom Gunmetal key
- app must accept provider-qualified model ids like `provider/model`
- if it hardcodes the upstream endpoint, Gunmetal cannot help there

## Commands

```bash
gunmetal setup
gunmetal web
gunmetal start
gunmetal status
gunmetal profiles list
gunmetal auth status <provider>
gunmetal models sync <provider>
gunmetal keys list
gunmetal logs list
```

## Structure

```
apps/gunmetal/      # native CLI entrypoint
apps/web/           # landing page, docs
packages/sdk/       # Gunmetal Provider SDK
packages/sdk-core/  # shared SDK-facing types + contracts
packages/extensions/ # first-party provider integrations
packages/app-cli/   # CLI command layer
packages/app-daemon/ # local OpenAI-compatible API server
packages/app-storage/ # sqlite + local state
packages/npm/       # npm install wrapper for the native binary
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
