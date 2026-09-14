use cordis_loader::resolver::ResolverFailure;
fn inspect(failure: &ResolverFailure) { let _ = failure.downcast_ref::<std::io::Error>(); }
fn main() {}
