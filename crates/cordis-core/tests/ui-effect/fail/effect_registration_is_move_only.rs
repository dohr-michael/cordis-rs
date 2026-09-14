// Both manual operations consume the registration: a second use is a
// move error, so dispose and disarm can never race each other.

use cordis_core::Context;

fn main() {
    let ctx = Context::new();
    let registration = ctx.effect_sync(|| {}).unwrap();
    let _ = registration.disarm();
    let _ = registration.dispose();
}
