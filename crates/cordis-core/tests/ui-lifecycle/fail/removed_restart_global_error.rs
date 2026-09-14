use cordis_core::Fork;

async fn old_restart_channel(fork: &Fork) {
    let _: cordis_core::Result<()> = fork.restart().await;
}

fn main() {}
