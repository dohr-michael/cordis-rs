//! Experimental Component Model integration for Cordis.
//!
//! One guest component instance belongs to one Cordis apply generation. The
//! adapter activates it during [`cordis_core::Plugin::apply`] and binds its
//! asynchronous guest cleanup to that generation through [`cordis_core::effect`].

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cordis_core::effect::EffectRegistrationError;
use cordis_core::{Context, Logger, Plugin};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use thiserror::Error;
#[allow(
    clippy::disallowed_types,
    reason = "a Component Store must remain exclusively borrowed across its async guest call"
)]
use tokio::sync::Mutex as TokioMutex;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub(crate) mod bindings {
    wasmtime::component::bindgen!({ path: "../../wit", world: "cordis-plugin" });
}

use bindings::CordisPlugin;

#[allow(
    clippy::disallowed_types,
    reason = "a Component Store must remain exclusively borrowed across its async guest call"
)]
type ComponentMutex<T> = TokioMutex<T>;

const EPOCH_TICK: Duration = Duration::from_millis(10);
const LIFECYCLE_DEADLINE_TICKS: u64 = 50;
const INITIALIZATION_DEADLINE_TICKS: u64 = 100_000;

/// Advances one engine's epoch so lifecycle calls have a wall-clock deadline.
///
/// Wasmtime can only interrupt a guest at an epoch check. The dedicated thread
/// provides those checks independently of Cordis's completion runtime; dropping
/// the final factory stops and joins it.
struct EpochTicker {
    running: Arc<AtomicBool>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl EpochTicker {
    fn start(engine: Arc<Engine>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();
        let join = thread::spawn(move || {
            while thread_running.load(Ordering::Acquire) {
                thread::sleep(EPOCH_TICK);
                engine.increment_epoch();
            }
        });
        Self {
            running,
            join: Mutex::new(Some(join)),
        }
    }
}

impl Drop for EpochTicker {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(join) = self.join.get_mut().take() {
            let _ = join.join();
        }
    }
}

/// The small host capability surface used by the lifecycle fixture.
mod capabilities;

/// A reusable Component Model Plugin factory.
#[derive(Clone)]
pub struct ComponentPlugin {
    engine: Arc<Engine>,
    linker: Arc<Linker<HostState>>,
    _epoch_ticker: Arc<EpochTicker>,
}

impl ComponentPlugin {
    /// Construct an engine configured for async Component Model calls.
    pub fn new() -> Result<Self, EngineError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.epoch_interruption(true);
        let engine = Arc::new(Engine::new(&config).map_err(EngineError::Create)?);
        let mut linker = Linker::new(&engine);
        capabilities::add_to_linker(&mut linker).map_err(EngineError::Linker)?;
        wasmtime_wasi::p2::add_to_linker_async(&mut linker).map_err(EngineError::Linker)?;
        let epoch_ticker = Arc::new(EpochTicker::start(engine.clone()));
        Ok(Self {
            engine,
            linker: Arc::new(linker),
            _epoch_ticker: epoch_ticker,
        })
    }
}

/// Failure while constructing a [`ComponentPlugin`] engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Wasmtime rejected the requested Component Model configuration.
    #[error("could not create the Component Model engine: {0}")]
    Create(#[source] wasmtime::Error),
    /// Wasmtime could not link the fixed host capability surface.
    #[error("could not link the Component Model host capabilities: {0}")]
    Linker(#[source] wasmtime::Error),
}

/// Source bytes and the claimed SHA-256 digest for one immutable guest artifact.
#[derive(Clone)]
pub struct ComponentArtifact {
    bytes: Arc<[u8]>,
    digest: [u8; 32],
}

impl ComponentArtifact {
    /// Retain one component artifact and derive its SHA-256 content digest.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
        let digest = Sha256::digest(&bytes).into();
        Self { bytes, digest }
    }

    /// Retain an artifact with an externally supplied digest for admission checks.
    pub fn with_digest(bytes: impl Into<Arc<[u8]>>, digest: [u8; 32]) -> Self {
        Self {
            bytes: bytes.into(),
            digest,
        }
    }
}

/// A prepared, compiled component artifact.
#[derive(Clone)]
pub struct ComponentInput {
    component: Arc<Component>,
}

