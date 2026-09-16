//! Integration coverage for a component guest lifecycle.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use cordis_component::{ComponentArtifact, ComponentPlugin};
use cordis_core::{Context, FiberState, Plugin, PreparedPlugin};

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
    );
    let input = plugin.prepare(artifact).expect("guest component prepares");
    let context = Context::new();

    let fiber = context
        .spawn(PreparedPlugin::from_input(plugin, input))
        .await
        .expect("guest component activates");
    assert_eq!(fiber.state(), FiberState::Active);

    fiber.restart().await.expect("guest component restarts");
    assert_eq!(fiber.state(), FiberState::Active);

    fiber.dispose().await.expect("guest component disposes");
    assert_eq!(fiber.state(), FiberState::Disposed);
}
