use std::convert::Infallible;
use cordis_core::PreparedPlugin;
use cordis_loader::PluginResolver;

struct LegacyResolver;
impl PluginResolver for LegacyResolver {
    type Error = Infallible;
    fn resolve(&self, _key: &str) -> Option<PreparedPlugin> { None }
}
fn main() {}
