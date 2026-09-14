//! Shared helper for the cordis examples suite (ADR 0011 decisions 2–3).
//!
//! **Mechanics in, policy out.** Every application-shaped probe rewrote
//! the same boot/teardown choreography by hand (probe wall W1, three
//! instances); this crate is the one helper the suite owns instead:
//!
//! - the ops-console trio — [`section`], [`flip!`] (the suite's one tiny
//!   macro), and [`boot_report`];
//! - the boot/teardown mechanics — [`Roster`] (the spawn-ordered
//!   accumulation), per-fork ready rows with ✓ / ⏳ +
//!   [`Fork::pending_missing`] / ✗ + error, and reverse-order
//!   [`teardown`];
//! - the settle seat — [`await_state`], the one wait-with-evidence
//!   spelling for a tour's "proven, not asserted" state waits.
//!
//! It deliberately does **not** own policy: supervision (the two-channel
//! apply-time vs run-time split stays app vocabulary), `internal/*`
//! narration ordering (a harness seam — the console prints what
//! arrives), and error aggregation (the app decides whether to continue
//! past a ✗; the helper only renders the rows and hands back
//! [`BootSummary`]).
//!
//! [`Fork::pending_missing`]: cordis_core::Fork::pending_missing

use std::time::Duration;

use cordis_core::{FiberState, Fork};

/// Print one ops-console section header — the suite's universal
/// narrative beat (`══ title ══`).
pub fn section(title: &str) {
    println!("\n══ {title} ══");
}

/// Report a config flip the way an operator reads it: match on the
/// [`Fork::update`](cordis_core::Fork::update) outcome — ✓ when the
/// swap applied, ⊘ when a layer vetoed (it returned without calling
/// `next`, so nothing swapped), ⚠ with the error when rejected.
///
/// The suite's one macro (the macro verdict: nothing else earned it):
/// plain functions everywhere else. The veto is real but silent in the
/// waterfall itself — this is where the suite names it (ADR 0018).
#[macro_export]
macro_rules! flip {
    ($result:expr, $what:expr) => {
        match $result {
            Ok(cordis_core::UpdateOutcome::Applied) => {
                println!("  ✓ flip accepted: {}", $what)
            }
            Ok(cordis_core::UpdateOutcome::Vetoed) => println!(
                "  ⊘ flip vetoed: {} — a layer returned without calling next",
                $what
            ),
            Err(e) => println!("  ⚠ flip rejected: {} — {e}", $what),
        }
    };
}

/// Shorten a plugin's type name for console rows: the last `::` segment
/// of [`Plugin::name`](cordis_core::Plugin::name)'s default (a
/// `std::any::type_name`), or the name unchanged when it has no path.
///
/// ```
/// assert_eq!(examples_common::short_type_name("hello_plugin::Echo"), "Echo");
/// assert_eq!(examples_common::short_type_name("standalone"), "standalone");
/// ```
pub fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}

/// The deployment's forks in spawn order — the accumulation
/// [`boot_report`] and [`teardown`] consume. `push` is the only way in
/// and hands the same handle back, so callers keep binding names
/// without the clone dance; the spawn-order contract (children die
/// before the furniture they were spawned under) is held by the type,
/// not by a doc note at each call site.
///
/// Reporting mid-accumulation is a supported shape (scopes_tenants
/// boots, reports, and keeps spawning); a one-off single-fork report
/// uses the free [`boot_report`] directly.
#[derive(Debug, Default)]
pub struct Roster {
    forks: Vec<Fork>,
}

impl Roster {
    /// An empty roster.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a spawned fork, in spawn order; returns the same handle,
    /// for the caller to keep binding names to.
    pub fn push(&mut self, fork: Fork) -> Fork {
        self.forks.push(fork.clone());
        fork
    }

    /// [`boot_report`] over the recorded order, so far.
    pub async fn report(&self) -> BootSummary {
        boot_report(&self.forks).await
    }

