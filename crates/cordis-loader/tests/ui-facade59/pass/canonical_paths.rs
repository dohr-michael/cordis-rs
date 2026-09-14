use cordis_loader::{EntryGroup, EntryId, LoadOutcome, LoadPlan, LoadPlanBuilder, PluginEntry, PluginResolver};
use cordis_loader::outcome::{EntryOutcome, LoaderFailure};
use cordis_loader::plan::{InjectEntry, IsolateEntry, PlanError, RealmPolicy};
use cordis_loader::resolver::{JsonPrepareError, PluginRequest, ResolverFailure, ResolverFailureKind, prepare_plugin_json, prepare_service_json};
use std::convert::Infallible;
use std::future::{Future, ready};

fn root<T>() {}
fn module<T>() {}

struct R;
impl PluginResolver for R {
    type Error = std::io::Error;
    fn resolve(&self, _: PluginRequest<'_>) -> Result<Option<cordis_core::PreparedPlugin>, Self::Error> { Ok(None) }
}

struct P;
impl cordis_core::Plugin for P {
    type Config = ();
    type Input = ();
    type PrepareError = Infallible;
    type ApplyError = Infallible;
    fn prepare(&self, _: ()) -> Result<(), Infallible> { Ok(()) }
    fn apply(&self, _: cordis_core::Context, _: &()) -> impl Future<Output = Result<(), Infallible>> + Send { ready(Ok(())) }
}
struct S;
impl cordis_core::Service for S { const NAME: &'static str = "s"; }
impl cordis_core::ConfigurableService for S {
    type Config = ();
    type Layer = ();
    type Resolved = ();
    type PrepareError = Infallible;
    type ComposeError = Infallible;
    fn prepare_config(_: ()) -> Result<(), Infallible> { Ok(()) }
    fn compose_config<'a>(_: Option<&'a ()>, _: impl IntoIterator<Item=&'a ()>, _: Option<&'a ()>) -> Result<(), Infallible> { Ok(()) }
}

fn main() {
    root::<EntryId>(); root::<PluginEntry>(); root::<EntryGroup>(); root::<LoadPlanBuilder>();
    root::<LoadPlan>(); root::<LoadOutcome>();
    fn assert_resolver<T: PluginResolver>() {}
    assert_resolver::<R>();
    module::<InjectEntry>(); module::<IsolateEntry>(); module::<RealmPolicy>(); module::<PlanError>();
    module::<EntryOutcome>(); module::<LoaderFailure>();
    module::<PluginRequest<'static>>(); module::<ResolverFailure>(); module::<ResolverFailureKind>();
    let _ = prepare_plugin_json::<P>;
    let _ = prepare_service_json::<S>;
    let _: Option<JsonPrepareError<std::io::Error>> = None;
}
