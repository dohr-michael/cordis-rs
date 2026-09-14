# Runtime observation follows protocol truth and never drives it

Status: accepted

Protocol truth commits first. Lifecycle transitions, Service visibility,
dependency drift, waiter progress, and operation completion all commit
through durable framework state and reliable signals; authoritative
state, visibility, registration, and dispatch commits — and their durable
wake and progress bookkeeping — complete before any observation request
exists. Deleting every observer leaves lifecycle, settlement, visibility,
cleanup, and completion outcomes identical, and a missing executor or a
failed observer can never lose drift or stall a waiter.

Only afterward may immutable observations be delivered through one
framework-published, consumer-subscribed, detached best-effort stream.
`Context::observe_runtime` accepts only the sealed `RuntimeObserver`
capability implemented for the Observer adapters, under a fixed policy:
repeatable, Runtime-wide, detached, unscoped, parallel, attempt-all. The
registering Context supplies callback attribution and generation
ownership, so disposing that generation stops future delivery. There is
no public observation Event marker and no emit/query/waterfall operation
over the stream; delivery recursively suppresses narration of its own
private-purpose dispatch — by private purpose, not Event name — so
consumers cannot recursively emit the stream, and source-operation
completion never waits for observer work.

The stream carries exactly five immutable record categories: Fiber
residency admitted/removed with a Fiber snapshot, Fiber state
previous/current, Service visibility previous/current exact publications,
listener registration/unregistration metadata, and primitive dispatch
completion with operation, Event name, routing, and outcome kind. Records
contain correlation facts — opaque Runtime-local ids such as `ScopeId`
and `ObservationRouting` — never callbacks, values, stores, topology, or
control capabilities. `EventOperation` and `DispatchOutcomeKind` are
semantic enums with no display or wire-name contract, and derived
`waterfall_query` narrates its constituent waterfall and query
primitives.

Observation has no veto, rollback, progress, completion, ordering,
retention, audit, or replay authority. Observer error or panic is
contained and reported diagnostically; it never fails, vetoes, rolls
back, wakes, or completes the source operation, and later observers are
still attempted. Facts that are not protocol transitions are not
observation categories: Loading installation alone emits no
ServiceVisibility record, restart has no residency change, and
apply-success birth, raw provided values, listener callbacks, generic
get/set, and update-control dispatch are never observed.

`Context::runtime_snapshot() -> RuntimeSnapshot` is the complementary
read-only seam: a flat current view holding exactly one active Root Fiber
record with empty missing Services plus every resident ordinary Fiber —
including a Disposed Fiber before unlink — and one Service record per
current occupied publication, including Loading/invisible ones. Each
record is self-consistent, but the collections have no semantic order,
are not a globally linearizable instant, and need not be referentially
closed. Neither seam supplies durable history, total order, replay, or
operational authority; opaque ids correlate records, nothing more.

## Rationale

Diagnostics stay useful without turning user callbacks or executor
availability into correctness dependencies: because every commit and
every reliable signal precedes observation, the protocol never owes an
observer anything, and observers can be slow, absent, failing, or
executor-starved without consequence. Retention, audit, and replay are
deliberately excluded because they are the properties of a different
durable-journal module, not of a best-effort stream.

## Falsifier

This decision falls if a concrete consumer requires reliable retained
audit/replay of Runtime transitions: such a consumer cannot be served by
the best-effort stream and forces the separate durable-journal module
this ADR assigns elsewhere.

## Consequences

- A failing observer leaves the observed dispatch, lifecycle, or Service
  result successful, with later observers still attempted.
- A dropped-observation gap is recovered by taking a later snapshot, not
  by replaying the stream.
- Loading a provider is invisible to the stream until its actual
  Active/withdraw transition; snapshot consumers see the occupied slot
  with `visible() == false` meanwhile.
- The v2 public internal observation Events and Registry-shaped snapshot
  surfaces have no v3 behavior; correlation across snapshot and stream
  uses opaque ids only.

## Non-normative lineage

| V3 rule | Lineage classification | Historical evidence only |
| --- | --- | --- |
| Protocol-first immutable Runtime observation | introduces a genuinely new decision | Historical internal hooks mixed observation and control |
| Reliable protocol signaling separated from optional callbacks | collapses several historical ADRs into one final rule | ADRs 0004, 0010, and 0013 |
