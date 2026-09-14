# Plugin input names role; Prepared wrappers name stage

Status: accepted

The Plugin associated value consumed by `apply()` is named `Plugin::Input`, not
`Plugin::Prepared`: `Input` states the value's stable role while `prepare()`
continues to name the source-adaptation operation that produces it before
lifecycle admission. `PreparedPlugin` and `PreparedChange` keep their names
because those move-only wrappers describe a prepared, not-yet-applied stage;
their constructors are `from_input`. We rejected `RuntimeInput` as broader than
the Plugin contract and `BoundPlugin` because "bound" is ambiguous beside
Cordis dependency, Scope, realm, and Runtime relationships. `from_input<P>`
proves only the `P`/`P::Input` type association; it does not encode which
particular Plugin object produced the value through `prepare()`.
