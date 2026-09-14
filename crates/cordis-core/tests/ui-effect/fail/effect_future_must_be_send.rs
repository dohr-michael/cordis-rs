// Async cleanup futures are Send: a future holding an Rc across an
// await is rejected.

use cordis_core::Context;
use std::rc::Rc;

fn main() {
    let ctx = Context::new();
    let _ = ctx.effect(|| {
        let rc = Rc::new(());
        async move {
            std::future::pending::<()>().await;
            let _ = &rc;
        }
    });
}
