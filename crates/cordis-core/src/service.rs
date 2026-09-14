//! Services — typed, reactive dependency slots (mirrors `service.ts` +
//! `reflect.ts`).
//!
//! In cordis, services are values exposed on the context by name. The Rust
//! port makes the contract explicit: [`Service`] ties a type to a service
//! name; slots are stored per **Service realm** so Context derivation
//! ([`with_isolated_service`](crate::Context::with_isolated_service) /
//! [`with_service_realms`](crate::Context::with_service_realms))
//! privatizes them exactly like upstream isolation. Each exact
//! `(Service, ServiceRealm)` slot carries at most one current publication
//! occurrence. Loading occupation is invisible; Active publication is visible;
//! lifecycle close withdraws visibility before generation-owned cleanup removes
//! the physical row. Dependency targets use exact publication occurrence
//! identity rather than provider identity.

use crate::context::{Context, RealmKey, Root};
use crate::fiber::Fiber;
use parking_lot::Mutex;
use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};

/// Private, non-zero-sized token proving a realm's Runtime membership.
pub(crate) struct RealmMembership {
    _identity: u8,
}

impl RealmMembership {
    pub(crate) fn new() -> Self {
        Self { _identity: 0 }
    }
}

/// An opaque Runtime-local placement identity for exact Service slots.
///
/// A realm carries no Service name, hierarchy, fallback rule, Event routing,
/// lifecycle ownership, or textual rendezvous policy. Allocate one through
/// [`Context::new_service_realm`] and install mappings atomically through
/// [`Context::with_service_realms`]. Realms from different Runtimes always
/// compare unequal.
#[derive(Clone)]
pub struct ServiceRealm {
    membership: Arc<RealmMembership>,
    key: RealmKey,
}

impl ServiceRealm {
    pub(crate) fn new(membership: Arc<RealmMembership>, key: RealmKey) -> Self {
        Self { membership, key }
    }

    pub(crate) fn belongs_to(&self, membership: &Arc<RealmMembership>) -> bool {
        Arc::ptr_eq(&self.membership, membership)
    }

    pub(crate) fn key(&self) -> RealmKey {
        self.key
    }
}

impl std::fmt::Debug for ServiceRealm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ServiceRealm(..)")
    }
}

impl PartialEq for ServiceRealm {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && Arc::ptr_eq(&self.membership, &other.membership)
    }
}

impl Eq for ServiceRealm {}

impl Hash for ServiceRealm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.membership), state);
        self.key.hash(state);
    }
}

/// A complete Service-realm mapping batch was invalid.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RealmMappingError {
    /// One Service name occurred more than once in the batch.
    #[error("service `{service}` is mapped more than once")]
    DuplicateService {
        /// The duplicated Service name.
        service: String,
    },
    /// A mapping used a realm allocated by another Runtime.
    #[error("service `{service}` uses a realm from another Runtime")]
    ForeignRealm {
        /// The Service name carrying the foreign realm.
        service: String,
    },
}

/// Marker trait linking a Rust type to one Runtime-local named Service contract.
pub trait Service: Send + Sync + 'static {
    /// The Service contract name.
    const NAME: &'static str;
}

/// A Service contract that owns typed source preparation and layer composition.
pub trait ConfigurableService: Service {
    /// Consumer-facing source configuration.
    type Config;
    /// One prepared configuration layer retained by the framework.
    type Layer: Send + Sync + 'static;
    /// The complete composed configuration delivered to the operation.
    type Resolved: Send + 'static;
    /// Failure while preparing source configuration.
    type PrepareError: std::error::Error;
    /// Failure while composing prepared layers.
    type ComposeError: std::error::Error;

    /// Validate and prepare one source configuration synchronously.
    fn prepare_config(config: Self::Config)
    -> std::result::Result<Self::Layer, Self::PrepareError>;

    /// Compose the optional base, ordered intercept layers, and optional head.
    fn compose_config<'a>(
        base: Option<&'a Self::Layer>,
        layers: impl IntoIterator<Item = &'a Self::Layer>,
        head: Option<&'a Self::Layer>,
    ) -> std::result::Result<Self::Resolved, Self::ComposeError>;
}

/// Failure to resolve one ConfigurableService's complete typed configuration.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigResolutionError<E: std::error::Error> {
    /// A same-name prepared layer belongs to another Service contract.
    #[error("configuration for service `{service}` belongs to a different contract")]
    ContractMismatch {
        /// The conflicting Service name.
        service: &'static str,
    },
    /// The Service contract rejected composition.
    #[error("service configuration composition failed: {0}")]
    Compose(E),
}

