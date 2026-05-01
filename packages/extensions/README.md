# gunmetal-providers

First-party provider adapters for Gunmetal.

Install:

```bash
cargo add gunmetal-providers
```

This crate exports:

- `builtin_registry()` for a `ProviderRegistry` with all bundled providers
- `builtin_provider_hub(paths)` for a ready-to-use `ProviderHub`
- `builtin_providers()` for provider metadata
- concrete clients for Codex, Copilot, OpenRouter, Zen, and OpenAI
- SDK re-exports from `gunmetal-sdk`

Typical embedding:

```rust
use gunmetal_providers::builtin_provider_hub;
use gunmetal_storage::AppPaths;

let paths = AppPaths::resolve()?;
let hub = builtin_provider_hub(paths);
let providers = hub.definitions();
```

Use this crate when you want Gunmetal's built-in upstream integrations. Use `gunmetal-sdk` directly when building your own provider adapter.
