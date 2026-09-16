//! The std-clock watchdog idiom, written once. Core carries no tokio
//! time feature (ADR 0002 — timer discipline lives in cordis-timer), so
//! a deadline is a std thread + oneshot raced against the wait via
//! `select`. Before this module the shape was hand-copied at every
//! deadline site; both arms below state the rationale so no site
//! re-derives it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Raw watchdog arm: resolves after `timeout` on its own std thread.
/// If the future is dropped first, its drop guard marks cancellation and
/// unparks the worker immediately. `Thread::unpark` retains one token, so a
/// cancellation racing the worker between its state check and `park_timeout`
/// cannot be lost. Race it via `select` — see `bounded` for the common one-shot
/// shape.
#[cfg(test)]
use std::sync::atomic::AtomicUsize;

#[cfg(test)]
const TRACKED_WATCHDOG_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(13);
#[cfg(test)]
static LIVE_WATCHDOG_THREADS: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct WatchdogCancellation {
    cancelled: AtomicBool,
    #[cfg(test)]
    parking: AtomicBool,
    #[cfg(test)]
    requested_park_millis: AtomicUsize,
}

impl WatchdogCancellation {
    fn cancel(&self, worker: &std::thread::Thread) {
        self.cancelled.store(true, Ordering::Release);
        worker.unpark();
    }

    /// Returns true only when the deadline elapsed before cancellation.
    fn wait_until_timeout(&self, timeout: std::time::Duration) -> bool {
        let started = std::time::Instant::now();
        loop {
            if self.cancelled.load(Ordering::Acquire) {
                return false;
            }
            let elapsed = started.elapsed();
            if elapsed >= timeout {
                return true;
            }

            let remaining = timeout - elapsed;
            #[cfg(test)]
            {
                self.requested_park_millis.store(
                    remaining.as_millis().min(usize::MAX as u128) as usize,
                    Ordering::SeqCst,
                );
                self.parking.store(true, Ordering::SeqCst);
            }
            std::thread::park_timeout(remaining);
            #[cfg(test)]
            self.parking.store(false, Ordering::SeqCst);
        }
    }

    #[cfg(test)]
    fn is_parking(&self) -> bool {
        self.parking.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    fn requested_park_millis(&self) -> usize {
        self.requested_park_millis.load(Ordering::SeqCst)
    }
}

pub(crate) struct Watchdog {
    receiver: tokio::sync::oneshot::Receiver<()>,
    cancellation: Arc<WatchdogCancellation>,
    worker: std::thread::Thread,
}

impl std::future::Future for Watchdog {
    type Output = Result<(), tokio::sync::oneshot::error::RecvError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.receiver).poll(cx)
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.cancellation.cancel(&self.worker);
    }
}

#[cfg(test)]
pub(crate) fn tracked_watchdog_timeout() -> std::time::Duration {
    TRACKED_WATCHDOG_TIMEOUT
}

#[cfg(test)]
pub(crate) fn live_watchdog_threads() -> usize {
    LIVE_WATCHDOG_THREADS.load(Ordering::SeqCst)
}

pub(crate) fn watchdog(timeout: std::time::Duration) -> Watchdog {
    let (tx, receiver) = tokio::sync::oneshot::channel::<()>();
    let cancellation = Arc::new(WatchdogCancellation::default());
    let worker_cancellation = cancellation.clone();
    #[cfg(test)]
    let tracked = timeout == TRACKED_WATCHDOG_TIMEOUT;
    #[cfg(test)]
    if tracked {
        LIVE_WATCHDOG_THREADS.fetch_add(1, Ordering::SeqCst);
    }
    let worker = std::thread::spawn(move || {
        let timed_out = worker_cancellation.wait_until_timeout(timeout);
        if timed_out {
            let _ = tx.send(());
        }
        #[cfg(test)]
        if tracked {
            LIVE_WATCHDOG_THREADS.fetch_sub(1, Ordering::SeqCst);
        }
    });
    let worker_thread = worker.thread().clone();
    drop(worker);
    Watchdog {
        receiver,
        cancellation,
        worker: worker_thread,
    }
}

/// Bounded wait for assertions that must NOT resolve in time: the
/// deadline is the [`watchdog`] thread raced against the future via
/// `select`. Returns `None` on deadline (the expected arm for "must
/// not leak" rows). Unit-tier arm: the contract tiers carry their own
/// copy in `tests/common` (they are separate crates).
#[cfg(test)]
pub(crate) async fn bounded<T>(
    millis: u64,
    fut: impl std::future::Future<Output = T>,
) -> Option<T> {
    let rx = watchdog(std::time::Duration::from_millis(millis));
    tokio::pin!(fut);
    tokio::pin!(rx);
    tokio::select! {
        out = &mut fut => Some(out),
        _ = &mut rx => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::time::{Duration, Instant};

    fn wait_until(timeout: Duration, condition: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + timeout;
        while !condition() {
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::yield_now();
        }
        true
    }

    #[test]
    fn cancellation_before_worker_wait_is_not_lost() {
        let cancellation = Arc::new(WatchdogCancellation::default());
        let start = Arc::new(Barrier::new(2));
        let waiting = cancellation.clone();
        let worker_start = start.clone();
        let worker = std::thread::spawn(move || {
            worker_start.wait();
            waiting.wait_until_timeout(Duration::from_secs(2))
        });
        let worker_thread = worker.thread().clone();

        cancellation.cancel(&worker_thread);
        start.wait();

        assert!(!worker.join().expect("watchdog worker stays alive"));
    }

    #[test]
    fn cancellation_at_worker_park_boundary_wakes_the_worker() {
        let cancellation = Arc::new(WatchdogCancellation::default());
        let waiting = cancellation.clone();
        let worker = std::thread::spawn(move || waiting.wait_until_timeout(Duration::from_secs(2)));
        let worker_thread = worker.thread().clone();

        assert!(wait_until(Duration::from_secs(2), || cancellation.is_parking()));
        cancellation.cancel(&worker_thread);

        assert!(!worker.join().expect("watchdog worker stays alive"));
    }

    #[test]
    fn concurrent_long_waits_schedule_remaining_deadlines_not_poll_slices() {
        const WATCHDOGS: usize = 8;
        let watchdogs = (0..WATCHDOGS)
            .map(|_| watchdog(Duration::from_secs(5)))
            .collect::<Vec<_>>();
        let cancellations = watchdogs
            .iter()
            .map(|watchdog| watchdog.cancellation.clone())
            .collect::<Vec<_>>();

        assert!(wait_until(Duration::from_secs(2), || {
            cancellations.iter().all(|state| state.is_parking())
        }));
        assert!(
            cancellations
                .iter()
                .all(|state| state.requested_park_millis() >= 4_000),
            "workers must park for the remaining deadline, not a short polling slice",
        );

        // The retired worker capped every sleep at 50 ms, so eight 175-ms
        // waits scheduled at least 24 polling wakeups. Even if `park_timeout`
        // returns spuriously, this worker re-parks for the full remaining
        // deadline rather than introducing an intentional short polling loop.
        std::thread::sleep(Duration::from_millis(175));
        assert!(
            cancellations
                .iter()
                .all(|state| state.requested_park_millis() >= 4_000),
        );

        drop(watchdogs);
    }
}
