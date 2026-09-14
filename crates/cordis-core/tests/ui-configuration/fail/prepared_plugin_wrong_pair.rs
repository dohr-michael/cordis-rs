use cordis_core::{Context, Plugin, PreparedPlugin};
use std::convert::Infallible;
use std::future::{Future, ready};

struct P;

impl Plugin for P {
    type Config = ();
    type Input = String;
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn prepare(&self, _: ()) -> Result<String, Infallible> { Ok(String::new()) }
    fn apply(&self, _: Context, _: &String) -> impl Future<Output = Result<(), Infallible>> + Send {
        ready(Ok(()))
    }
}

fn main() {
    let _ = PreparedPlugin::from_input(P, 42_u32);
}
