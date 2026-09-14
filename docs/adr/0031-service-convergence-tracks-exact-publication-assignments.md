# Service convergence tracks exact publication assignments

Status: accepted

Service identity, exact realm slot, and provider publication occurrence
are three distinct things. A Service is a Runtime-local named typed
value contract: within one Runtime, one name denotes one compatible
contract. An exact slot is the pair `(Service, ServiceRealm)`; the
isolate axis selects the realm, lookup addresses exactly the
Context-selected realm with no fallback or ancestry, and a slot has at
most one eligible current occurrence. A provider publication is one
successful publication occurrence by one Fiber generation into one
exact slot; exact occurrence identity — not Service identity — governs
visibility, replacement, stale-cleanup safety, mutation, removal, and
dependency assignment.

Actual current Active publications — never declared metadata —
determine visibility. A Loading generation may occupy a slot but is
invisible: lookup and dependency assignment treat the slot as Missing
until the publication reaches Active. Generation gate close withdraws
visibility before cleanup runs. Replacement is exact: a later
generation may occupy the slot over a closed occurrence, and stale
cleanup from the old generation cannot remove or shadow the
replacement, because each is a distinct occurrence. Only actual
visibility changes create dependency drift: declared provide metadata,
Loading installation, a same-occurrence payload `set`, stale cleanup,
and storage relocation change no target. Lookup requires no InjectSpec
membership, and an incompatible same-name contract reports mismatch
rather than absence.

Fibers own the authoritative dependency edges. At creation, each
required Service resolves exactly once through the spawning Context's
isolate mapping into a fixed era-local exact slot; the final InjectSpec
is one normalized required row per Service after Plugin declarations
and Loader overlay. These edges are immutable for the era: restart and
same-Fiber update preserve them, and era replacement resolves a fresh
set from the captured creation recipe. No store, index, or Registry
record is the authority for these relationships.

A Fiber settles and converges against its SemanticTarget: the
committed effective apply input plus every exact dependency slot, each
assigned `Missing` or one exact visible provider publication
occurrence. A `Missing` assignment makes the Fiber Pending; all
present makes it apply-eligible. Target equality is semantic: it
includes committed apply-relevant configuration and excludes store,
index, counter, ordering, wakeup, and value identity. Mutating the
payload of the same occurrence does not change the target; removing an
occurrence or providing a new one does. Failed-target parking follows
from this equality: a notification resolving to the same effective
input and publication assignments does not retry, a genuinely changed
assignment or input permits another pass, and an explicit restart may
bypass same-target parking.

Every effective visibility mutation atomically commits a durable
target recheck for the complete affected set — deduplicated per Fiber —
in the same semantic commit; the settlement kicks that follow run
outside locks. The race law is closed: edges fixed before the mutation
receive a durable recheck, edges fixed after it see the current state
in their initial settle, and unregistered or disposed dependents owe
no progress. Durable recheck is protocol state, not executor or
observer availability: a missing executor may skip a kick but cannot
lose drift, settlement release rechecks the live target before
publishing idle so provider/apply races converge without concurrent
duplicate apply, and a later `ready` drives any drift that remains.

DependencyIndex is replaceable acceleration, never semantic authority.
It is a derived projection of the Fiber-owned edges that accelerates
affected-dependent discovery; disabling it, rebuilding it, changing
its order, or replacing it with authoritative scanning changes cost
only — pending diagnostics, SemanticTargets, affected-set membership,
and settlement outcomes are unchanged.

## Rationale

Exact occurrence identity and durable recheck make the hard races
deterministic. Stale cleanup after replacement, failed-target parking,
off-runtime provider mutation, and provider/settle races all resolve
to closed rules because visibility changes commit their complete
affected set durably before any settlement runs, and because
dependents address occurrences rather than providers. This removes
provider stacks, snapshot-based dependency authority, and any need for
synchronous convergence under locks.

## Consequences

- Undeclared dynamic publication still creates drift for every Fiber
  whose fixed edges address its exact slot, while a declared
  provide that never reaches Active visibility creates none.
- A multi-slot visibility batch deduplicates each affected Fiber into
  one recheck.
- A dependent whose own re-apply fails parks Failed against its
  current SemanticTarget as its own business; it is never a corrupting
  state for the provider mutation that notified it.
- Rebuilding, reordering, or deleting DependencyIndex may change cost
  only; any behavior change from doing so is a bug in the derived
  projection, not in the authoritative edges.

## Non-normative lineage

| V3 rule | Lineage classification | Historical evidence only |
| --- | --- | --- |
| Named Service contract, exact realm placement, and actual-publication authority | supersedes part of an existing ADR | ADRs 0003, 0008, and 0022 |
| Exact publication identity, gated visibility, and atomic mutation-to-drift commit | collapses several historical ADRs into one final rule | ADRs 0010, 0015, and 0022 |
| Era-local dependency edges and publication-based SemanticTarget | supersedes part of an existing ADR | ADR 0013 |
| DependencyIndex as derived acceleration rather than semantic authority | supersedes part of an existing ADR | ADRs 0016 and 0024 |
