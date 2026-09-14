//! Issue 33 contract: completion-aware notification, parallel notification, and query.

mod common;

use cordis_core::event::{
    DispatchError, InvocationFailureKind, ListenerOptions, ListenerRegistration, mapper_sync,
    observer, observer_sync, responder, responder_sync, with_state,
};
use cordis_core::{Context, Event, QueryOutcome, Routing};
use parking_lot::Mutex;
use std::convert::Infallible;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Notify;
impl Event for Notify {
    const NAME: &'static str = "issue33/notify";
    type Args = usize;
    type Output = usize;
}

struct Ask;
impl Event for Ask {
    const NAME: &'static str = "issue33/ask";
    type Args = usize;
    type Output = usize;
}

#[tokio::test]
async fn emit_is_ordered_awaited_fail_first_and_correlates_the_failure() {
    let ctx = Context::new();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started_tx)));
    let release = Arc::new(tokio::sync::Notify::new());
    let later_hits = Arc::new(AtomicUsize::new(0));

    let first_started = started.clone();
    let first_release = release.clone();
    ctx.on::<Notify, _>(observer(move |_, _| {
        let started = first_started.clone();
        let release = first_release.clone();
        async move {
            if let Some(tx) = started.lock().take() {
                let _ = tx.send(());
            }
            release.notified().await;
            Err::<(), std::io::Error>(std::io::Error::other("emit-first"))
        }
    }))
    .unwrap();
    let later = later_hits.clone();
    ctx.on::<Notify, _>(responder_sync(move |_, _| {
        later.fetch_add(1, Ordering::SeqCst);
        Ok::<_, Infallible>(Some(99))
    }))
    .unwrap();

    let emitter = ctx.clone();
    let dispatch = tokio::spawn(async move { emitter.emit::<Notify>(Routing::Unscoped, 7).await });
    started_rx.await.unwrap();
    assert_eq!(
        later_hits.load(Ordering::SeqCst),
        0,
        "emit awaits each listener before the next"
    );
    release.notify_one();

    let DispatchError::Invocation(failure) = dispatch.await.unwrap().unwrap_err() else {
        panic!("emit failure must use the Invocation phase")
    };
    assert_eq!(failure.kind(), InvocationFailureKind::ReturnedError);
    assert_eq!(failure.diagnostic(), "emit-first");
    assert!(failure.registration_id().is_some());
    assert_eq!(
        later_hits.load(Ordering::SeqCst),
        0,
        "emit stops at the first failure"
    );
}

#[tokio::test]
async fn emit_ignores_successful_responder_answers_and_keeps_the_fixed_snapshot() {
    let ctx = Context::new();
    let late_hits = Arc::new(AtomicUsize::new(0));
    let registration_slot = Arc::new(Mutex::new(Some(ctx.clone())));
    let late_counter = late_hits.clone();
    let slot = registration_slot.clone();

    ctx.on::<Notify, _>(responder_sync(move |_, value| {
        if let Some(registration_ctx) = slot.lock().take() {
            let hits = late_counter.clone();
            registration_ctx
                .on::<Notify, _>(observer_sync(move |_, _| {
                    hits.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), Infallible>(())
                }))
                .unwrap();
        }
        Ok::<_, Infallible>(Some(value + 100))
    }))
    .unwrap();

    ctx.emit::<Notify>(Routing::Unscoped, 1).await.unwrap();
    assert_eq!(
        late_hits.load(Ordering::SeqCst),
        0,
        "a callback cannot grow the fixed emit snapshot"
    );
    ctx.emit::<Notify>(Routing::Unscoped, 2).await.unwrap();
    assert_eq!(
        late_hits.load(Ordering::SeqCst),
        1,
        "responder answers are ignored by notification"
    );
}

#[tokio::test]
async fn emit_parallel_preflights_the_complete_role_set_before_any_claim() {
    let ctx = Context::new();
    let hits = Arc::new(AtomicUsize::new(0));
    let callback_hits = hits.clone();
    let once = ctx
        .on_with::<Notify, _>(
            observer_sync(move |_, _| {
                callback_hits.fetch_add(1, Ordering::SeqCst);
                Ok::<(), Infallible>(())
            }),
            ListenerOptions::default().once(),
        )
        .unwrap();
    let incompatible = ctx
        .on::<Notify, _>(mapper_sync(|_, value| Ok::<_, Infallible>(value)))
        .unwrap();

    let error = ctx
        .emit_parallel::<Notify>(Routing::Unscoped, 1)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        DispatchError::IncompatibleRole {
            operation: cordis_core::event::EventOperation::EmitParallel,
            role: cordis_core::event::ListenerRole::Mapper,
        }
    ));
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "preflight failure claims no listener"
    );
    assert!(incompatible.remove());

    ctx.emit_parallel::<Notify>(Routing::Unscoped, 2)
        .await
        .unwrap();
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "the once occurrence survived failed preflight"
    );
    assert!(!once.remove());
}

