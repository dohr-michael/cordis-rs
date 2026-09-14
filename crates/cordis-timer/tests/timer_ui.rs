//! Compile-time evidence for the Timer public surface.

#[test]
fn timer_public_surface() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui-timer/fail/*.rs");
    t.pass("tests/ui-timer/pass/*.rs");
}
