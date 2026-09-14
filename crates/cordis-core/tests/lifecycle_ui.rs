//! Compile-time evidence for migrated v3 lifecycle surfaces (Issues 28–29).

use trybuild::TestCases;

#[test]
fn lifecycle_interface() {
    let tests = TestCases::new();
    tests.compile_fail("tests/ui-lifecycle/fail/*.rs");
}
