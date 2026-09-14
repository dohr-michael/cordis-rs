use cordis_loader::resolver::{ResolverFailure, ResolverFailureKind};
fn main() {
    let _ = ResolverFailure { kind: ResolverFailureKind::Panic, diagnostic: String::new() };
}
