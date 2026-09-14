use cordis_core::{Context, Service};
use std::sync::Arc;

struct S;
impl Service for S { const NAME: &'static str = "s"; }

fn main() {
    let ctx = Context::new();
    ctx.set_service(Arc::new(S));
    ctx.remove_service::<S>();
    let _ = ctx.services_snapshot();
    let _: Option<cordis_core::service::ServiceRecord> = None;
}
