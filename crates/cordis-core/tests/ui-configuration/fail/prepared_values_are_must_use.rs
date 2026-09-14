#![deny(unused_must_use)]

use cordis_core::{Context, InjectSpec, Plugin, PreparedChange, PreparedPlugin};
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
    InjectSpec::none();
    PreparedPlugin::from_input(P, ());
    PreparedChange::from_input::<P>(());
}
