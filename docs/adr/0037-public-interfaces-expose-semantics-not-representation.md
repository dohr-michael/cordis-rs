# Public interfaces expose semantics, not representation

Status: accepted

Every public path names a consumer-meaningful semantic family. Rust
modules are private by default; each crate exposes one curated
semantic facade in which every reachable item has exactly one
canonical path. No public module mirrors source layout, and no item
is reachable through a second spelling, a glob re-export, a broad
prelude, or a deprecated alias.

The facades expose semantics:

- **Complete operations.** Each operation performs its whole
  semantic transaction; a caller never assembles protocol phases,
  claim machinery, or continuation steps to reach a correct outcome.
- **Typed preparation.** Source adaptation is typed and completes
  before lifecycle admission: preparation produces typed Plugin
  input, and sealing proves the Plugin contract association
  before any lifecycle transaction begins. Raw `Any` configuration,
  downcasts, default-config conveniences, and erased-builder
  protocols never enter application code.
- **Opaque capabilities and identities.** Control over one exact
  occurrence — a provider publication, a listener registration, a
  cleanup obligation — moves through move-only capabilities whose
  Drop is inert. Correlation identities such as FiberId are opaque
  Runtime-local facts: not numeric, not constructible, and never
  convertible into lookup or control capabilities.
- **Immutable observations.** Snapshots and observation records are
  immutable postcommit projections correlated by opaque identities;
  they carry no callbacks, stores, topology, or operational
  authority.
- **Operation-owned errors.** Each operation family owns its error
  type with its meaningful phases, normalized once at its boundary.
  There are no global error codes, no required downcasting, and
  error plumbing imposes no accidental Clone or Sync bounds.

Representation stays out. Storage topology, identity representation,
type-erasure adapters, claim state, continuation machinery, and
partial transaction phases remain private; compatibility aliases and
second canonical paths are absent, so a removed v2 path fails to
compile rather than silently continuing through an alias or a
storage escape hatch.

The exact declarations — exhaustive names, signatures, bounds, and
module paths — live only in the public-interface inventory. This ADR
records the policy those declarations follow; it is not a second
home for them.

## Rationale

Consumers learn stable meaning rather than reconstructing protocol
correctness or coupling to Rust representation: an interface that
publishes storage or claim machinery obliges every caller to
reassemble correctness, and one that publishes representation
couples every consumer to layout the owners must stay free to
change. Typed preparation makes source adaptation, contract
association, and lifecycle admission distinct failure and ownership
seams (with the public input vocabulary refined by ADR 0038), so a source or
configuration failure cannot create partial
Runtime state. Operation-owned errors let downstream consumers match
meaningful phases without global codes or downcasts.

## Consequences

- Migration cannot silently continue: removed paths and
  representations fail to compile.
- Moving an item is a deliberate break, never an alias; one
  canonical path per item keeps consumer code and documentation
  unambiguous.
- Opaque identities correlate observations but never unlock
  operations; control requires the capability issued by the
  operation that created the occurrence.
- Preparation and apply errors carry only the bounds their boundary
  actually requires; no universal Default, Clone, Send, or Sync
  bound leaks into consumer contracts.

## Non-normative lineage

| V3 rule | Lineage classification | Historical evidence only |
| --- | --- | --- |
| Default-private curated semantic facades | introduces a genuinely new decision | Earlier ADRs exposed or preserved source-layout-shaped public paths |
| Typed preparation and sealing before lifecycle admission | introduces a genuinely new decision | Earlier ADRs did not separate source adaptation, typed association, and lifecycle commit |
| Opaque exact-occurrence control and topology-free observation | collapses several historical ADRs into one final rule | ADRs 0005, 0007, 0010, 0014, 0015, and 0022 |
| Operation-specific errors and one-time normalization | collapses several historical ADRs into one final rule | Earlier decisions distributed error and containment rules by mechanism |
| Semantic public-interface policy with exact declarations moved to the interface inventory | supersedes part of an existing ADR | ADR 0027 |
