use cordis_core::logger::{BufferExporter, ExporterRegistration};
use cordis_core::Level;
fn clone_registration(registration: ExporterRegistration) { let _ = registration.clone(); }
fn main() {
    let _ = BufferExporter::default();
    let _ = BufferExporter::new(1, Level::Info).unwrap().clone();
}
