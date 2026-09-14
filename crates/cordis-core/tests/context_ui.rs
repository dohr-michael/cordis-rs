//! Compile-time evidence for the opaque v3 Context/ServiceRealm surface.

use trybuild::TestCases;

#[test]
fn context_and_realm_interface() {
    let tests = TestCases::new();
    tests.compile_fail("tests/ui-context/fail/*.rs");
}
