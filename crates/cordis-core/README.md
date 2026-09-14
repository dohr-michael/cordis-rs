# cordis-core

The semantic runtime contract for Cordis v3: Context, Plugin/Fork lifecycle, exact Service placement, typed Events, generation-owned effects, logging, and runtime observation.

```toml
[dependencies]
cordis-core = "0.1"
```

Application authors who want to preserve the historical `use cordis::...` import can depend on `cordis-rs = "0.7"` instead. Framework and plugin authors should normally depend on `cordis-core` directly.

See the repository README and `docs/v3-architecture.md` for the complete model.
