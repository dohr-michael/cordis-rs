# cordis-timer

Generation-owned time operations for Cordis v3. The crate provides sleep, interval, and timeout shapes without putting Tokio time policy into `cordis-core`.

```toml
[dependencies]
cordis-rs = "0.7"
cordis-timer = "0.1"
```

Use `cordis_timer::TimerExt` alongside a Cordis Context.
