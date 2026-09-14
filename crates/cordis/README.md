# cordis-rs

Application-facing facade for Cordis v3.

```toml
[dependencies]
cordis-rs = "0.7"
```

The package remains `cordis-rs` and its Rust library crate remains `cordis`:

```rust
use cordis::{Context, Plugin};
```

The runtime contract is implemented by [`cordis-core`](https://crates.io/crates/cordis-core).
Framework and plugin authors that need the semantic runtime boundary may depend on
`cordis-core = "0.1"` directly.

See the repository README for the full v3 model and migration guidance.
