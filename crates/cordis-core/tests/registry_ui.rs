//! Compile-time evidence for Issue 30's semantic Registry surface.

use trybuild::TestCases;

#[test]
fn registry_interface() {
    let tests = TestCases::new();
    tests.pass("tests/ui-registry/pass/*.rs");
    tests.compile_fail("tests/ui-registry/fail/*.rs");
}
