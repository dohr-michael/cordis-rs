use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

use cordis_core::Context;
use cordis_timer::{TimerExt, Timeout};

fn witness<'a>(ctx: &Context, text: &'a str, local: Rc<()>) {
    let work = async move {
        let _local = local;
        Rc::new(text)
    };
    let _: Result<Timeout<_>, _> = ctx.timeout(Duration::from_secs(1), work);
}

fn assert_future<'a, F: Future>(_future: F) where F::Output: 'a {}

fn main() {}
