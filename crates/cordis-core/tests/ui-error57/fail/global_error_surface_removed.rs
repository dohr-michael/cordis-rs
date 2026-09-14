use cordis_core::{CordisError, CordisErrorCode, Result};
use cordis_core::error::CordisError as ModuleCordisError;

fn main() {
    let _ = CordisError::inactive();
    let _ = CordisErrorCode::InactiveEffect;
    let _: Result<()> = Ok(());
    let _ = std::mem::size_of::<ModuleCordisError>();
}
