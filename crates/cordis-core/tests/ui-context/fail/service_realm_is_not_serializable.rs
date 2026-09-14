use cordis_core::ServiceRealm;
use serde::{Deserialize, Serialize};

fn requires_serialize<T: Serialize>() {}
fn requires_deserialize<T: for<'de> Deserialize<'de>>() {}

fn main() {
    requires_serialize::<ServiceRealm>();
    requires_deserialize::<ServiceRealm>();
}
