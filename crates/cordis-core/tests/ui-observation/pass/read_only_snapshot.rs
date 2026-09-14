use cordis_core::{Context, FiberState};
use cordis_core::lifecycle::FiberRole;
use cordis_core::observation::{FiberSnapshot, RuntimeSnapshot, ServicePublicationId, ServiceSnapshot};
use std::fmt::Debug;
use std::hash::Hash;

fn clone_debug<T: Clone + Debug>(_: &T) {}
fn opaque_id<T: Clone + Debug + Eq + Hash>(_: &T) {}

fn main() {
    let snapshot: RuntimeSnapshot = Context::new().runtime_snapshot();
    clone_debug(&snapshot);
    for fiber in snapshot.fibers() {
        let _: &cordis_core::FiberId = fiber.id();
        let _: FiberRole = fiber.role();
        let _: &str = fiber.name();
        let _: FiberState = fiber.state();
        let _: &[String] = fiber.missing_services();
        clone_debug::<FiberSnapshot>(fiber);
    }
    for service in snapshot.services() {
        opaque_id::<ServicePublicationId>(service.id());
        let _: &str = service.service();
        let _: &cordis_core::ServiceRealm = service.realm();
        let _: &cordis_core::FiberId = service.provider();
        let _: bool = service.visible();
        clone_debug::<ServiceSnapshot>(service);
    }
}
