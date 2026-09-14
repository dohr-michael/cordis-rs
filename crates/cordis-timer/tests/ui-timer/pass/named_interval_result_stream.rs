use std::time::Duration;

use cordis_core::Context;
use cordis_timer::{Interval, TimerCancelled, TimerExt};
use futures::Stream;

fn accepts_result_stream<S: Stream<Item = Result<(), TimerCancelled>>>(_: S) {}
fn accepts_named(_: Interval) {}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _guard = runtime.enter();
    let ctx = Context::new();
    accepts_named(ctx.interval(Duration::from_millis(1)).unwrap());
    accepts_result_stream(ctx.interval(Duration::from_millis(1)).unwrap());
}
