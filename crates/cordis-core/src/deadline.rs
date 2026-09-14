//! The std-clock watchdog idiom, written once. Core carries no tokio
//! time feature (ADR 0002 — timer discipline lives in cordis-timer), so
//! a deadline is a std thread + oneshot raced against the wait via
//! `select`. Before this module the shape was hand-copied at every
//! deadline site; both arms below state the rationale so no site
//! re-derives it.

/// Raw watchdog arm: resolves after `timeout` on its own std thread.
/// Race it via `select` — see `bounded` for the common one-shot shape.
#[cfg(test)]
use std::future::Future;

pub(crate) fn watchdog(timeout: std::time::Duration) -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        let _ = tx.send(());
    });
    rx
}

/// Bounded wait for assertions that must NOT resolve in time: the
/// deadline is the [`watchdog`] thread raced against the future via
/// `select`. Returns `None` on deadline (the expected arm for "must
/// not leak" rows). Unit-tier arm: the contract tiers carry their own
/// copy in `tests/common` (they are separate crates).
#[cfg(test)]
pub(crate) async fn bounded<T>(millis: u64, fut: impl Future<Output = T>) -> Option<T> {
    let rx = watchdog(std::time::Duration::from_millis(millis));
    tokio::pin!(fut);
    tokio::pin!(rx);
    tokio::select! {
        out = &mut fut => Some(out),
        _ = &mut rx => None,
    }
}
