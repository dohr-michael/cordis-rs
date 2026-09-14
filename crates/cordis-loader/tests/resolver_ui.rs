//! Compile-time evidence for Issue 50's semantic resolver surface.

#[test]
fn loader_resolver_public_surface() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-resolver/pass/*.rs");
    t.compile_fail("tests/ui-resolver/fail/*.rs");
}
