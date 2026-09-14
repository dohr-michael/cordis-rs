use cordis_core::{Context, Service};
use std::sync::Arc;

struct S;
impl Service for S { const NAME: &'static str = "s"; }

fn main() {
    let ctx = Context::new();
    let publication = ctx.provide(Arc::new(S)).unwrap();
    let _copy = publication.clone();
}
