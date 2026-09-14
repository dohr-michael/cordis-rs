use cordis_core::event::ListenerRegistrationId;
use std::sync::Arc;

fn main() {
    let _ = ListenerRegistrationId(Arc::new(0u8));
}
