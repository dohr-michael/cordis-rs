use cordis_core::logger::LogRecord;
fn inspect(record: &LogRecord) {
    let _ = record.sn;
    let _ = record.ts;
    let _ = &record.name;
    let _ = &record.text;
}
fn main() {}
