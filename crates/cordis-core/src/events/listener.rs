//! Semantic Event listener adapters and invocation-local state composition.

use super::store::HookKind;
use super::types::{ErasedPayload, Event, InvocationFailure, ListenerRegistrationId, ListenerRole};
use crate::context::Context;
use crate::fiber::Fiber;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Weak};

#[doc(hidden)]
pub enum CallbackValue {
    Observer,
    Responder(Option<ErasedPayload>),
    Mapper(ErasedPayload),
}

#[doc(hidden)]
pub type CallbackFuture =
    Pin<Box<dyn Future<Output = Result<CallbackValue, InvocationFailure>> + Send>>;
#[doc(hidden)]
pub type ErasedListener = Arc<dyn Fn(ErasedPayload) -> CallbackFuture + Send + Sync>;
#[doc(hidden)]
pub type ErasedAroundListener = Arc<
    dyn Fn(
            ErasedPayload,
            NextFn,
        )
            -> Pin<Box<dyn Future<Output = Result<ErasedPayload, InvocationFailure>> + Send>>
        + Send
        + Sync,
>;

#[doc(hidden)]
pub type AroundFuture =
    Pin<Box<dyn Future<Output = Result<ErasedPayload, InvocationFailure>> + Send>>;
#[doc(hidden)]
pub type NextCallback = Box<dyn FnOnce(ErasedPayload) -> AroundFuture + Send>;

#[doc(hidden)]
pub struct NextFn(pub(crate) NextCallback);

impl NextFn {
    pub(crate) async fn call(
        self,
        payload: ErasedPayload,
    ) -> Result<ErasedPayload, InvocationFailure> {
        (self.0)(payload).await
    }
}

/// Consuming continuation handed to an Around listener.
pub struct Next<E: Event> {
    pub(crate) inner: NextFn,
    _marker: PhantomData<fn(E) -> E>,
}

impl<E: Event> Next<E> {
    pub(crate) fn new(inner: NextFn) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    /// Invoke the remaining waterfall chain exactly once.
    pub async fn call(self, args: E::Args) -> Result<E::Output, InvocationFailure> {
        let output = self.inner.call(Box::new(args)).await?;
        Ok(*output
            .downcast::<E::Output>()
            .expect("Event contract preflight guarantees Around output type"))
    }
}

/// Immutable registration options. Default is append, scoped, repeatable.
#[derive(Debug, Clone, Copy, Default)]
pub struct ListenerOptions {
    prepend: bool,
    global: bool,
    once: bool,
}

impl ListenerOptions {
    /// Place this occurrence before existing registrations in the same typed stream.
    #[must_use]
    pub const fn prepend(mut self) -> Self {
        self.prepend = true;
        self
    }
    /// Make this registration globally reachable instead of Scope-limited.
    #[must_use]
    pub const fn global(mut self) -> Self {
        self.global = true;
        self
    }
    /// Mark this registration as claim-at-most-once.
    #[must_use]
    pub const fn once(mut self) -> Self {
        self.once = true;
        self
    }
    /// Whether this registration is prepended.
    pub const fn is_prepend(self) -> bool {
        self.prepend
    }
    /// Whether this registration is globally reachable.
    pub const fn is_global(self) -> bool {
        self.global
    }
    /// Whether this registration is claim-at-most-once.
    pub const fn is_once(self) -> bool {
        self.once
    }
}

/// Move-only control for one exact listener registration occurrence.
pub struct ListenerRegistration {
    pub(crate) remove: Arc<dyn Fn(&ListenerRegistrationId) -> bool + Send + Sync>,
    pub(crate) id: ListenerRegistrationId,
    pub(crate) cleanup: crate::effect::DisposableToken,
    pub(crate) owner: Weak<Fiber>,
}

impl ListenerRegistration {
    /// Unregister this exact occurrence if it has not already been consumed or cleaned up.
    ///
    /// Removal controls only future invocation claims. Work that already won its claim
    /// remains framework-owned and runs to completion. Dropping this control is inert.
    pub fn remove(self) -> bool {
        let removed = (self.remove)(&self.id);
        let claimed_cleanup = self
            .owner
            .upgrade()
            .and_then(|fiber| fiber.remove_disposable(self.cleanup));
        drop(claimed_cleanup);
        removed
    }
}

mod sealed {
    use super::{Context, Event, HookKind};

    pub trait ListenerImpl<E: Event>: Send + Sync + 'static {
        fn into_hook(self, registration: Context) -> HookKind;
    }
}

/// Sealed, methodless semantic listener capability.
pub trait Listener<E: Event>: sealed::ListenerImpl<E> {}
impl<E: Event, T> Listener<E> for T where T: sealed::ListenerImpl<E> {}

/// Opaque invocation-local state composition.
pub struct StatefulCallback<F, C, S> {
    factory: F,
    callback: C,
    _state: PhantomData<fn() -> S>,
}

