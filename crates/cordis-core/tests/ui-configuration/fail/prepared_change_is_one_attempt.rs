use cordis_core::{Context, Fork, Plugin, PreparedChange};
use std::convert::Infallible;
use std::future::{Future, ready};

struct P;

impl Plugin for P {
    type Config = ();
    type Input = String;
    type PrepareError = Infallible;
    type ApplyError = Infallible;

    fn prepare(&self, _: ()) -> Result<String, Infallible> {
        Ok(String::new())
    }

    fn apply(
        &self,
        _: Context,
        _: &String,
    ) -> impl Future<Output = Result<(), Infallible>> + Send {
        ready(Ok(()))
    }
}

// A PreparedChange is consumed by one attempted lifecycle operation: the
// real `Fork::update` entry takes it by value, so a second attempt does
// not compile.
#[allow(dead_code)]
async fn drive(fork: Fork) {
    let candidate = PreparedChange::from_input::<P>(String::new());
    let _ = fork.update(candidate).await;
    let _ = fork.update(candidate).await;
}

fn main() {}
