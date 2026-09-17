# WASM component examples

This directory will mirror the native `examples/` suite with guest Components
running through the `cordis:plugin` WIT runtime. It is deliberately separate:
the native examples remain the reference for `cordis-core`, while these prove
the guest ABI and its host adapter.

The current WIT world intentionally exposes only lifecycle plus host-owned
diagnostics. No native example can yet be ported without moving its interesting
work back into the host. That would prove neither WIT nor the Component Model.

## Porting order

| Native example | Guest ABI required before it can be ported |
| --- | --- |
| `hello_plugin` | Event subscription and emission |
| `scopes_tenants` | Service publication/lookup and scoped Event routing |
| `logging_exporters` | Logging and runtime observation |
| `worker_daemon` | Services, owned tasks, effects, and timers |
| `gateway` | Events, Services, timers, Loader input, and update control |
| `chat_capstone` | Events, Services, scoped routing, and update/era replacement |

## Rules

- Each guest is a `wasm32-wasip2` Component and has no direct access to
  `cordis-core`.
- All host authority remains in `cordis-component`; guest imports are narrow,
  capability-oriented WIT interfaces.
- A port must demonstrate its behavior from guest code. A native host-side
  reimplementation is not a port.
- Every guest instance still maps to exactly one Cordis apply generation.

`diagnostics` is the only current capability: the guest emits through its
owning Fiber's host-assigned logger channel. Event support is a later ABI
increment, after the lifecycle spike has passed its replacement proof.