/// Compose a fresh invocation-local state factory with a role callback.
pub fn with_state<S, F, C>(factory: F, callback: C) -> StatefulCallback<F, C, S>
where
    F: Fn() -> S,
{
    StatefulCallback {
        factory,
        callback,
        _state: PhantomData,
    }
}

// `Next` may carry an already-normalized downstream failure through an Around
// callback. Preserve that identity; normalize only newly returned typed errors.
fn normalize_returned<Err>(error: Err) -> InvocationFailure
where
    Err: Error + 'static,
{
    let diagnostic = error.to_string();
    let erased: Box<dyn Error> = Box::new(error);
    match erased.downcast::<InvocationFailure>() {
        Ok(failure) => *failure,
        Err(_) => InvocationFailure::returned(diagnostic),
    }
}

fn downcast_args<E: Event>(payload: ErasedPayload) -> E::Args {
    *payload
        .downcast::<E::Args>()
        .expect("Event contract preflight guarantees listener Args type")
}

pub struct ObserverAdapter<C, const ASYNC: bool>(pub(crate) C);
struct ResponderAdapter<C, const ASYNC: bool>(C);
pub struct MapperAdapter<C, const ASYNC: bool, T>(pub(crate) C, PhantomData<fn(T) -> T>);
pub struct AroundAdapter<C, T>(pub(crate) C, PhantomData<fn(T) -> T>);

pub trait ObserverAsync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait ObserverSync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait ResponderAsync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait ResponderSync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait MapperAsync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait MapperSync<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedListener;
}
pub trait AroundCallback<E: Event>: Send + Sync + 'static {
    fn erase(self, registration: Context) -> ErasedAroundListener;
}

impl<E, F, Fut, Err> ObserverAsync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let fut = self(registration.clone(), downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|()| CallbackValue::Observer)
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, S, F, C, Fut, Err> ObserverAsync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let fut = callback(registration.clone(), state, downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|()| CallbackValue::Observer)
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, F, Err> ObserverSync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Result<(), Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let result = self(registration.clone(), downcast_args::<E>(payload))
                .map(|()| CallbackValue::Observer)
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, S, F, C, Err> ObserverSync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Result<(), Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let result = callback(registration.clone(), state, downcast_args::<E>(payload))
                .map(|()| CallbackValue::Observer)
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, F, Fut, Err> ResponderAsync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<E::Output>, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let fut = self(registration.clone(), downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|value| {
                        CallbackValue::Responder(value.map(|v| Box::new(v) as ErasedPayload))
                    })
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, S, F, C, Fut, Err> ResponderAsync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Option<E::Output>, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let fut = callback(registration.clone(), state, downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|value| {
                        CallbackValue::Responder(value.map(|v| Box::new(v) as ErasedPayload))
                    })
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, F, Err> ResponderSync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Result<Option<E::Output>, Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let result = self(registration.clone(), downcast_args::<E>(payload))
                .map(|value| CallbackValue::Responder(value.map(|v| Box::new(v) as ErasedPayload)))
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, S, F, C, Err> ResponderSync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Result<Option<E::Output>, Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let result = callback(registration.clone(), state, downcast_args::<E>(payload))
                .map(|value| CallbackValue::Responder(value.map(|v| Box::new(v) as ErasedPayload)))
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, F, Fut, Err> MapperAsync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<E::Args, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let fut = self(registration.clone(), downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|value| CallbackValue::Mapper(Box::new(value)))
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, S, F, C, Fut, Err> MapperAsync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<E::Args, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let fut = callback(registration.clone(), state, downcast_args::<E>(payload));
            Box::pin(async move {
                fut.await
                    .map(|value| CallbackValue::Mapper(Box::new(value)))
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, F, Err> MapperSync<E> for F
where
    E: Event,
    F: Fn(Context, E::Args) -> Result<E::Args, Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        Arc::new(move |payload| {
            let result = self(registration.clone(), downcast_args::<E>(payload))
                .map(|value| CallbackValue::Mapper(Box::new(value)))
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, S, F, C, Err> MapperSync<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args) -> Result<E::Args, Err> + Send + Sync + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload| {
            let state = factory();
            let result = callback(registration.clone(), state, downcast_args::<E>(payload))
                .map(|value| CallbackValue::Mapper(Box::new(value)))
                .map_err(normalize_returned);
            Box::pin(std::future::ready(result))
        })
    }
}