#[tokio::test]
async fn emit_parallel_claims_the_complete_set_before_any_callback_can_remove_it() {
    let ctx = Context::new();
    let second_registration = Arc::new(Mutex::new(None::<ListenerRegistration>));
    let removed = Arc::new(AtomicUsize::new(usize::MAX));
    let second_hits = Arc::new(AtomicUsize::new(0));

    let registration_slot = second_registration.clone();
    let removed_result = removed.clone();
    ctx.on::<Notify, _>(observer_sync(move |_, _| {
        let registration = registration_slot
            .lock()
            .take()
            .expect("second control present");
        removed_result.store(usize::from(registration.remove()), Ordering::SeqCst);
        Ok::<(), Infallible>(())
    }))
    .unwrap();

    let hits = second_hits.clone();
    let second = ctx
        .on_with::<Notify, _>(
            observer_sync(move |_, _| {
                hits.fetch_add(1, Ordering::SeqCst);
                Ok::<(), Infallible>(())
            }),
            ListenerOptions::default().once(),
        )
        .unwrap();
    *second_registration.lock() = Some(second);

    ctx.emit_parallel::<Notify>(Routing::Unscoped, 1)
        .await
        .unwrap();
    assert_eq!(
        removed.load(Ordering::SeqCst),
        0,
        "the once occurrence was already claimed"
    );
    assert_eq!(
        second_hits.load(Ordering::SeqCst),
        1,
        "claimed work survives later removal"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_parallel_starts_all_awaits_all_and_reports_failures_in_listener_order() {
    let ctx = Context::new();
    let release_first = Arc::new(tokio::sync::Notify::new());
    let (second_done_tx, second_done_rx) = tokio::sync::oneshot::channel();
    let second_done = Arc::new(Mutex::new(Some(second_done_tx)));

    let release = release_first.clone();
    ctx.on::<Notify, _>(observer(move |_, _| {
        let release = release.clone();
        async move {
            release.notified().await;
            Err::<(), std::io::Error>(std::io::Error::other("first-failure"))
        }
    }))
    .unwrap();
    let done = second_done.clone();
    ctx.on::<Notify, _>(observer(move |_, _| {
        let done = done.clone();
        async move {
            if let Some(tx) = done.lock().take() {
                let _ = tx.send(());
            }
            panic!("second-panic");
            #[allow(unreachable_code)]
            Ok::<(), Infallible>(())
        }
    }))
    .unwrap();

    let emitter = ctx.clone();
    let dispatch =
        tokio::spawn(async move { emitter.emit_parallel::<Notify>(Routing::Unscoped, 2).await });
    let second_started = common::bounded(1_000, second_done_rx).await;
    if second_started.is_none() {
        release_first.notify_one();
        let _ = dispatch.await;
        panic!(
            "parallel notification did not start the second claimed listener while the first was blocked"
        );
    }
    assert!(
        !dispatch.is_finished(),
        "parallel completion must still await the blocked first listener"
    );
    release_first.notify_one();

    let DispatchError::Parallel(failures) = dispatch.await.unwrap().unwrap_err() else {
        panic!("parallel failures must use the Parallel phase")
    };
    let failures = failures.failures();
    assert_eq!(failures.len(), 2);
    assert_eq!(failures[0].diagnostic(), "first-failure");
    assert_eq!(failures[0].kind(), InvocationFailureKind::ReturnedError);
    assert_eq!(failures[1].kind(), InvocationFailureKind::Panic);
    assert!(failures[1].diagnostic().contains("second-panic"));
    let first_id = failures[0].registration_id().expect("listener correlation");
    let second_id = failures[1].registration_id().expect("listener correlation");
    assert_ne!(
        first_id, second_id,
        "each failure correlates its exact occurrence"
    );
}

#[tokio::test]
async fn emit_parallel_uses_parallel_even_for_one_failure() {
    let ctx = Context::new();
    ctx.on::<Notify, _>(observer_sync(|_, _| {
        Err::<(), std::io::Error>(std::io::Error::other("only-failure"))
    }))
    .unwrap();

    let DispatchError::Parallel(failures) = ctx
        .emit_parallel::<Notify>(Routing::Unscoped, 3)
        .await
        .unwrap_err()
    else {
        panic!("one parallel failure still uses DispatchError::Parallel")
    };
    assert_eq!(failures.failures().len(), 1);
    assert_eq!(failures.failures()[0].diagnostic(), "only-failure");
}

#[tokio::test]
async fn query_is_sequential_first_answer_fail_fast_and_owned_through_remove() {
    let ctx = Context::new();
    let observer_hits = Arc::new(AtomicUsize::new(0));
    let observer_count = observer_hits.clone();
    ctx.on::<Ask, _>(observer_sync(move |_, _| {
        observer_count.fetch_add(1, Ordering::SeqCst);
        Ok::<(), Infallible>(())
    }))
    .unwrap();

    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started_tx)));
    let release = Arc::new(tokio::sync::Notify::new());
    let started_callback = started.clone();
    let release_callback = release.clone();
    let registration = ctx
        .on::<Ask, _>(responder(move |_, value| {
            let started = started_callback.clone();
            let release = release_callback.clone();
            async move {
                if let Some(tx) = started.lock().take() {
                    let _ = tx.send(());
                }
                release.notified().await;
                Ok::<_, Infallible>(Some(value + 10))
            }
        }))
        .unwrap();

    let later_hits = Arc::new(AtomicUsize::new(0));
    let later = later_hits.clone();
    ctx.on::<Ask, _>(responder_sync(move |_, value| {
        later.fetch_add(1, Ordering::SeqCst);
        Ok::<_, Infallible>(Some(value + 100))
    }))
    .unwrap();

    let asker = ctx.clone();
    let query = tokio::spawn(async move { asker.query::<Ask>(Routing::Unscoped, 5).await });
    started_rx.await.unwrap();
    assert!(
        registration.remove(),
        "remove unregisters only future claims"
    );
    assert!(
        !query.is_finished(),
        "query owns the already-claimed callback through completion"
    );
    release.notify_one();
    assert_eq!(query.await.unwrap().unwrap(), QueryOutcome::Answer(15));
    assert_eq!(observer_hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        later_hits.load(Ordering::SeqCst),
        0,
        "query stops at the first explicit answer"
    );
}

