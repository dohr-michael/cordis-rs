//! Typed Event contracts, semantic listener roles, and explicit Scope routing.
//!
//! Event identity is Runtime-local name plus compatible `Args`/`Output`.
//! Listener callbacks select an explicit Observer, Responder, Mapper, or Around
//! role through adapters; routing is supplied independently to dispatch.

mod dispatch;
mod listener;
mod store;
mod types;

pub use crate::context::Scope;
pub use listener::{
    Listener, ListenerOptions, ListenerRegistration, Next, StatefulCallback, around, mapper,
    mapper_sync, observer, observer_sync, responder, responder_sync, with_state,
};
pub use types::{
    DispatchError, DispatchOutcomeKind, Event, EventOperation, InvocationFailure,
    InvocationFailureKind, ListenerRegistrationError, ListenerRegistrationId, ListenerRole,
    ParallelFailures, QueryOutcome, Routing,
};

pub(crate) use listener::{AroundAdapter, MapperAdapter};
pub(crate) use store::EventStore;
pub(crate) use types::{fresh_listener_registration_id, panic_message};
