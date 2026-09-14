use cordis_loader::resolver::PluginRequest;
fn main() {
    let _ = PluginRequest { resolve_key: "x", config: &serde_json::Value::Null, inject: &[] };
}