/// Exact visible lookup failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceLookupError {
    /// The exact selected realm has no visible publication.
    #[error("service `{service}` is unavailable")]
    Unavailable {
        /// The requested Service name.
        service: &'static str,
    },
    /// The Runtime already binds this name to another typed contract.
    #[error("service `{service}` belongs to a different contract")]
    ContractMismatch {
        /// The conflicting Service name.
        service: &'static str,
    },
}

/// Service publication admission failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServicePublishError {
    /// The current Context generation no longer admits registrations.
    #[error("the current Context generation is closed")]
    InactiveContext,
    /// The exact slot already has an eligible current publication.
    #[error("service `{service}` already has a publication in this realm")]
    DuplicatePublication {
        /// The occupied Service name.
        service: &'static str,
    },
    /// The Runtime already binds this name to another typed contract.
    #[error("service `{service}` belongs to a different contract")]
    ContractMismatch {
        /// The conflicting Service name.
        service: &'static str,
    },
}

/// Failure to control one exact Service publication occurrence.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceControlError {
    /// The capability no longer names the current occurrence in its exact slot.
    #[error("service `{service}` publication is stale")]
    StalePublication {
        /// The Service whose exact publication is no longer current.
        service: &'static str,
    },
    /// The exact occurrence is still current, but its generation no longer admits mutation.
    #[error("service `{service}` publication mutation is closed")]
    MutationClosed {
        /// The Service whose current publication can no longer be mutated.
        service: &'static str,
    },
}

/// Opaque identity of one successful Service publication occurrence.
///
/// Equality is allocation identity: every replacement receives a fresh value,
/// so stale cleanup can never compare equal to a later publication.
#[derive(Clone)]
pub(crate) struct ServiceOccurrenceId(Arc<u8>);

impl ServiceOccurrenceId {
    pub(crate) fn fresh() -> Self {
        Self(Arc::new(0))
    }
}

impl PartialEq for ServiceOccurrenceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ServiceOccurrenceId {}
impl std::hash::Hash for ServiceOccurrenceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl std::fmt::Debug for ServiceOccurrenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServiceOccurrenceId(..)")
    }
}

struct ServiceSlot {
    occurrence: ServiceOccurrenceId,
    value: Arc<dyn Any + Send + Sync>,
    contract: std::any::TypeId,
    fiber: Weak<Fiber>,
}

#[derive(Default)]
struct ServiceState {
    /// A Service name binds one typed contract for the Runtime lifetime.
    contracts: HashMap<&'static str, std::any::TypeId>,
    /// Exactly one current occupied occurrence per `(Service, realm)` slot.
    slots: HashMap<(RealmKey, &'static str), ServiceSlot>,
}

enum ServiceInstallRefusal {
    Duplicate,
    ContractMismatch,
}

struct ServiceSlotPublish<'a> {
    store: &'a ServiceStore,
    key: RealmKey,
    name: &'static str,
    slot: Option<ServiceSlot>,
    evicted: Option<ServiceSlot>,
    refusal: Option<ServicePublishError>,
}

impl crate::gated::PublishStep for ServiceSlotPublish<'_> {
    fn publish(&mut self) -> std::result::Result<(), crate::gated::PublishRefused> {
        let slot = self.slot.take().expect("publish runs once");
        match self.store.install(slot, self.key, self.name) {
            Ok(evicted) => {
                self.evicted = evicted;
                Ok(())
            }
            Err((refusal, slot)) => {
                self.slot = Some(slot);
                self.refusal = Some(match refusal {
                    ServiceInstallRefusal::Duplicate => {
                        ServicePublishError::DuplicatePublication { service: self.name }
                    }
                    ServiceInstallRefusal::ContractMismatch => {
                        ServicePublishError::ContractMismatch { service: self.name }
                    }
                });
                // The gated seam needs only an abort signal here. The exact
                // operation error is retained privately on this publish step
                // and returned by Context::provide after every lock is released.
                Err(crate::gated::PublishRefused)
            }
        }
    }
}

/// Runtime-local Service contract binding and exact-slot occurrence store.
#[derive(Default)]
pub(crate) struct ServiceStore {
    state: Mutex<ServiceState>,
}

