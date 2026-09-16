wit_bindgen::generate!({
    path: "../../../../../wit",
    world: "cordis-plugin",
    async: true,
});

use exports::cordis::plugin::lifecycle::{Guest, LifecycleError};
use cordis::plugin::diagnostics::{Level, emit};

struct Component;

impl Guest for Component {
    async fn activate() -> Result<(), LifecycleError> {
        emit(Level::Info, "guest lifecycle activated".into()).await;
        Ok(())
    }

    async fn dispose() -> Result<(), LifecycleError> {
        emit(Level::Info, "guest lifecycle disposed".into()).await;
        Ok(())
    }
}

export!(Component);
