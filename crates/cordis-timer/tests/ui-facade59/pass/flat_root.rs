use cordis_timer::{Interval, Sleep, Timeout, TimeoutOutcome, TimerCancelled, TimerExt, TimerRegistrationError};
use std::future::Future;
fn ty<T>() {}
fn future_ty<F: Future>() { ty::<Timeout<F>>(); }
fn main() {
    ty::<Interval>(); ty::<Sleep>(); ty::<TimeoutOutcome<()>>(); ty::<TimerCancelled>(); ty::<TimerRegistrationError>();
    fn assert_ext<T: TimerExt>() {}
    assert_ext::<cordis_core::Context>();
    future_ty::<std::future::Ready<()>>();
}