impl ServiceStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn install(
        &self,
        slot: ServiceSlot,
        key: RealmKey,
        name: &'static str,
    ) -> std::result::Result<Option<ServiceSlot>, (ServiceInstallRefusal, ServiceSlot)> {
        let mut state = self.state.lock();
        if let Some(contract) = state.contracts.get(name) {
            if *contract != slot.contract {
                return Err((ServiceInstallRefusal::ContractMismatch, slot));
            }
        } else {
            state.contracts.insert(name, slot.contract);
        }

        if let Some(existing) = state.slots.get(&(key, name))
            && existing
                .fiber
                .upgrade()
                .is_some_and(|fiber| fiber.is_alive())
        {
            return Err((ServiceInstallRefusal::Duplicate, slot));
        }
        let evicted = state.slots.remove(&(key, name));
        state.slots.insert((key, name), slot);
        Ok(evicted)
    }

    fn visible(slot: &ServiceSlot) -> bool {
        slot.fiber.upgrade().is_some_and(|fiber| {
            fiber.is_alive() && fiber.state() == crate::fiber::FiberState::Active
        })
    }

    pub(crate) fn occurrence_id(&self, key: &RealmKey, name: &str) -> Option<ServiceOccurrenceId> {
        let state = self.state.lock();
        let slot = state.slots.get(&(*key, name))?;
        Self::visible(slot).then(|| slot.occurrence.clone())
    }

    fn visible_value<S: Service>(
        &self,
        key: RealmKey,
    ) -> std::result::Result<Arc<S>, ServiceLookupError> {
        let value = {
            let state = self.state.lock();
            match state.contracts.get(S::NAME) {
                Some(contract) if *contract != std::any::TypeId::of::<S>() => {
                    return Err(ServiceLookupError::ContractMismatch { service: S::NAME });
                }
                _ => {}
            }
            let Some(slot) = state
                .slots
                .get(&(key, S::NAME))
                .filter(|slot| Self::visible(slot))
            else {
                return Err(ServiceLookupError::Unavailable { service: S::NAME });
            };
            slot.value.clone()
        };
        value
            .downcast::<S>()
            .map_err(|_| ServiceLookupError::ContractMismatch { service: S::NAME })
    }

    fn mutation_state(owner: &Weak<Fiber>) -> Option<crate::fiber::FiberState> {
        let fiber = owner.upgrade()?;
        if !fiber.is_alive() {
            return None;
        }
        let state = fiber.state();
        matches!(
            state,
            crate::fiber::FiberState::Loading | crate::fiber::FiberState::Active
        )
        .then_some(state)
    }

    fn set_exact(
        &self,
        key: RealmKey,
        name: &'static str,
        occurrence: &ServiceOccurrenceId,
        owner: &Weak<Fiber>,
        value: Arc<dyn Any + Send + Sync>,
    ) -> std::result::Result<
        Arc<dyn Any + Send + Sync>,
        (ServiceControlError, Arc<dyn Any + Send + Sync>),
    > {
        let mut state = self.state.lock();
        let Some(slot) = state.slots.get_mut(&(key, name)) else {
            return Err((
                ServiceControlError::StalePublication { service: name },
                value,
            ));
        };
        if slot.occurrence != *occurrence {
            return Err((
                ServiceControlError::StalePublication { service: name },
                value,
            ));
        }
        if Self::mutation_state(owner).is_none() {
            return Err((ServiceControlError::MutationClosed { service: name }, value));
        }
        Ok(std::mem::replace(&mut slot.value, value))
    }

    fn remove_exact(
        &self,
        key: RealmKey,
        name: &'static str,
        occurrence: &ServiceOccurrenceId,
        owner: &Weak<Fiber>,
    ) -> std::result::Result<(ServiceSlot, bool), ServiceControlError> {
        let mut state = self.state.lock();
        let entry = match state.slots.entry((key, name)) {
            std::collections::hash_map::Entry::Occupied(entry)
                if entry.get().occurrence == *occurrence =>
            {
                entry
            }
            _ => return Err(ServiceControlError::StalePublication { service: name }),
        };
        let Some(owner_state) = Self::mutation_state(owner) else {
            return Err(ServiceControlError::MutationClosed { service: name });
        };
        let was_visible = owner_state == crate::fiber::FiberState::Active;
        Ok((entry.remove(), was_visible))
    }

    fn withdraw_exact(&self, key: RealmKey, name: &'static str, occurrence: &ServiceOccurrenceId) {
        let removed = {
            let mut state = self.state.lock();
            match state.slots.entry((key, name)) {
                std::collections::hash_map::Entry::Occupied(entry)
                    if entry.get().occurrence == *occurrence =>
                {
                    Some(entry.remove())
                }
                _ => None,
            }
        };
        // The slot contains an Arc<S>; destroy it only after bookkeeping
        // synchronization is released.
        drop(removed);
    }

    /// Exact occupied slots owned by `fiber`. Called immediately after a
    /// lifecycle state transition; because visibility is exactly Active,
    /// entering or leaving Active makes precisely these slots change visibility.
    pub(crate) fn slots_owned_by(&self, fiber: &Arc<Fiber>) -> Vec<(String, RealmKey)> {
        let owner = Arc::downgrade(fiber);
        self.state
            .lock()
            .slots
            .iter()
            .filter(|(_, slot)| Weak::ptr_eq(&slot.fiber, &owner))
            .map(|((key, name), _)| ((*name).to_owned(), *key))
            .collect()
    }

    pub(crate) fn visibility_owned_by(
        &self,
        fiber: &Arc<Fiber>,
    ) -> Vec<ServiceVisibilityOccurrence> {
        let owner = Arc::downgrade(fiber);
        self.state
            .lock()
            .slots
            .iter()
            .filter(|(_, slot)| Weak::ptr_eq(&slot.fiber, &owner))
            .map(|((key, name), slot)| ServiceVisibilityOccurrence {
                service: (*name).to_owned(),
                realm: *key,
                occurrence: slot.occurrence.clone(),
            })
            .collect()
    }

    pub(crate) fn snapshot_occurrences(&self) -> Vec<ServiceOccurrenceSnapshot> {
        let state = self.state.lock();
        state
            .slots
            .iter()
            .filter_map(|((realm, service), slot)| {
                let fiber = slot.fiber.upgrade()?;
                let fiber_state = fiber.state();
                // Current publication truth follows the same generation gate as
                // mutation/registration. A restart/dispose closes that gate
                // synchronously before the later Unloading publication, so an
                // Active-looking physical row can already be stale.
                let current = fiber.assert_can_register().is_ok();
                current.then(|| ServiceOccurrenceSnapshot {
                    id: slot.occurrence.clone(),
                    service: (*service).to_owned(),
                    realm: *realm,
                    provider: fiber.id().clone(),
                    visible: fiber_state == crate::fiber::FiberState::Active,
                })
            })
            .collect()
    }
}

