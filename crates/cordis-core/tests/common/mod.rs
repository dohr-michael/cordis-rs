//! Shared scaffolding for the cordis-core contract tests.
//!
//! Each test file links this module whole (`mod common;`) and uses a
//! subset, so an unused helper here is expected, not drift.
#![allow(dead_code)]

use std::future::Future;

/// Bounded wait (std-clock watchdog thread — core carries no tokio time
/// feature, ADR 0002, and tests share the crate's dependency set):
/// `None` means the future did not resolve within `millis` — the shape
/// a missing refusal or a leak would take. The one-shot shape of
/// core's crate-private `deadline` module; contract tiers are separate
/// crates, so their copy lives here beside `deadlock_watchdog`.
pub async fn bounded<T>(millis: u64, fut: impl Future<Output = T>) -> Option<T> {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(millis));
        let _ = tx.send(());
    });
    tokio::pin!(fut);
    tokio::pin!(rx);
    tokio::select! {
        out = &mut fut => Some(out),
        _ = &mut rx => None,
    }
}

/// Deadlock watchdog — the converged shape of the ADR 0010 lock-law
/// probes (ADR 0004's pre-seeded convergence list names "watchdog test
/// scaffolding"): `probe` runs on a plain worker thread and its result
/// only reaches the caller after it returns, so user code deadlocking
/// under a lock surfaces as the 5 s recv timeout — a failing test, not a
/// hung suite. `failure` names the probe and the law it pins.
pub fn deadlock_watchdog<T: Send + 'static>(
    failure: &str,
    probe: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = probe();
        let _ = tx.send(out);
    });
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .unwrap_or_else(|_| panic!("{failure}"))
}

/// Count current ordinary residents through the final topology-free observation facade.
pub fn ordinary_fiber_count(ctx: &cordis_core::Context) -> usize {
    ctx.runtime_snapshot()
        .fibers()
        .iter()
        .filter(|fiber| fiber.role() == cordis_core::lifecycle::FiberRole::Ordinary)
        .count()
}
