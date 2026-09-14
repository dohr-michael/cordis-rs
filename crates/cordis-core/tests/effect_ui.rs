//! Compile evidence for the exact effect-cleanup surface: the frozen
//! FnOnce forms and move-only registration compile, while labels,
//! introspection, reusable guards, and clone/Debug contracts do not.

use trybuild::TestCases;

#[test]
fn effect_interface() {
    let tests = TestCases::new();
    tests.pass("tests/ui-effect/pass/*.rs");
    tests.compile_fail("tests/ui-effect/fail/*.rs");
}