    /// [`teardown`] over the recorded order, reversed.
    pub async fn teardown(&self) {
        teardown(&self.forks).await
    }
}

impl FromIterator<Fork> for Roster {
    fn from_iter<I: IntoIterator<Item = Fork>>(forks: I) -> Self {
        Self {
            forks: forks.into_iter().collect(),
        }
    }
}

/// The suite's probe-evidence budget for state waits (v1's hand-rolled
/// 5 ms poll loops died at `wait_state`, ADR 0004): a timeout here is
/// CI failure evidence, not a crash policy.
const EVIDENCE_BUDGET: Duration = Duration::from_secs(5);

/// Wait for `fork` to reach `state` under the suite's evidence budget,
/// narrating what the wait proves on failure — the one spelling every
/// tour's "proven, not asserted" state wait collapses into.
pub async fn await_state(fork: &Fork, state: FiberState, proof: &str) {
    fork.wait_state(state, EVIDENCE_BUDGET).await.expect(proof)
}

/// Settled outcome of one [`boot_report`] pass: the row counts the app's
/// continue/abort policy reads. Aggregation itself stays app-side — the
/// helper renders rows, the app decides what they mean (a deployment
/// with a pended row may be exactly the "broken but running" shape
/// worth keeping up, probe wall W12).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BootSummary {
    /// Rows that settled [`Active`](FiberState::Active).
    pub up: usize,
    /// Rows still waiting on missing services (⏳ rows).
    pub pending: usize,
    /// Rows whose apply failed (✗ rows).
    pub failed: usize,
}

/// Wait for every fork to settle and render the boot report: one row per
/// fork — ✓ up, ⏳ waiting for the names [`Fork::pending_missing`]
/// reports, and ✗ with the apply error.
///
/// `forks` must be in spawn order: [`teardown`] disposes the same slice
/// in reverse. Rows settle sequentially in that order, mirroring the
/// probe's hand-written loop. Odd settled states (a fork disposed during
/// boot) render as `?` rows and count nowhere — the summary counts what
/// boot produced, not what a concurrent teardown took away.
///
/// Returns the counts; whether to continue past a ✗ is the app's call.
pub async fn boot_report(forks: &[Fork]) -> BootSummary {
    let mut summary = BootSummary::default();
    for fork in forks {
        match fork.ready().await {
            Ok(FiberState::Active) => {
                println!("  ✓ {} up", short_type_name(fork.name()));
                summary.up += 1;
            }
            Ok(FiberState::Pending) => {
                println!(
                    "  ⏳ {} waiting for {:?}",
                    short_type_name(fork.name()),
                    fork.pending_missing()
                );
                summary.pending += 1;
            }
            Ok(other) => println!("  ? {} {other:?}", short_type_name(fork.name())),
            Err(e) => {
                println!("  ✗ {} failed: {e}", short_type_name(fork.name()));
                summary.failed += 1;
            }
        }
    }
    summary
}

/// Dispose `forks` in reverse spawn order — children die before the
/// furniture they were spawned under (probe wall W1's teardown half).
/// Renders one ✓ row per fork; every effect a plugin registered
/// (listeners, services, fiber-bound tasks) vanishes with its fiber.
///
/// `forks` is the same spawn-ordered slice [`boot_report`] reported on.
/// Already-disposed forks are skipped, so repeated teardown is quiet and
/// idempotent.
pub async fn teardown(forks: &[Fork]) {
    for fork in forks.iter().rev() {
        if fork.state() == FiberState::Disposed {
            continue;
        }
        // A per-Fork operation failure is rendered rather than promoted into
        // Harness policy, so one refusal cannot abort later teardown attempts.
        match fork.dispose().await {
            Ok(()) => println!("  ✓ {} down", short_type_name(fork.name())),
            Err(e) => println!("  ⚠ {} dispose error: {e}", short_type_name(fork.name())),
        }
    }
}
