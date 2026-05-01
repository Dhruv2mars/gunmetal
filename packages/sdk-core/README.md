# gunmetal-core

Shared public types for Gunmetal provider integrations.

This crate contains the stable data contracts used by `gunmetal-sdk`, `gunmetal-storage`, `gunmetal-providers`, the CLI, and the local daemon:

- provider identifiers with `ProviderKind`
- Gunmetal key records and scopes
- provider profiles and auth status
- model descriptors and model metadata
- chat completion requests and responses
- token usage and request mode options

Install:

```bash
cargo add gunmetal-core
```

Most SDK users should pair this crate with `gunmetal-sdk`:

```bash
cargo add gunmetal-sdk gunmetal-core gunmetal-storage
```

Use `gunmetal-core` when your integration needs to construct or inspect request/response values without depending on the provider hub implementation.
