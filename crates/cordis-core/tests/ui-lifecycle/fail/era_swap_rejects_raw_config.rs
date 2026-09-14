use cordis_core::lifecycle::Fork;

async fn check(fork: Fork) {
    let _ = fork.era_swap(7u8).await;
}

fn main() {}
