# Gunmetal



Gunmetal lets you connect provider access you already have, create local Gunmetal API keys, and point OpenAI-compatible apps at `http://127.0.0.1:4684/v1`. It is built for local development and personal workflows: provider auth, model sync, key management, playground testing, and request history all stay on your machine.

> Alpha software: Gunmetal is public and usable, but the product is still early. Commands, provider behavior, SDK contracts, and dashboard flows can change while the project stabilizes. Do not treat it as a production security boundary yet.

- Website: https://web-nine-sigma-59.vercel.app
- npm package: https://www.npmjs.com/package/@dhruv2mars/gunmetal
- Latest alpha release: https://github.com/Dhruv2mars/gunmetal/releases/tag/v0.1.15

## Install

```bash
npm i -g @dhruv2mars/gunmetal
```

The npm package installs the native `gunmetal` binary into `~/.gunmetal/bin/`.

Current release: `@dhruv2mars/gunmetal@0.1.15` / `v0.1.15` alpha. Native binaries are published for macOS, Linux, and Windows on x64 and arm64.

## Quickstart

```bash
gunmetal setup
gunmetal start
gunmetal status
```

`gunmetal setup` walks the first provider connection, checks auth, syncs models, creates one Gunmetal key, and prints a ready-to-run local API request.

`gunmetal start` starts the local daemon, opens the Dashboard at `http://127.0.0.1:4684/`, and serves the OpenAI-compatible API at `http://127.0.0.1:4684/v1`. Use `gunmetal start --no-open` when you want the daemon without opening a browser.

## Dashboard Workflow

1. Install the CLI.
2. Run `gunmetal start`.
3. Open the Dashboard if it did not open automatically.
4. Enable the providers you want to use.
5. Authenticate browser-session providers such as Codex and Copilot, or save upstream keys for API-key providers.
6. Sync models for each ready provider.
7. Create a Gunmetal API key.
8. Paste that key into the Playground, choose a synced provider/model ID, and send a test prompt.
9. Use the same base URL and Gunmetal key in any OpenAI-compatible app.

## Client Config

```bash
export OPENAI_BASE_URL=http://127.0.0.1:4684/v1
export OPENAI_API_KEY=gm_your_local_key
```

```bash
curl "$OPENAI_BASE_URL/models" \
  -H "Authorization: Bearer $OPENAI_API_KEY"

curl "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "provider/model",
    "messages": [{"role":"user","content":"say ok"}]
  }'
```

| Setting | Value |
| --- | --- |
| Base URL | `http://127.0.0.1:4684/v1` |
| API key | a local Gunmetal key, usually `gm_...` |
| Model | a provider-qualified model ID such as `codex/...`, `copilot/...`, `openrouter/...`, `openai/...`, or `zen/...` |

Gunmetal works when the app lets you set a custom OpenAI-compatible base URL, pass your own API key, and choose arbitrary provider-qualified model IDs. If an app hardcodes the upstream provider endpoint, Gunmetal cannot route it.

## Providers

| Connection type | Providers |
| --- | --- |
| Browser-session providers | `codex`, `copilot` |
| API-key providers | `openrouter`, `zen`, `openai` |

Browser-session providers require the local auth flow to finish before models can sync. API-key providers require an upstream key or compatible base URL credentials.

## API

```text
GET  /v1/models
POST /v1/chat/completions
POST /v1/responses
```

Streaming is supported on both POST endpoints. Gunmetal defaults to normalized behavior across providers, with passthrough available per request through `gunmetal.mode = "passthrough"` and `provider_options` when provider-native behavior is needed.

## CLI

```bash
gunmetal setup
gunmetal start
gunmetal start --no-open
gunmetal status
gunmetal doctor
gunmetal chat
gunmetal profiles list
gunmetal auth status <provider>
gunmetal models sync <provider>
gunmetal keys list
gunmetal logs list
gunmetal logs summary
```

## Gunmetal Provider SDK

Developers can build provider integrations against the same contracts Gunmetal uses internally.

```bash
cargo add gunmetal-sdk gunmetal-core gunmetal-storage
```

Use `gunmetal-sdk` for `ProviderAdapter`, `ProviderRegistry`, `ProviderHub`, streaming helpers, and model enrichment. Use `gunmetal-core` for shared request, response, provider, key, profile, and model types. Use `gunmetal-storage` for local `AppPaths` and storage handles when embedding the hub in a local application.

First-party adapters live in `gunmetal-providers`:

```bash
cargo add gunmetal-providers
```

It exposes `builtin_registry()`, `builtin_provider_hub()`, and concrete clients for Codex, Copilot, OpenRouter, Zen, and OpenAI.

## Repository

```text
apps/gunmetal/        native CLI entrypoint
apps/web/             hosted landing site and docs
packages/app-cli/     CLI command layer
packages/app-daemon/  local Dashboard and OpenAI-compatible API server
packages/app-storage/ SQLite and local state
packages/sdk-core/    shared SDK-facing types and contracts
packages/sdk/         provider SDK
packages/extensions/  first-party provider adapters
packages/npm/         npm install wrapper for the native binary
```

## Development

```bash
bun install
bun run dev
bun run test
bun run check
bun run build
cargo run -p gunmetal -- --help
```

## Release

Public releases are tagged as `vX.Y.Z`. The release workflow builds native binaries for GitHub Releases, publishes the npm CLI package, and publishes the public Rust SDK crates in dependency order. Gunmetal is alpha, so all current `v0.1.x` GitHub releases are marked as prereleases.

The current release is `v0.1.15` alpha:

- GitHub Release: https://github.com/Dhruv2mars/gunmetal/releases/tag/v0.1.15
- npm: `npm i -g @dhruv2mars/gunmetal@0.1.15`
- Binaries: `gunmetal-darwin-arm64`, `gunmetal-darwin-x64`, `gunmetal-linux-arm64`, `gunmetal-linux-x64`, `gunmetal-win32-arm64.exe`, `gunmetal-win32-x64.exe`
- Checksums: published beside the binaries in the GitHub Release

## License

MIT
