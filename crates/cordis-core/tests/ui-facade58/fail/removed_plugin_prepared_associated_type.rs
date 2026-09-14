use cordis_core::{Context, Plugin};
use std::convert::Infallible;

struct P;
impl Plugin for P {
    type Config = ();
    type Prepared = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn prepare(&self, (): ()) -> Result<Self::Prepared, Self::PrepareError> { Ok(()) }
    async fn apply(&self, _: Context, _: &Self::Prepared) -> Result<(), Self::ApplyError> { Ok(()) }
}

fn main() {}
