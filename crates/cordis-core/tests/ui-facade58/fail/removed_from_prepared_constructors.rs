use cordis_core::{Context, Plugin, PreparedChange, PreparedPlugin};
use std::convert::Infallible;

struct P;
impl Plugin for P {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn prepare(&self, (): ()) -> Result<Self::Input, Self::PrepareError> { Ok(()) }
    async fn apply(&self, _: Context, _: &Self::Input) -> Result<(), Self::ApplyError> { Ok(()) }
}

fn main() {
    let _ = PreparedPlugin::from_prepared(P, ());
    let _ = PreparedChange::from_prepared::<P>(());
}
