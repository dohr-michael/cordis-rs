use cordis_core::Context;
use cordis_core::observation::{FiberSnapshot, RuntimeSnapshot, ServicePublicationId, ServiceSnapshot};

fn main() {
    let ctx = Context::new();
    let _ = RuntimeSnapshot { fibers: vec![], services: vec![] };
    let _ = FiberSnapshot {
        id: ctx.runtime_snapshot().fibers()[0].id().clone(),
        role: cordis_core::lifecycle::FiberRole::Root,
        name: String::new(),
        state: cordis_core::FiberState::Active,
        missing_services: vec![],
    };
    let _ = ServiceSnapshot {
        id: panic!(), service: String::new(), realm: ctx.new_service_realm(),
        provider: ctx.runtime_snapshot().fibers()[0].id().clone(), visible: false,
    };
    let id: &ServicePublicationId = panic!();
    let _ = id.as_u64();
    let _ = ServicePublicationId::from(1_u64);
    let _ = ctx.registry_snapshot();
    let _ = ctx.services_snapshot();
    let _ = ctx.live_fiber_count();
    let _ = ctx.pending_missing();
}
