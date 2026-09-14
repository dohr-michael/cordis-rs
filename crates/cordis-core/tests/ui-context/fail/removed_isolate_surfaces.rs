use cordis_core::{Context, IsolateId, IsolateRealm};

fn main() {
    let ctx = Context::new();
    let _ = IsolateId::GLOBAL;
    let _ = IsolateRealm::Private;
    let _ = ctx.with_isolate_realms([(String::from("service"), IsolateRealm::Private)]);
}