#[tokio::test]
async fn query_fails_fast_with_exact_correlation() {
    let ctx = Context::new();
    ctx.on::<Ask, _>(responder_sync(|_, _| {
        Err::<Option<usize>, std::io::Error>(std::io::Error::other("query-failure"))
    }))
    .unwrap();
    let later_hits = Arc::new(AtomicUsize::new(0));
    let later = later_hits.clone();
    ctx.on::<Ask, _>(responder_sync(move |_, value| {
        later.fetch_add(1, Ordering::SeqCst);
        Ok::<_, Infallible>(Some(value))
    }))
    .unwrap();

    let DispatchError::Invocation(failure) =
        ctx.query::<Ask>(Routing::Unscoped, 9).await.unwrap_err()
    else {
        panic!("query listener failure must use the Invocation phase")
    };
    assert_eq!(failure.kind(), InvocationFailureKind::ReturnedError);
    assert_eq!(failure.diagnostic(), "query-failure");
    assert!(failure.registration_id().is_some());
    assert_eq!(
        later_hits.load(Ordering::SeqCst),
        0,
        "query fails before later listeners start"
    );
}

#[tokio::test]
async fn query_preserves_false_zero_empty_text_and_empty_collection_answers() {
    struct FalseAnswer;
    impl Event for FalseAnswer {
        const NAME: &'static str = "issue33/false";
        type Args = ();
        type Output = bool;
    }
    struct ZeroAnswer;
    impl Event for ZeroAnswer {
        const NAME: &'static str = "issue33/zero";
        type Args = ();
        type Output = usize;
    }
    struct EmptyText;
    impl Event for EmptyText {
        const NAME: &'static str = "issue33/empty-text";
        type Args = ();
        type Output = String;
    }
    struct EmptyCollection;
    impl Event for EmptyCollection {
        const NAME: &'static str = "issue33/empty-collection";
        type Args = ();
        type Output = Vec<usize>;
    }

    let ctx = Context::new();
    ctx.on::<FalseAnswer, _>(responder_sync(|_, ()| Ok::<_, Infallible>(Some(false))))
        .unwrap();
    ctx.on::<ZeroAnswer, _>(responder_sync(|_, ()| Ok::<_, Infallible>(Some(0))))
        .unwrap();
    ctx.on::<EmptyText, _>(responder_sync(|_, ()| {
        Ok::<_, Infallible>(Some(String::new()))
    }))
    .unwrap();
    ctx.on::<EmptyCollection, _>(responder_sync(|_, ()| {
        Ok::<_, Infallible>(Some(Vec::new()))
    }))
    .unwrap();

    assert_eq!(
        ctx.query::<FalseAnswer>(Routing::Unscoped, ())
            .await
            .unwrap(),
        QueryOutcome::Answer(false)
    );
    assert_eq!(
        ctx.query::<ZeroAnswer>(Routing::Unscoped, ())
            .await
            .unwrap(),
        QueryOutcome::Answer(0)
    );
    assert_eq!(
        ctx.query::<EmptyText>(Routing::Unscoped, ()).await.unwrap(),
        QueryOutcome::Answer(String::new())
    );
    assert_eq!(
        ctx.query::<EmptyCollection>(Routing::Unscoped, ())
            .await
            .unwrap(),
        QueryOutcome::Answer(Vec::new())
    );
}

