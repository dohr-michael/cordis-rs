use cordis_core::Context;

async fn old_shutdown_surface(ctx: &Context) {
    ctx.shutdown().await;
}

fn main() {}
