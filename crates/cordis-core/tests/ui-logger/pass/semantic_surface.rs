use cordis_core::logger::{BufferExporter, Exporter, ExporterRegistration, LogRecord};
use cordis_core::{Context, Level, Logger};
use std::sync::Arc;

struct Sink;
impl Exporter for Sink { fn export(&self, _: &LogRecord) {} }

fn inspect(record: &LogRecord, logger: &Logger) {
    let _: u64 = record.sequence();
    let _: std::time::SystemTime = record.timestamp();
    let _: &str = record.channel();
    let _: Level = record.level();
    let _: &str = record.text();
    let _ = logger.name();
}

fn main() {
    let ctx = Context::new();
    let registration: ExporterRegistration = ctx.add_exporter(Arc::new(Sink)).unwrap();
    let _ = registration.remove();
    let _ = BufferExporter::new(1, Level::Debug).unwrap();
    let _ = inspect;
}
