use cordis_core::{Context};
use cordis_core::observation::RuntimeObservation;
use std::convert::Infallible;

fn main() {
    let ctx = Context::new();
    ctx.observe_runtime(|_: Context, _: RuntimeObservation| async { Ok::<_, Infallible>(()) }).unwrap();
}
