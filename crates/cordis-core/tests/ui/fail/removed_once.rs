use cordis_core::event::observer_sync;
use cordis_core::{Context, Event};
use std::convert::Infallible;

struct Ping;
impl Event for Ping {
    const NAME: &'static str = "ui/removed-once";
    type Args = ();
    type Output = ();
}

fn main() {
    let ctx = Context::new();
    let _ = ctx.once::<Ping, _>(observer_sync(|_, ()| Ok::<(), Infallible>(() )));
}
