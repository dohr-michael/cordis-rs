use std::convert::Infallible;
use cordis_core::PreparedPlugin;
use cordis_loader::{PluginResolver};
use cordis_loader::resolver::PluginRequest;
struct AsyncResolver;
impl PluginResolver for AsyncResolver {
    type Error = Infallible;
    async fn resolve(&self, _: PluginRequest<'_>) -> Result<Option<PreparedPlugin>, Self::Error> { Ok(None) }
}
fn main() {}
