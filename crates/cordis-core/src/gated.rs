//! The gated-publish seam: the single home of the registration protocol
//! every fiber-lifecycle-bound resource crosses (ADR 0015).
//!
//! The protocol, fixed: gate (lock-free liveness assert) → journal
//! lock → under-lock recheck → [`PublishStep::publish`] (fallible — a
//! refusal means *nothing happened*: nothing published, no cleanup
//! obligation committed) → cleanup-obligation push. The variable part —
//! what to publish — crosses the section as a [`PublishStep`], a
//! crate-only trait: no user-implementable bound, no closure, no `dyn`
//! dispatch inside the section (ADR 0010 decision 1's law, in the trait
//! spelling of the entry-enum it ratified).
//!
//! # Nesting posture (the crate's lock-order audit at landing, ADR 0015)
//!
//! The canonical nesting is **disposables outside, store inside**:
//! every resource-publishing step with a semantic-owner store takes that
//! store's bookkeeping lock beneath the disposables guard this module holds.
//! Unifying on that direction reversed
//! exactly one pre-existing edge — `provide`'s slots ∋ disposables —
//! and the audit of the crate's full nesting map (eight nesting sites
//! across six distinct lock pairs at landing time) found the reversal
//! cycle-free. The exact publisher set has evolved since that landing
//! audit; the retained v3 invariant is the direction, not a store inventory:
//! no semantic-owner store path takes its bookkeeping lock and then the
//! cleanup journal lock. Drains run cleanups with no guard held, so a
//! withdrawal firing during drain never re-enters this order.

use crate::effect::Cleanup;
use crate::fiber::Fiber;

/// What a gated push publishes inside its critical section. Crate-only
/// by visibility — the section stays free of user-implementable code
/// (ADR 0010 decision 1). The enum → trait move is ADR 0015's: one enum
/// cannot carry every publisher's payload shape without infecting every
/// caller's signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PublishRefused;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GatedPublishError {
    InactiveContext,
    PublishRefused,
}

pub(crate) trait PublishStep {
    /// Make the resource visible. Runs between the under-lock liveness
    /// recheck and the cleanup push; `Err` aborts the whole
    /// registration with nothing published and nothing to roll back.
    /// By-value payloads move out of `self` — on refusal the step (and
    /// its payload) drops at the call site, outside every lock: the
    /// two-phase drop discipline by construction (ADR 0010).
    fn publish(&mut self) -> std::result::Result<(), PublishRefused>;
}

/// Publish nothing — the plain effect-push step.
pub(crate) struct NoPublish;

impl PublishStep for NoPublish {
    fn publish(&mut self) -> std::result::Result<(), PublishRefused> {
        Ok(())
    }
}

/// The gated, atomic core every registration path crosses: gate →
/// lock → recheck → publish → push, all fixed code (ADR 0010's
/// `push_gated` shape, promoted from a `Fiber` method to the seam).
///
/// The cleanup obligation arrives already wrapped in the journal's
/// normalized [`Cleanup`] shape: construction and any result conversion
/// are caller-side, so nothing user-owned executes inside the section.
pub(crate) fn push_gated(
    fiber: &Fiber,
    cleanup: Cleanup,
    step: &mut impl PublishStep,
) -> std::result::Result<crate::effect::DisposableToken, GatedPublishError> {
    fiber
        .assert_can_register()
        .map_err(|_| GatedPublishError::InactiveContext)?;
    let mut list = fiber.disposables.lock();
    // Re-check under the list lock. Dispose marks death
    // BEFORE draining tokens, so a push that wins the lock before the
    // mark is drained normally and one that lands after sees the dead
    // mark here: a registered cleanup can never strand behind an
    // already-taken drain snapshot. Closes the register-vs-dispose
    // TOCTOU class for every registration path.
    fiber
        .assert_can_register()
        .map_err(|_| GatedPublishError::InactiveContext)?;
    // Publish before the cleanup push: a refused step (a live
    // duplicate, …) leaves no obligation behind — refusal means nothing
    // happened, and no call site has a rollback to forget. The refused
    // cleanup itself drops after this guard releases (parameters drop
    // after locals), never inside the critical section.
    step.publish()
        .map_err(|_| GatedPublishError::PublishRefused)?;
    let token = list.push(cleanup);
    Ok(token)
}