pub(crate) struct ServiceVisibilityOccurrence {
    pub(crate) service: String,
    pub(crate) realm: RealmKey,
    pub(crate) occurrence: ServiceOccurrenceId,
}

pub(crate) struct ServiceOccurrenceSnapshot {
    pub(crate) id: ServiceOccurrenceId,
    pub(crate) service: String,
    pub(crate) realm: RealmKey,
    pub(crate) provider: crate::fiber::FiberId,
    pub(crate) visible: bool,
}

pub(crate) fn notify_dependents_for_edges(root: &Arc<Root>, edges: &[(String, RealmKey)]) {
    let affected = root.dependents_for_edges(edges);
    // Commit durable protocol truth for the complete deduplicated affected set
    // before any executor-dependent work starts. Kicks are acceleration only.
    for fiber in &affected {
        fiber.slot.commit_recheck();
    }
    for fiber in affected {
        fiber.kick_committed_recheck(root);
    }
}

pub(crate) fn notify_dependents_of_service(root: &Arc<Root>, name: &str, key: RealmKey) {
    notify_dependents_for_edges(root, &[(name.to_owned(), key)]);
}

/// Move-only opaque capability naming one exact Service publication occurrence.
///
/// The capability is the sole mutation authority for the occurrence that created
/// it. Dropping it is inert: the owning generation retains its cleanup claim.
#[must_use = "dropping a ServicePublication leaves generation-owned publication cleanup armed"]
pub struct ServicePublication<S: Service> {
    root: Arc<Root>,
    key: RealmKey,
    occurrence: ServiceOccurrenceId,
    owner: Weak<Fiber>,
    cleanup: crate::effect::DisposableToken,
    _service: std::marker::PhantomData<fn() -> S>,
}

impl<S: Service> ServicePublication<S> {
    /// Replace only this exact publication occurrence's payload.
    ///
    /// Publication identity is preserved, so a successful set does not create
    /// dependency-target drift. Exact staleness is checked before generation
    /// closure. Rejected incoming values and replaced outgoing values are
    /// destroyed only after Service synchronization is released.
    pub fn set(&self, value: Arc<S>) -> std::result::Result<(), ServiceControlError> {
        let erased: Arc<dyn Any + Send + Sync> = value;
        match self
            .root
            .services
            .set_exact(self.key, S::NAME, &self.occurrence, &self.owner, erased)
        {
            Ok(outgoing) => {
                drop(outgoing);
                Ok(())
            }
            Err((error, rejected)) => {
                drop(rejected);
                Err(error)
            }
        }
    }

