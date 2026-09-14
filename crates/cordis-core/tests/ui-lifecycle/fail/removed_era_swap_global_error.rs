use cordis_core::{PreparedChange, Result};
use cordis_core::lifecycle::Fork;

async fn check(fork: Fork, change: PreparedChange) {
    let _: Result<Fork> = fork.era_swap(change).await;
}

fn main() {}
