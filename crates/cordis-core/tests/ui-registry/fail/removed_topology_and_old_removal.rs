use cordis_core::{Context, Plugin};
use std::convert::Infallible;

struct P;
impl Plugin for P {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;
    fn prepare(&self, (): ()) -> Result<(), Infallible> { Ok(()) }
    async fn apply(&self, _ctx: Context, _prepared: &()) -> Result<(), Infallible> { Ok(()) }
}

fn main() {
    let ctx = Context::new();
    let _ = ctx.live_fiber_count();
    let _ = ctx.pending_missing();
    let _ = ctx.registry();
    let _ = ctx.registry_remove::<P>();
    let _ = ctx.remove_plugin::<P>();
}
