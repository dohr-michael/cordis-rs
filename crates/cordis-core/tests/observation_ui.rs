//! Compile-time evidence for Issue 41's read-only observation facade.
use trybuild::TestCases;

#[test]
fn runtime_snapshot_interface() {
    let tests = TestCases::new();
    tests.pass("tests/ui-observation/pass/*.rs");
    tests.compile_fail("tests/ui-observation/fail/*.rs");
}
