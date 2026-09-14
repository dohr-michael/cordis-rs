use cordis_core::event::observer_sync;
use cordis_core::{Context, Event};
use std::convert::Infallible;

struct Ping;
impl Event for Ping {
    const NAME: &'static str = "ui/move-only-registration";
    type Args = ();
    type Output = ();
}

fn main() {
    let ctx = Context::new();
    let registration = ctx
        .on::<Ping, _>(observer_sync(|_, ()| Ok::<(), Infallible>(() )))
        .unwrap();
    let _alias = registration.clone();
}
