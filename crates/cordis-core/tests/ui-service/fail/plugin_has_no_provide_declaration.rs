use cordis_core::{Context, Plugin};
use std::convert::Infallible;
use std::future::{Future, ready};

struct P;
impl Plugin for P {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn provide(&self) -> &[&'static str] { &["service"] }
    fn prepare(&self, (): ()) -> Result<(), Infallible> { Ok(()) }
    fn apply(&self, _: Context, _: &()) -> impl Future<Output = Result<(), Infallible>> + Send {
        ready(Ok(()))
    }
}

fn main() {}
