//! Aggregate compile-time evidence for Issue 59 Timer facade closure.

#[test]
fn timer_flat_facade_is_closed() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui-facade59/pass/*.rs");
    t.compile_fail("tests/ui-facade59/fail/*.rs");
}
