use cordis_core::{FiberState, Fork};

async fn old_ready_channel(fork: &Fork) {
    let _: cordis_core::Result<FiberState> = fork.ready().await;
}

fn main() {}
