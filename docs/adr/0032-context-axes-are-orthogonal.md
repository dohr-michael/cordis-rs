# Context has orthogonal isolate, Scope, and intercept axes

Status: accepted

A Context is a cheap immutable view into one Runtime, selecting a
current Fiber together with three independent axes. There is no
general Context hierarchy: two views with the same Runtime, Fiber,
and semantically equal axis positions behave identically regardless
of derivation allocation or history, and no subsystem infers
ownership, authorization, Service fallback, Event routing, or
lifecycle meaning from derivation ancestry. A Context's derivation
lineage carries no meaning beyond the explicit axes.

The three axes are orthogonal:

- **isolate** maps Service identities to exact service realms. A fresh
  private realm separates the selected Service; an explicit equal
  realm joins only that exact slot. Isolation neither renames a
  Service nor introduces fallback, Service inheritance, Event
  reachability, or Context ancestry.
- **Scope** alone defines rooted Event reachability. It is an opaque
  Runtime-local routing capability containing only a Runtime
  association and one Event reachability position; it exposes no
  ancestry or Context capabilities. Scoped dispatch reaches
  ancestor-or-self registrations plus global registrations; siblings
  and descendants are excluded, and `Scoped(root)` differs from
  `Unscoped`. Scope implies no lifecycle, Service, Fiber-parenthood,
  authorization, or Context relation.
- **intercept** carries ordered Service-owned configuration. It is
  immutable; matching prepared layers compose outer-to-inner under
  the relevant Service contract. Intercept is not Event middleware,
  authorization, routing, or Service placement.

Each primitive derivation changes exactly one axis and never starts
lifecycle work: `with_isolated_service` and `with_service_realms`
change isolate only, `with_child_scope` changes Scope only, and
`with_intercept` appends a new innermost intercept layer only. An
isolate derivation leaves listener reachability and intercept layers
untouched; a Scope derivation leaves Service placement untouched; an
intercept derivation leaves both untouched.

Spawn origin is explicit provenance — the Context, and therefore the
originating Fiber and view, from which a spawn was initiated. It is
recorded and captured so that era replacement can replay creation
semantics, and it implies none of the three axis relations: no
parenthood or teardown responsibility, no Scope ancestry authority,
no Event ancestry, no continuing isolate or intercept coupling, and
no lifecycle ownership. The spawn derivation takes its initial axis
positions from the spawning Context exactly as the derivation matrix
specifies; the stored origin itself grants nothing further.

## Derivation matrix

Root, spawn, restart, update, and era replacement follow this complete
matrix; no operation acquires an axis change it does not name.

| Operation | Fiber | isolate | Scope | intercept / apply input |
| --- | --- | --- | --- | --- |
| `Context::new` | permanent root Fiber | Runtime default mapping | root Scope | empty/root intercept |
| primitive derivation | unchanged | only isolate operation changes it | only Scope operation changes it | only intercept operation changes it |
| root derivation | permanent root Fiber | reset to Runtime default | reset to root Scope | reset to empty/root |
| new-Fiber spawn | fresh Fiber | inherit spawning Context | fresh child of spawning Scope | inherit Context intercept; compute fresh effective input |
| restart | same Fiber | unchanged | unchanged | reapply current committed input; preserve resolved edges |
| same-Fiber update | same Fiber | unchanged | unchanged | change only explicitly accepted committed input; no implicit retarget |
| era swap | fresh Fiber | inherit captured spawn origin and resolve anew | fresh child of captured origin Scope | reconstruct captured creation recipe and effective spec |

Root derivation stays in the same Runtime and discards caller-local
axes. Restart and update preserve Fiber Scope. Era swap replays
creation semantics: the successor receives fresh resolved edges,
target state, Scope, generations, and resource publications. It
inherits no provider publication, listener registration, effect
journal, or old Fiber Scope.

Old and successor Fiber Scopes are sibling-era Scopes beneath the
captured spawn-origin Scope. Surviving old-scope descendants are not
migrated to the successor. An old Scope node may remain referenced
after its Fiber dies; that does not preserve the Fiber or create a
stable event seat.

## Rationale

Orthogonal axes remove accidental cross-axis inheritance: a derivation
performed for one purpose can never silently acquire Service, Event,
ownership, or authorization meaning from another axis. Fixing every
multi-axis operation in one matrix keeps root, spawn, restart, update,
and era replacement honest about exactly which positions they reset,
preserve, or recompute, and separating spawn origin from all three
axes prevents provenance storage from growing into a hidden hierarchy.

## Consequences

- Scoped dispatch from the successor's sibling-era Scope never reaches
  listeners registered under the old era's Scope, and vice versa:
  siblings are excluded from rooted reachability.
- Because restart and update preserve Scope and resolved edges, a
  realm derivation performed after spawn never retargets the Fiber.
- A listener's callback Context attribution comes from registration,
  not from the emitter's routing position; Scope controls eligibility,
  not attribution.
- Derivation-history inspection is not a v3 behavior: equivalence of
  views is observable only through the axes themselves.

## Non-normative lineage

| V3 rule | Lineage classification | Historical evidence only |
| --- | --- | --- |
| No general Context hierarchy; isolate, Scope, and intercept are orthogonal | supersedes part of an existing ADR | ADR 0008 |
| Rooted Scope reachability and one Routing relation | collapses several historical ADRs into one final rule | ADRs 0003 and 0006 |
| Spawn origin is explicit provenance and implies none of the three axis relations | introduces a genuinely new decision | Earlier storage retained the originating Context without defining this separation |
