//! Cordis Timer: complete generation-owned time operations.
//!
//! Timer owns scheduling, monotonic deadlines, cancellation outcomes, and
//! operation delivery. Core sees only generic generation cleanup obligations.

mod shapes;

pub use shapes::{Interval, Sleep, Timeout, TimeoutOutcome, TimerExt};

/// Generation cleanup cancelled an already-constructed timer operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("timer cancelled")]
pub struct TimerCancelled;

/// Why a timer operation could not be registered. Every variant is a
/// synchronous pre-delivery refusal: no operation is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TimerRegistrationError {
    /// The selected Context generation is closed to new cleanup.
    #[error("the context's fiber generation is inactive")]
    InactiveContext,
    /// No usable Tokio time environment is current.
    #[error("no usable timer environment is current")]
    TimerUnavailable,
    /// Interval periods must be nonzero.
    #[error("interval period must be nonzero")]
    ZeroPeriod,
    /// The requested monotonic deadline cannot be represented.
    #[error("timer deadline is out of range")]
    DeadlineOutOfRange,
}
