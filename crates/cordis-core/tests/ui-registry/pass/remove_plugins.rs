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

fn assert_surface(ctx: &Context) {
    let future: _ = ctx.remove_plugins::<P>();
    drop(future);
}

fn main() { assert_surface(&Context::new()); }