#[derive(Debug)]
struct CountingError(Arc<AtomicUsize>);
impl fmt::Display for CountingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fetch_add(1, Ordering::SeqCst);
        f.write_str("normalized-once")
    }
}
impl std::error::Error for CountingError {}

#[tokio::test]
async fn returned_errors_future_panics_and_state_factory_panics_are_normalized_with_correlation() {
    let returned = Context::new();
    let renders = Arc::new(AtomicUsize::new(0));
    let count = renders.clone();
    returned
        .on::<Notify, _>(observer_sync(move |_, _| {
            Err::<(), _>(CountingError(count.clone()))
        }))
        .unwrap();
    let DispatchError::Invocation(returned_failure) = returned
        .emit::<Notify>(Routing::Unscoped, 1)
        .await
        .unwrap_err()
    else {
        panic!("returned error should be an invocation failure")
    };
    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "concrete error is rendered only at its typed adapter"
    );
    assert_eq!(
        returned_failure.kind(),
        InvocationFailureKind::ReturnedError
    );
    assert_eq!(returned_failure.diagnostic(), "normalized-once");
    assert!(returned_failure.registration_id().is_some());

    let future_panic = Context::new();
    future_panic
        .on::<Notify, _>(observer(|_, _| async move {
            tokio::task::yield_now().await;
            panic!("poll-panic");
            #[allow(unreachable_code)]
            Ok::<(), Infallible>(())
        }))
        .unwrap();
    let DispatchError::Invocation(poll_failure) = future_panic
        .emit::<Notify>(Routing::Unscoped, 1)
        .await
        .unwrap_err()
    else {
        panic!("future panic should be an invocation failure")
    };
    assert_eq!(poll_failure.kind(), InvocationFailureKind::Panic);
    assert!(poll_failure.diagnostic().contains("poll-panic"));
    assert!(poll_failure.registration_id().is_some());

    let factory_panic = Context::new();
    factory_panic
        .on::<Notify, _>(observer_sync(with_state(
            || -> usize { panic!("factory-panic") },
            |_, _state, _| Ok::<(), Infallible>(()),
        )))
        .unwrap();
    let DispatchError::Invocation(factory_failure) = factory_panic
        .emit::<Notify>(Routing::Unscoped, 1)
        .await
        .unwrap_err()
    else {
        panic!("factory panic should be an invocation failure")
    };
    assert_eq!(factory_failure.kind(), InvocationFailureKind::Panic);
    assert!(factory_failure.diagnostic().contains("factory-panic"));
    assert!(factory_failure.registration_id().is_some());
}

struct CloneReentry;
impl Event for CloneReentry {
    const NAME: &'static str = "issue55/clone-reentry";
    type Args = ReentrantArgs;
    type Output = ();
}

struct ReentrantArgs {
    ctx: Context,
    clones: Arc<AtomicUsize>,
}

impl Clone for ReentrantArgs {
    fn clone(&self) -> Self {
        let registration = self
            .ctx
            .on::<CloneReentry, _>(observer_sync(|_, _| Ok::<_, Infallible>(())))
            .expect("Args Clone may reenter Event registration");
        assert!(registration.remove());
        self.clones.fetch_add(1, Ordering::SeqCst);
        Self {
            ctx: self.ctx.clone(),
            clones: self.clones.clone(),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn event_argument_clone_can_reenter_event_registration() {
    let ctx = Context::new();
    let clones = Arc::new(AtomicUsize::new(0));
    let _listener = ctx
        .on::<CloneReentry, _>(observer_sync(|_, _| Ok::<_, Infallible>(())))
        .unwrap();
    let task_ctx = ctx.clone();
    let task_clones = clones.clone();
    let task = tokio::spawn(async move {
        task_ctx
            .emit::<CloneReentry>(
                Routing::Unscoped,
                ReentrantArgs {
                    ctx: task_ctx.clone(),
                    clones: task_clones,
                },
            )
            .await
    });
    common::bounded(5_000, task)
        .await
        .expect("Args Clone deadlocked Event bookkeeping")
        .unwrap()
        .unwrap();
    assert!(clones.load(Ordering::SeqCst) >= 1);
}
