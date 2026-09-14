use cordis_core::lifecycle::{LifecycleRecursion, WaitStateError};

fn legacy(error: LifecycleRecursion) {
    let _ = WaitStateError::SettleRecursion(error);
}

fn main() {}
