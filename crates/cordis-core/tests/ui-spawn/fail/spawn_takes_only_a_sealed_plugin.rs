use cordis_core::{Context, Plugin};
use std::borrow::Cow;
use std::convert::Infallible;

#[derive(Debug, thiserror::Error)]
#[error("boom")]
struct Boom;

struct Raw;

impl Plugin for Raw {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Boom;

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("raw")
    }

    fn prepare(&self, _config: ()) -> Result<(), Infallible> {
        Ok(())
    }

    async fn apply(&self, _ctx: Context, _prepared: &()) -> Result<(), Boom> {
        Ok(())
    }
}

#[allow(dead_code)]
async fn spawn_paths(ctx: Context) {
    // the raw Plugin is not an admission input
    let _ = ctx.spawn(Raw).await;
    // and neither is a bare config value
    let _ = ctx.spawn(()).await;
}

fn main() {}
