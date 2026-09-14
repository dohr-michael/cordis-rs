use cordis_core::Context;
use cordis_timer::TimerExt;
use std::time::Duration;

fn main() {
    let ctx = Context::new();
    let _ = ctx.timeout(Duration::from_millis(1));
}
