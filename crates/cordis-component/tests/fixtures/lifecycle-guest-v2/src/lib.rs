wit_bindgen::generate!({ path: "../../../../../wit", world: "cordis-plugin", async: true });

use cordis::plugin::diagnostics::{emit, Level};
use exports::cordis::plugin::lifecycle::{Guest, LifecycleError};

struct Component;

impl Guest for Component {
    async fn activate() -> Result<(), LifecycleError> {
        emit(Level::Info, "v2 activate".into()).await;
        Ok(())
    }

    async fn dispose() -> Result<(), LifecycleError> {
        emit(Level::Info, "v2 dispose".into()).await;
        Ok(())
    }
}

export!(Component);
