use cordis_timer::{TimerCancelled, TimerRegistrationError};
fn registration(e: TimerRegistrationError) { match e { TimerRegistrationError::InactiveContext | TimerRegistrationError::TimerUnavailable | TimerRegistrationError::ZeroPeriod | TimerRegistrationError::DeadlineOutOfRange => {}, _ => {} } }
fn cancellation(_: TimerCancelled) {}
fn main() { let _ = (registration, cancellation); }
