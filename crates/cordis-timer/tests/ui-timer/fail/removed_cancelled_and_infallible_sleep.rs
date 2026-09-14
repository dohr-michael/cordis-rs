use cordis_core::Context;
use cordis_timer::{Cancelled, Sleep, TimerExt};
use std::time::Duration;

fn main() {
    let ctx = Context::new();
    let _: Sleep = ctx.sleep(Duration::ZERO);
    let _ = std::mem::size_of::<Cancelled>();
}
