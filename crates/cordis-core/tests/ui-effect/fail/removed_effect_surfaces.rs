// The removed v2 effect surface: labeled registration, the cloneable
// guard, and the introspection seam.

use cordis_core::{Context, EffectGuard, EffectMeta};

fn main() {
    let ctx = Context::new();
    let guard = ctx.effect_sync("db-handle", || {}).unwrap();
    let guard2 = guard.clone();
    guard2.remove();
    let _labels: Vec<EffectMeta> = ctx.effects();
    let _ = guard;
}
