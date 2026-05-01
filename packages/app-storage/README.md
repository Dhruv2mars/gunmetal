# gunmetal-storage

Local state and path helpers for Gunmetal SDK integrations.

This crate provides:

- `AppPaths` for resolving Gunmetal home paths
- `StorageHandle` for opening local state
- local records for provider connections, synced models, Gunmetal keys, and request logs

Install:

```bash
cargo add gunmetal-storage
```

Typical SDK embedding:

```rust
use gunmetal_storage::AppPaths;

let paths = AppPaths::resolve()?;
let storage = paths.storage_handle()?;
```

`gunmetal-sdk::ProviderHub` accepts `AppPaths`, so host applications can share one local state layout with the CLI and local daemon.
