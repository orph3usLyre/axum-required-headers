//! Compile-time error tests using trybuild.
//!
//! These tests verify that the macros produce helpful compile errors
//! for invalid usage patterns.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
