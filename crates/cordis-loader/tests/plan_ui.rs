//! Compile-time evidence for Issue 49's immutable Loader plan surface.

#[test]
fn loader_plan_public_surface() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-plan/pass/*.rs");
    t.compile_fail("tests/ui-plan/fail/*.rs");
}
