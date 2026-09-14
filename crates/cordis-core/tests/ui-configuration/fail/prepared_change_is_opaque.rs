use cordis_core::{Context, Plugin, PreparedChange};
use std::convert::Infallible;
use std::future::{Future, ready};

struct P;
impl Plugin for P {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;
    fn prepare(&self, _: ()) -> Result<(), Infallible> { Ok(()) }
    fn apply(&self, _: Context, _: &()) -> impl Future<Output = Result<(), Infallible>> + Send {
        ready(Ok(()))
    }
}

fn main() {
    let change = PreparedChange::from_input::<P>(());
    let _ = change.contract;
    let _ = change.input;
}
