use cordis_core::logger::{ExporterId, Message};
use cordis_core::Context;
fn main() {
    let ctx = Context::new();
    let id: ExporterId = panic!();
    let _ = ctx.remove_exporter(id);
    let _: Option<Message> = None;
}
