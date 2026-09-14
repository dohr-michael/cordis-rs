use cordis_core::event::{observer, observer_sync};
use cordis_core::observation::RuntimeObservation;
use cordis_core::Context;
use std::convert::Infallible;

fn main() {
    let ctx = Context::new();
    ctx.observe_runtime(observer(|_: Context, _: RuntimeObservation| async { Ok::<_, Infallible>(()) })).unwrap();
    ctx.observe_runtime(observer_sync(|_: Context, _: RuntimeObservation| Ok::<_, Infallible>(()))).unwrap();
}
