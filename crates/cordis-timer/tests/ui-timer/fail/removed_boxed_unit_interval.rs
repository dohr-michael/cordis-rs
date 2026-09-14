use std::time::Duration;

use cordis_core::Context;
use cordis_timer::TimerExt;
use futures::stream::BoxStream;

fn main() {
    let ctx = Context::new();
    let _: Result<BoxStream<'static, ()>, _> = ctx.interval(Duration::from_millis(1));
}
