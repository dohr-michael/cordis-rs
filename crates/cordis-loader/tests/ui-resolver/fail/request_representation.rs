use cordis_loader::resolver::PluginRequest;
fn leak(request: PluginRequest<'_>) {
    let _ = request.resolve_key;
    let _ = request.config;
    let _ = request.inject;
    let _ = request.entry_id();
    let _ = request.name();
    let _ = request.isolate();
    let _ = request.context();
    let _ = request.scope();
    let _ = request.realm();
    let _ = request.parent();
}
fn main() {}
