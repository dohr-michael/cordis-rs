// Only the task's *output* shed its Send bound: the future itself still
// crosses to the executor, so a future holding !Send state across an
// await remains refused.

use cordis_core::Context;
use std::rc::Rc;

fn main() {
    let ctx = Context::new();
    let rc = Rc::new(());
    ctx.run(async move {
        std::future::ready(()).await;
        drop(rc);
    })
    .expect("root admits");
}
