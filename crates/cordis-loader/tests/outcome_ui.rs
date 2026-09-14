//! Compile-time evidence for Issue 51 outcome surface.

#[test]
fn loader_outcome_public_surface() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-outcome/pass/*.rs");
    t.compile_fail("tests/ui-outcome/fail/*.rs");
}
