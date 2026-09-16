//! Experimental Component Model integration for Cordis.
//!
//! One guest component instance belongs to one Cordis apply generation. The
//! adapter activates it during [`cordis_core::Plugin::apply`] and binds its
//! asynchronous guest cleanup to that generation through
//! [`cordis_core::effect`].

use std::sync::Arc;

use cordis_core::effect::EffectRegistrationError;
use cordis_core::{Context, Logger, Plugin};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

#[allow(
    missing_docs,
    reason = "Wasmtime generates bindings from the documented WIT contract"
)]
pub(crate) mod bindings {
    wasmtime::component::bindgen!({
        path: "../../wit",
        world: "cordis-plugin",
    });
}

use bindings::CordisPlugin;

/// Standard host capabilities available to guest Components.
mod capabilities;

pub use capabilities::ComponentEvent;

/// A reusable Component Model Plugin factory.
///
/// The factory owns one Wasmtime engine while each [`ComponentInput`] owns one
/// compiled guest artifact. Each call to [`Plugin::apply`] creates a fresh
/// guest instance for the current Cordis apply generation.
#[derive(Clone)]
pub struct ComponentPlugin {
    engine: Arc<Engine>,
}

impl ComponentPlugin {
    /// Construct an engine configured for async Component Model calls.
    pub fn new() -> Result<Self, EngineError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        let engine = Engine::new(&config).map_err(EngineError::Create)?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }
}

/// Failure while constructing a [`ComponentPlugin`] engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Wasmtime rejected the requested Component Model configuration.
    #[error("could not create the Component Model engine: {0}")]
    Create(#[source] wasmtime::Error),
}

/// Source bytes for one immutable guest component artifact.
#[derive(Clone)]
pub struct ComponentArtifact {
    bytes: Arc<[u8]>,
    configuration: Arc<[u8]>,
}

impl ComponentArtifact {
    /// Retain one component artifact's bytes for synchronous preparation.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            configuration: Arc::from([]),
        }
    }

    /// Attach immutable configuration for every generation using this artifact.
    pub fn with_configuration(mut self, bytes: impl Into<Arc<[u8]>>) -> Self {
        self.configuration = bytes.into();
        self
    }
}

/// A prepared, compiled component artifact.
#[derive(Clone)]
pub struct ComponentInput {
    component: Arc<Component>,
    configuration: Arc<[u8]>,
}

