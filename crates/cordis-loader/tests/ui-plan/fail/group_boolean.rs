use cordis_loader::PluginEntry;
fn main() {
    let _ = PluginEntry {
        key: Some("worker".into()), name: None, config: serde_json::Value::Null,
        disabled: false, inject: vec![], isolate: vec![], group: true,
    };
}
