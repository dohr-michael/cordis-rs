use cordis_core::Fork;

async fn old_dispose_channel(fork: &Fork) {
    let _: cordis_core::Result<()> = fork.dispose().await;
}

fn main() {}
