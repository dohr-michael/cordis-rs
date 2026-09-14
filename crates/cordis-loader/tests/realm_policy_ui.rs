//! Compile-time evidence for Issue 52 realm-policy ownership.

#[test]
fn loader_realm_policy_does_not_cross_the_core_seam() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui-realm-policy/fail/*.rs");
}