    /// Consume this capability and remove only its exact publication occurrence.
    ///
    /// A successful visible withdrawal creates dependency drift once. The
    /// generation-owned cleanup claim is disarmed when still available; if the
    /// generation drain already claimed it, its exact cleanup becomes a stale
    /// no-op. Stale identity is reported before a closed-generation error.
    pub fn remove(self) -> std::result::Result<(), ServiceControlError> {
        let (removed, was_visible) =
            self.root
                .services
                .remove_exact(self.key, S::NAME, &self.occurrence, &self.owner)?;

        let claimed_cleanup = self
            .owner
            .upgrade()
            .and_then(|fiber| fiber.remove_disposable(self.cleanup));

        if was_visible {
            notify_dependents_of_service(&self.root, S::NAME, self.key);
            self.root.observations.publish(
                crate::observation::RuntimeObservation::ServiceVisibility {
                    service: S::NAME.to_owned(),
                    realm: ServiceRealm::new(self.root.realm_membership.clone(), self.key),
                    previous: Some(crate::observation::ServicePublicationId(
                        self.occurrence.clone(),
                    )),
                    current: None,
                },
            );
        }

        // Both values can own user-authored Service payloads or captures. Every
        // framework lock is already released before either destruction occurs.
        drop(claimed_cleanup);
        drop(removed);
        Ok(())
    }
}

impl Context {
    /// Publish one generation-owned occurrence into the exact Service slot
    /// selected by this Context's isolate axis.
    pub fn provide<S: Service>(
        &self,
        value: Arc<S>,
    ) -> std::result::Result<ServicePublication<S>, ServicePublishError> {
        let fiber = self.fiber().clone();
        fiber
            .assert_can_register()
            .map_err(|_| ServicePublishError::InactiveContext)?;
        let key = self.isolate_key(S::NAME);
        let occurrence = ServiceOccurrenceId::fresh();
        let cleanup_root = self.root.clone();
        let cleanup_occurrence = occurrence.clone();
        let mut step = ServiceSlotPublish {
            store: &self.root.services,
            key,
            name: S::NAME,
            slot: Some(ServiceSlot {
                occurrence: occurrence.clone(),
                value,
                contract: std::any::TypeId::of::<S>(),
                fiber: Arc::downgrade(&fiber),
            }),
            evicted: None,
            refusal: None,
        };
        let committed = crate::gated::push_gated(
            &fiber,
            crate::effect::sync_cleanup(move || {
                // Gate close already made an Active occurrence invisible and
                // emitted its drift. Physical stale cleanup is identity-checked
                // and semantically inert.
                cleanup_root
                    .services
                    .withdraw_exact(key, S::NAME, &cleanup_occurrence);
            }),
            &mut step,
        );
        let cleanup = match committed {
            Ok(cleanup) => cleanup,
            Err(_) => {
                return Err(step
                    .refusal
                    .take()
                    .unwrap_or(ServicePublishError::InactiveContext));
            }
        };
        drop(step);

        // Loading installation is occupied but invisible. Active late/root
        // publication is immediately visible and therefore creates drift.
        if fiber.state() == crate::fiber::FiberState::Active {
            notify_dependents_of_service(&self.root, S::NAME, key);
            self.root.observations.publish(
                crate::observation::RuntimeObservation::ServiceVisibility {
                    service: S::NAME.to_owned(),
                    realm: ServiceRealm::new(self.root.realm_membership.clone(), key),
                    previous: None,
                    current: Some(crate::observation::ServicePublicationId(occurrence.clone())),
                },
            );
        }

        Ok(ServicePublication {
            root: self.root.clone(),
            key,
            occurrence,
            owner: Arc::downgrade(&fiber),
            cleanup,
            _service: std::marker::PhantomData,
        })
    }

    /// Resolve only the exact Service slot selected by this Context's isolate
    /// axis. InjectSpec membership is irrelevant and no fallback/provider
    /// predicate is consulted.
    pub fn try_service<S: Service>(&self) -> std::result::Result<Arc<S>, ServiceLookupError> {
        let key = self.isolate_key(S::NAME);
        self.root.services.visible_value::<S>(key)
    }
}
