use cordis_core::Context;

fn main() {
    let realm = Context::new().new_service_realm();
    let _ = realm.key;
}
