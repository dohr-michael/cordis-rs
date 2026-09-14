//! Aggregate compile-time evidence for Issue 59 Loader facade closure.

#[test]
fn loader_semantic_facade_is_closed() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-facade59/pass/*.rs");
    t.compile_fail("tests/ui-facade59/fail/*.rs");
}
