use cordis_core::{Context, FiberId};

fn removed_registry_types() {
    let _: Option<cordis_core::RuntimeId> = None;
    let _: Option<cordis_core::RuntimeRecord> = None;
    let _: Option<cordis_core::RuntimeFiberRecord> = None;
}

fn main() {
    let _forged = FiberId(std::sync::Arc::new(0));
    let ctx = Context::new();
    let _topology = ctx.registry();
    let _records = ctx.registry_snapshot();
    let _: Option<cordis_core::fiber::FiberId> = None;
}
