//! Compile-time evidence for the Issue 24/25 Service publication and exact-control surface.

use trybuild::TestCases;

#[test]
fn service_interface() {
    let tests = TestCases::new();
    tests.compile_fail("tests/ui-service/fail/*.rs");
}
