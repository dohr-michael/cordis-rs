# Migrating from cordis-rs 0.6.x to v3

`cordis-rs 0.7` is an architectural replacement, not a source-compatible update of
`0.6.x`. The old implementation remains maintained on `legacy/0.6` for critical
bug and security fixes.

## Dependency identity

Applications may keep the historical package and Rust import names:

```toml
cordis-rs = "0.7"
```

```rust
use cordis::Context;
```

The `cordis-rs` package is now a thin facade over `cordis-core = "0.1"`. Framework
and plugin crates should normally depend on `cordis-core` directly. Timer and
loader capabilities are explicit optional crates rather than facade features.

## Architectural changes

The migration is governed by the accepted v3 ADRs in `docs/adr/0028` through
`0038`. The most visible changes are:

- Context no longer implies one hidden hierarchy: Service isolation, Event Scope,
  and intercept are orthogonal axes.
- Service dependencies converge on exact `(Service, ServiceRealm)` publication
  assignments; there is no implicit fallback lookup.
- Events use typed contracts, explicit routing, semantic listener roles, and
  completion-aware occurrence claiming.
- Generation cleanup ownership is distinct from Runtime/Registry Fiber residency.
- update preserves Fiber identity; era swap is an identity-breaking replacement.
- update control is a precommit lifecycle protocol, not an ordinary dispatchable Event.
- runtime observation follows committed protocol truth and never drives it.
- public modules and crate seams expose semantic roles rather than internal stores,
  registry topology, or other representation details.

## Legacy companion crates

The legacy `cordis-include`, `cordis-group`, and `cordis-cli` crates are not
mechanically carried into v3. They remain part of the `0.6.x` ecosystem until a
v3-native design is justified by the new semantic architecture.

For the exhaustive migration inventory, see `docs/v3-migration.md`. For the exact
v3 public surface, see `docs/v3-public-interface.md`.
