use cordis_loader::{
    outcome::LoaderFailure,
    plan::PlanError,
    resolver::{JsonPrepareError, ResolverFailure, ResolverFailureKind},
};
fn plan(e: PlanError) { match e { PlanError::ForeignParent { .. } | PlanError::MissingResolveIdentity { .. } | PlanError::DuplicateInjectService { .. } | PlanError::DuplicateIsolateService { .. } => {}, _ => {} } }
fn loader(e: LoaderFailure) { match e { LoaderFailure::UnresolvedKey { .. } | LoaderFailure::Resolver(_) | LoaderFailure::Spawn(_) => {}, _ => {} } }
fn resolver(e: &ResolverFailure) { let _: ResolverFailureKind = e.kind(); let _: &str = e.diagnostic(); }
fn direct_typed<E: std::error::Error>(_: Option<JsonPrepareError<E>>) {}
fn main() { let _ = (plan, loader, resolver, direct_typed::<std::io::Error>); }
