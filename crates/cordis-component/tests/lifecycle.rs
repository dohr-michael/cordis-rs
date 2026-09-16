//! Integration coverage for a component guest lifecycle.

use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};

use cordis_component::{ComponentArtifact, ComponentEvent, ComponentPlugin, HostEvent};
use cordis_core::event::observer_sync;
use cordis_core::logger::BufferExporter;
use cordis_core::{Context, FiberState, Level, Plugin, PreparedPlugin, Routing};
use parking_lot::Mutex;

fn guest_component() -> &'static Path {
    static COMPONENT: OnceLock<PathBuf> = OnceLock::new();
    COMPONENT
        .get_or_init(|| {
            let fixture =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lifecycle-guest");
            let status = Command::new(env!("CARGO"))
                .args([
                    "build",
                    "--locked",
                    "--manifest-path",
                    fixture
                        .join("Cargo.toml")
                        .to_str()
                        .expect("UTF-8 fixture path"),
                    "--target",
                    "wasm32-wasip2",
                ])
                .status()
                .expect("guest component build starts");
            assert!(status.success(), "guest component build succeeds");
            fixture.join("target/wasm32-wasip2/debug/cordis_component_lifecycle_guest.wasm")
        })
        .as_path()
}

#[tokio::test]
async fn component_guest_activates_and_disposes_with_its_fiber() {
    let plugin = ComponentPlugin::new().expect("the local Wasmtime engine is constructible");
    let artifact = ComponentArtifact::from_bytes(
        std::fs::read(guest_component()).expect("compiled guest component is readable"),
    )
    .with_configuration(b"revision=v1".to_vec());
    let input = plugin.prepare(artifact).expect("guest component prepares");
    let context = Context::new();
    let logs = Arc::new(BufferExporter::new(16, Level::Debug).expect("valid log buffer"));
    let _logs = context
        .add_exporter(logs.clone())
        .expect("root admits diagnostic exporter");
    let events = Arc::new(Mutex::new(Vec::new()));
    let observed_events = events.clone();
    let _events = context
        .on::<ComponentEvent, _>(observer_sync(move |_, event| {
            observed_events.lock().push(event);
            Ok::<_, Infallible>(())
        }))
        .expect("root admits Component event observer");

    let fiber = context
        .spawn(PreparedPlugin::from_input(plugin, input))
        .await
        .expect("guest component activates");
    assert_eq!(fiber.state(), FiberState::Active);
    context
        .emit::<HostEvent>(Routing::Unscoped, HostEvent::new("fixture/inbound", []))
        .await
        .expect("guest receives host event");

    fiber.restart().await.expect("guest component restarts");
    assert_eq!(fiber.state(), FiberState::Active);
    context
        .emit::<HostEvent>(Routing::Unscoped, HostEvent::new("fixture/inbound", []))
        .await
        .expect("restarted guest receives host event");

    fiber.dispose().await.expect("guest component disposes");
    assert_eq!(fiber.state(), FiberState::Disposed);

    let records = logs.snapshot();
    assert_eq!(
        records
            .iter()
            .filter(|record| record.text() == "guest lifecycle activated")
            .count(),
        2,
        "each apply generation forwarded its activation diagnostic"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.text() == "guest received: fixture/inbound")
            .count(),
        2
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.text() == "guest configuration: revision=v1")
            .count(),
        2,
        "each apply generation receives the artifact configuration"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.text() == "guest manifest read")
            .count(),
        2,
        "each generation is described before activation"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.text() == "guest lifecycle disposed")
            .count(),
        2,
        "each generation forwarded exactly one disposal diagnostic"
    );
    assert!(
        records
            .iter()
            .all(|record| record.channel() == "wasm-component"),
        "the host, not the guest, owns the logger channel"
    );
    assert_eq!(
        *events.lock(),
        vec![
            ComponentEvent::new("fixture/activated", b"revision=v1".to_vec()),
            ComponentEvent::new("fixture/activated", b"revision=v1".to_vec()),
        ],
        "guest events reach native Cordis observers for each generation"
    );
}
