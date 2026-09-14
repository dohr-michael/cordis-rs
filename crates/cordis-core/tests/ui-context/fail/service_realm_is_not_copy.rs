use cordis_core::ServiceRealm;

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<ServiceRealm>();
}
