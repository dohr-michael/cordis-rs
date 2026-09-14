use cordis_core::ServiceRealm;

fn requires_display<T: std::fmt::Display>() {}
fn requires_order<T: Ord>() {}

fn main() {
    let _ = ServiceRealm::default();
    requires_display::<ServiceRealm>();
    requires_order::<ServiceRealm>();
}