/// Failure while compiling a [`ComponentArtifact`] before lifecycle admission.
#[derive(Debug, Error)]
pub enum ComponentPrepareError {
    /// The artifact is not a valid component for the configured engine.
    #[error("could not compile the guest component: {0}")]
    Compile(#[source] wasmtime::Error),
}

/// Failure while applying one guest component generation.
#[derive(Debug, Error)]
pub enum ComponentApplyError {
    /// The component could not be instantiated against the minimal host.
    #[error("could not instantiate the guest component: {0}")]
    Instantiate(#[source] wasmtime::Error),
    /// The guest trapped while it was being activated.
    #[error("guest activation trapped: {0}")]
    ActivateTrap(#[source] wasmtime::Error),
    /// The guest rejected activation with its own diagnostic.
    #[error("guest activation rejected: {0}")]
    ActivateRejected(String),
    /// The guest component actor stopped before activation completed.
    #[error("the guest component actor stopped before activation")]
    ActorStopped,
    /// The guest trapped while its static manifest was being read.
    #[error("guest manifest trapped: {0}")]
    ManifestTrap(#[source] wasmtime::Error),
    /// The generation no longer admitted the cleanup obligation.
    #[error("could not bind guest cleanup to the Cordis generation: {0}")]
    CleanupRegistration(#[from] EffectRegistrationError),
}

/// A cleanup diagnostic reported after the host has claimed guest disposal.
#[derive(Debug, Error)]
enum ComponentDisposeError {
    #[error("guest disposal trapped: {0}")]
    Trap(#[source] wasmtime::Error),
    #[error("guest disposal rejected: {0}")]
    Rejected(String),
    #[error("the guest component actor stopped before disposal")]
    ActorStopped,
}

pub(crate) struct HostState {
    context: Context,
    table: ResourceTable,
    wasi: WasiCtx,
    logger: Logger,
    configuration: Arc<[u8]>,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

struct LiveComponent {
    store: Store<HostState>,
    bindings: CordisPlugin,
}

impl LiveComponent {
    async fn describe(&mut self) -> Result<(), ComponentApplyError> {
        self.store
            .run_concurrent(async |accessor| {
                self.bindings
                    .cordis_plugin_manifest()
                    .call_describe(accessor)
                    .await
            })
            .await
            .map_err(ComponentApplyError::ManifestTrap)?
            .map_err(ComponentApplyError::ManifestTrap)?;
        Ok(())
    }

    async fn activate(&mut self) -> Result<(), ComponentApplyError> {
        let outcome = self
            .store
            .run_concurrent(async |accessor| {
                self.bindings
                    .cordis_plugin_lifecycle()
                    .call_activate(accessor)
                    .await
            })
            .await
            .map_err(ComponentApplyError::ActivateTrap)?
            .map_err(ComponentApplyError::ActivateTrap)?;
        outcome.map_err(|error| ComponentApplyError::ActivateRejected(error.message))
    }

    async fn dispose(&mut self) -> Result<(), ComponentDisposeError> {
        let outcome = self
            .store
            .run_concurrent(async |accessor| {
                self.bindings
                    .cordis_plugin_lifecycle()
                    .call_dispose(accessor)
                    .await
            })
            .await
            .map_err(ComponentDisposeError::Trap)?
            .map_err(ComponentDisposeError::Trap)?;
        outcome.map_err(|error| ComponentDisposeError::Rejected(error.message))
    }
}

struct GenerationComponent {
    commands: mpsc::UnboundedSender<ComponentCommand>,
}

impl GenerationComponent {
    fn new(mut component: LiveComponent) -> Self {
        let (commands, mut receiver) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    ComponentCommand::Activate(response) => {
                        let _ = response.send(component.activate().await);
                    }
                    ComponentCommand::Dispose(response) => {
                        let _ = response.send(component.dispose().await);
                        break;
                    }
                }
            }
        });
        Self { commands }
    }

    async fn activate(&self) -> Result<(), ComponentApplyError> {
        let (send, receive) = oneshot::channel();
        self.commands
            .send(ComponentCommand::Activate(send))
            .map_err(|_| ComponentApplyError::ActorStopped)?;
        receive
            .await
            .map_err(|_| ComponentApplyError::ActorStopped)?
    }

    async fn dispose(&self) -> Result<(), ComponentDisposeError> {
        let (send, receive) = oneshot::channel();
        self.commands
            .send(ComponentCommand::Dispose(send))
            .map_err(|_| ComponentDisposeError::ActorStopped)?;
        receive
            .await
            .map_err(|_| ComponentDisposeError::ActorStopped)?
    }
}

enum ComponentCommand {
    Activate(oneshot::Sender<Result<(), ComponentApplyError>>),
    Dispose(oneshot::Sender<Result<(), ComponentDisposeError>>),
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
        Component::new(&self.engine, &artifact.bytes)
            .map(|component| ComponentInput {
                component: Arc::new(component),
                configuration: artifact.configuration,
            })
            .map_err(ComponentPrepareError::Compile)
    }

    async fn apply(&self, ctx: Context, input: &ComponentInput) -> Result<(), ComponentApplyError> {
        let mut linker = Linker::<HostState>::new(&self.engine);
        capabilities::add_to_linker(&mut linker).map_err(ComponentApplyError::Instantiate)?;
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)
            .map_err(ComponentApplyError::Instantiate)?;
        let mut store = Store::new(
            &self.engine,
            HostState {
                context: ctx.clone(),
                table: ResourceTable::new(),
                wasi: WasiCtx::builder().build(),
                logger: ctx.logger().with_name("wasm-component"),
                configuration: input.configuration.clone(),
            },
        );
        let bindings = CordisPlugin::instantiate_async(&mut store, &input.component, &linker)
            .await
            .map_err(ComponentApplyError::Instantiate)?;
        let mut live = LiveComponent { store, bindings };
        live.describe().await?;

        let generation = Arc::new(GenerationComponent::new(live));
        let cleanup_generation = generation.clone();
        if let Err(error) = ctx.effect(move || async move { cleanup_generation.dispose().await }) {
            let _ = generation.dispose().await;
            return Err(error.into());
        }

        generation.activate().await
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentArtifact, ComponentPlugin};
    use cordis_core::Plugin;

    #[test]
    fn malformed_artifact_is_rejected_before_lifecycle_admission() {
        let plugin = ComponentPlugin::new().expect("the local Wasmtime engine is constructible");
        let artifact = ComponentArtifact::from_bytes(b"not a component".to_vec());

        assert!(plugin.prepare(artifact).is_err());
    }
}
