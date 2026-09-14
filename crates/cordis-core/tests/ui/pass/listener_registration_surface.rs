use cordis_core::event::{observer_sync, ListenerRegistrationId};
use cordis_core::{Context, Event};
use std::convert::Infallible;
use std::hash::Hash;

struct Ping;
impl Event for Ping {
    const NAME: &'static str = "ui/listener-registration";
    type Args = ();
    type Output = ();
}

fn assert_id_traits<T: std::fmt::Debug + Clone + Eq + Hash>() {}

fn main() {
    assert_id_traits::<ListenerRegistrationId>();
    let ctx = Context::new();
    let registration = ctx
        .on::<Ping, _>(observer_sync(|_, ()| Ok::<(), Infallible>(() )))
        .unwrap();
    let _ = registration.remove();
}