impl<E, F, Fut, Err> AroundCallback<E> for F
where
    E: Event,
    F: Fn(Context, E::Args, Next<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<E::Output, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedAroundListener {
        Arc::new(move |payload, next| {
            let fut = self(
                registration.clone(),
                downcast_args::<E>(payload),
                Next::new(next),
            );
            Box::pin(async move {
                fut.await
                    .map(|value| Box::new(value) as ErasedPayload)
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, S, F, C, Fut, Err> AroundCallback<E> for StatefulCallback<F, C, S>
where
    E: Event,
    S: 'static,
    F: Fn() -> S + Send + Sync + 'static,
    C: Fn(Context, S, E::Args, Next<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<E::Output, Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn erase(self, registration: Context) -> ErasedAroundListener {
        let StatefulCallback {
            factory, callback, ..
        } = self;
        Arc::new(move |payload, next| {
            let state = factory();
            let fut = callback(
                registration.clone(),
                state,
                downcast_args::<E>(payload),
                Next::new(next),
            );
            Box::pin(async move {
                fut.await
                    .map(|value| Box::new(value) as ErasedPayload)
                    .map_err(normalize_returned)
            })
        })
    }
}

impl<E, C> sealed::ListenerImpl<E> for ObserverAdapter<C, true>
where
    E: Event,
    C: ObserverAsync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Observer,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for ObserverAdapter<C, false>
where
    E: Event,
    C: ObserverSync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Observer,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for ResponderAdapter<C, true>
where
    E: Event,
    C: ResponderAsync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Responder,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for ResponderAdapter<C, false>
where
    E: Event,
    C: ResponderSync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Responder,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for MapperAdapter<C, true, E>
where
    E: Event + 'static,
    C: MapperAsync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Mapper,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for MapperAdapter<C, false, E>
where
    E: Event + 'static,
    C: MapperSync<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Plain {
            role: ListenerRole::Mapper,
            callback: self.0.erase(registration),
        }
    }
}
impl<E, C> sealed::ListenerImpl<E> for AroundAdapter<C, E>
where
    E: Event + 'static,
    C: AroundCallback<E>,
{
    fn into_hook(self, registration: Context) -> HookKind {
        HookKind::Around(self.0.erase(registration))
    }
}

impl<C, Fut, Err> crate::observation::runtime_observer_sealed::Sealed for ObserverAdapter<C, true>
where
    C: Fn(Context, crate::observation::RuntimeObservation) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
    Err: Error + 'static,
{
    fn into_callback(self) -> crate::observation::ObservationCallback {
        Arc::new(move |registration, record| {
            let future = self.0(registration, record);
            Box::pin(async move { future.await.map_err(|error| error.to_string()) })
        })
    }
}

impl<C, Err> crate::observation::runtime_observer_sealed::Sealed for ObserverAdapter<C, false>
where
    C: Fn(Context, crate::observation::RuntimeObservation) -> Result<(), Err>
        + Send
        + Sync
        + 'static,
    Err: Error + 'static,
{
    fn into_callback(self) -> crate::observation::ObservationCallback {
        Arc::new(move |registration, record| {
            let result = self.0(registration, record).map_err(|error| error.to_string());
            Box::pin(std::future::ready(result))
        })
    }
}

/// Adapt an asynchronous `Context, Args -> Result<(), _>` callback as an Observer.
pub fn observer<C>(callback: C) -> ObserverAdapter<C, true> {
    ObserverAdapter(callback)
}
/// Adapt a synchronous `Context, Args -> Result<(), _>` callback as an Observer.
pub fn observer_sync<C>(callback: C) -> ObserverAdapter<C, false> {
    ObserverAdapter(callback)
}
/// Adapt an asynchronous optional-answer callback as a Responder.
pub fn responder<E, C>(callback: C) -> impl Listener<E>
where
    E: Event,
    C: ResponderAsync<E>,
{
    ResponderAdapter::<C, true>(callback)
}
/// Adapt a synchronous optional-answer callback as a Responder.
pub fn responder_sync<E, C>(callback: C) -> impl Listener<E>
where
    E: Event,
    C: ResponderSync<E>,
{
    ResponderAdapter::<C, false>(callback)
}
/// Adapt an asynchronous owned-value callback as a Mapper role.
///
/// `T` may be an Event contract or a Plugin contract used by typed update control;
/// the receiving registration API supplies the corresponding sealed role bound.
pub fn mapper<T, C>(callback: C) -> MapperAdapter<C, true, T> {
    MapperAdapter(callback, PhantomData)
}
/// Adapt a synchronous owned-value callback as a Mapper role.
pub fn mapper_sync<T, C>(callback: C) -> MapperAdapter<C, false, T> {
    MapperAdapter(callback, PhantomData)
}
/// Adapt an asynchronous callback receiving a consuming continuation as an Around role.
pub fn around<T, C>(callback: C) -> AroundAdapter<C, T> {
    AroundAdapter(callback, PhantomData)
}

pub(crate) fn into_hook<E: Event, L: Listener<E>>(listener: L, registration: Context) -> HookKind {
    sealed::ListenerImpl::into_hook(listener, registration)
}
