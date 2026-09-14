use cordis_core::Context;
use cordis_loader::plan::RealmPolicy;

fn main() {
    let ctx = Context::new();
    let policy = RealmPolicy::Shared {
        label: "tenant-a".to_owned(),
    };
    let _ = ctx.with_service_realms([("service", policy)]);
}
