wit_bindgen::generate!({ path: "../../../../../wit", world: "cordis-plugin", async: true });

use exports::cordis::plugin::lifecycle::{Guest, LifecycleError};

struct Component;

impl Guest for Component {
    async fn activate() -> Result<(), LifecycleError> {
        Ok(())
    }

    async fn dispose() -> Result<(), LifecycleError> {
        loop {
            core::hint::spin_loop();
        }
    }
}

export!(Component);
