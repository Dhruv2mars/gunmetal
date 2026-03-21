# gunmetal

Local-first AI switchboard.

## Current shape

- `apps/site`: landing page, docs, install, changelog
- `crates/gunmetal-app`: main binary entry
- `crates/gunmetal-daemon`: local HTTP service
- `crates/gunmetal-cli`: CLI commands
- `crates/gunmetal-tui`: default terminal UI
- `crates/gunmetal-storage`: local sqlite state
- `crates/gunmetal-providers`: built-in provider catalog

## Local commands

```bash
bun install
bun run test
bun run check
cargo run -p gunmetal-app -- providers list
cargo run -p gunmetal-app -- keys create --name default --scope inference,models_read --provider codex
cargo run -p gunmetal-app -- serve --host 127.0.0.1 --port 4684
```

## Status

Implemented now:

- monorepo foundation
- local app paths and sqlite state
- Gunmetal key lifecycle
- provider profile storage
- model registry storage
- request metadata logging
- daemon health endpoint
- daemon models endpoint
- chat-completions request validation skeleton
- CLI shell
- TUI shell
