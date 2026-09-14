use cordis_core::observation::{FiberSnapshot, RuntimeSnapshot, ServicePublicationId, ServiceSnapshot};
use cordis_core::{FiberId, ServiceRealm};

fn construct_service(id: ServicePublicationId, realm: ServiceRealm, provider: FiberId) {
    let _ = ServiceSnapshot {
        id, service: String::new(), realm, provider, visible: false,
    };
}

fn main() {
    let ctx = cordis_core::Context::new();
    let snapshot = ctx.runtime_snapshot();
    let root = &snapshot.fibers()[0];
    let _ = RuntimeSnapshot { fibers: vec![], services: vec![] };
    let _ = FiberSnapshot {
        id: root.id().clone(),
        role: cordis_core::lifecycle::FiberRole::Root,
        name: String::new(),
        state: cordis_core::FiberState::Active,
        missing_services: vec![],
    };
}
