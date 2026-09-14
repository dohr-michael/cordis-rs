use cordis_core::InjectSpec;

fn main() {
    let spec = InjectSpec::none().require("svc");
    let _ = spec.entries();
    let _ = std::mem::size_of::<cordis_core::deps::DependencyIndex>();
}
