wit_bindgen::generate!({
    path: "../../../../../wit",
    world: "cordis-plugin",
    async: true,
});

use exports::cordis::plugin::lifecycle::{Guest, LifecycleError};
use exports::cordis::plugin::manifest::{Descriptor, Guest as ManifestGuest, StandardCapability};
use cordis::plugin::configuration::get;
use cordis::plugin::diagnostics::{Level, emit};

struct Component;

impl Guest for Component {
    async fn activate() -> Result<(), LifecycleError> {
        let configuration = String::from_utf8(get().await).expect("fixture configuration is UTF-8");
        emit(Level::Info, format!("guest configuration: {configuration}")).await;
        emit(Level::Info, "guest lifecycle activated".into()).await;
        Ok(())
    }

    async fn dispose() -> Result<(), LifecycleError> {
        emit(Level::Info, "guest lifecycle disposed".into()).await;
        Ok(())
    }
}

impl ManifestGuest for Component {
    async fn describe() -> Descriptor {
        emit(Level::Debug, "guest manifest read".into()).await;
        Descriptor {
            name: "lifecycle-guest".into(),
            standard_capabilities: vec![
                StandardCapability::Configuration,
                StandardCapability::Diagnostics,
            ],
        }
    }
}

export!(Component);
