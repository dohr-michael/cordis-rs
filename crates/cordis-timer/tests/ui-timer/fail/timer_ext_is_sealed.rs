use std::future::Future;
use std::time::Duration;
use cordis_timer::{Interval, Sleep, Timeout, TimerExt, TimerRegistrationError};

struct Foreign;

impl TimerExt for Foreign {
    fn timeout<F: Future>(&self, _: Duration, _: F) -> Result<Timeout<F>, TimerRegistrationError> { unimplemented!() }
    fn sleep(&self, _: Duration) -> Result<Sleep, TimerRegistrationError> { unimplemented!() }
    fn interval(&self, _: Duration) -> Result<Interval, TimerRegistrationError> { unimplemented!() }
}

fn main() {}