/// Failure while compiling a [`ComponentArtifact`] before lifecycle admission.
#[derive(Debug, Error)]
pub enum ComponentPrepareError {
    /// The artifact bytes do not match their claimed digest.
    #[error("guest component artifact digest does not match its content")]
    DigestMismatch,
    /// The artifact is not a valid component for the configured engine.
    #[error("could not compile the guest component: {0}")]
    Compile(#[source] wasmtime::Error),
}

/// A guest lifecycle call failed after the host claimed the instance.
#[derive(Debug, Error)]
pub enum GuestCallError {
    /// Wasmtime reported a trap while invoking the guest lifecycle operation.
    #[error("guest lifecycle call trapped: {0}")]
    Trap(#[source] wasmtime::Error),
    /// The guest returned its structured lifecycle diagnostic.
    #[error("guest lifecycle call was rejected: {0}")]
    Rejected(String),
}

/// Failure while applying one guest component generation.
#[derive(Debug, Error)]
pub enum ComponentApplyError {
    /// The component could not be instantiated against the fixed host surface.
    #[error("could not instantiate the guest component: {0}")]
    Instantiate(#[source] wasmtime::Error),
    /// The guest failed while activation was in progress.
    #[error("guest activation failed: {0}")]
    Activate(#[source] GuestCallError),
    /// The generation no longer admitted the cleanup obligation.
    #[error("could not bind guest cleanup to the Cordis generation: {0}")]
    CleanupRegistration(#[from] EffectRegistrationError),
}

/// State supplied to the one diagnostics capability for an instance.
pub(crate) struct HostState {
    logger: Logger,
    table: ResourceTable,
    wasi: WasiCtx,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// One instantiated guest component. Access is serialized by its generation mutex.
struct LiveComponent {
    store: Store<HostState>,
    bindings: CordisPlugin,
}

impl LiveComponent {
    fn arm_deadline(&mut self) {
        self.store.set_epoch_deadline(LIFECYCLE_DEADLINE_TICKS);
        self.store.epoch_deadline_trap();
    }

    async fn activate(&mut self) -> Result<(), GuestCallError> {
        self.arm_deadline();
        let outcome = self
            .store
            .run_concurrent(async |accessor| {
                self.bindings
                    .cordis_plugin_lifecycle()
                    .call_activate(accessor)
                    .await
            })
            .await
            .map_err(GuestCallError::Trap)?
            .map_err(GuestCallError::Trap)?;
        outcome.map_err(|error| GuestCallError::Rejected(error.message))
    }

    async fn dispose(&mut self) -> Result<(), GuestCallError> {
        self.arm_deadline();
        let outcome = self
            .store
            .run_concurrent(async |accessor| {
                self.bindings
                    .cordis_plugin_lifecycle()
                    .call_dispose(accessor)
                    .await
            })
            .await
            .map_err(GuestCallError::Trap)?
            .map_err(GuestCallError::Trap)?;
        outcome.map_err(|error| GuestCallError::Rejected(error.message))
    }
}

async fn dispose_once(
    component: Arc<ComponentMutex<Option<LiveComponent>>>,
) -> Result<(), GuestCallError> {
    let mut component = component.lock().await;
    match component.take() {
        Some(mut component) => component.dispose().await,
        None => Ok(()),
    }
}

impl Plugin for ComponentPlugin {
    type Config = ComponentArtifact;
    type Input = ComponentInput;
    type PrepareError = ComponentPrepareError;
    type ApplyError = ComponentApplyError;

    fn prepare(
        &self,
        artifact: ComponentArtifact,
    ) -> Result<ComponentInput, ComponentPrepareError> {
        if Sha256::digest(&artifact.bytes).as_slice() != artifact.digest {
            return Err(ComponentPrepareError::DigestMismatch);
        }
        Component::new(&self.engine, &artifact.bytes)
            .map(|component| ComponentInput {
                component: Arc::new(component),
            })
            .map_err(ComponentPrepareError::Compile)
    }

    async fn apply(&self, ctx: Context, input: &ComponentInput) -> Result<(), ComponentApplyError> {
        let mut store = Store::new(
            &self.engine,
            HostState {
                logger: ctx.logger().with_name("wasm-component"),
                table: ResourceTable::new(),
                wasi: WasiCtx::builder().build(),
            },
        );
        // Instantiation is host work, not a lifecycle call. It must not inherit
        // Store's immediate epoch deadline before `LiveComponent::arm_deadline`.
        store.set_epoch_deadline(INITIALIZATION_DEADLINE_TICKS);
        let bindings = CordisPlugin::instantiate_async(&mut store, &input.component, &self.linker)
            .await
            .map_err(ComponentApplyError::Instantiate)?;
        let generation = Arc::new(ComponentMutex::new(Some(LiveComponent { store, bindings })));

        // Effects are committed before any future generation-owned callback. Cordis drains
        // them in reverse commit order, so this disposal runs after those callbacks are gone.
        let cleanup_generation = generation.clone();
        if let Err(error) =
            ctx.effect(move || async move { dispose_once(cleanup_generation).await })
        {
            let _ = dispose_once(generation).await;
            return Err(error.into());
        }
        let mut component = generation.lock().await;
        component
            .as_mut()
            .expect("cleanup cannot run before apply returns")
            .activate()
            .await
            .map_err(ComponentApplyError::Activate)
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentArtifact, ComponentPlugin, ComponentPrepareError};
    use cordis_core::Plugin;

    #[test]
    fn digest_mismatch_is_rejected_before_lifecycle_admission() {
        let plugin = ComponentPlugin::new().expect("the local Wasmtime engine is constructible");
        let artifact = ComponentArtifact::with_digest(b"not a component".to_vec(), [0; 32]);
        assert!(matches!(
            plugin.prepare(artifact),
            Err(ComponentPrepareError::DigestMismatch)
        ));
    }
}
