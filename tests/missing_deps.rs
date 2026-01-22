//! Compile-time error tests for missing dependencies.
//!
//! These tests verify that the macros produce helpful compile errors
//! when required dependencies (axum, http) are not present in the user's Cargo.toml.
//!
//! Unlike the regular compile_fail tests, these tests need to be run in a separate
//! context without the axum/http dependencies, so they use separate test crates.

use std::process::Command;

/// Test that the Headers macro produces a helpful error when axum is missing
#[test]
fn missing_axum_dependency() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.join("tests/ui-deps/missing_axum");

    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run cargo build");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "Expected compilation to fail when axum is missing"
    );
    assert!(
        stderr.contains("Expected to find the 'axum' dependency but none was found"),
        "Expected error message about missing axum dependency, got:\n{}",
        stderr
    );
}

/// Test that the Headers macro produces a helpful error when http is missing
#[test]
fn missing_http_dependency() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.join("tests/ui-deps/missing_http");

    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run cargo build");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "Expected compilation to fail when http is missing"
    );
    assert!(
        stderr.contains("Expected to find the 'http' dependency but none was found"),
        "Expected error message about missing http dependency, got:\n{}",
        stderr
    );
}
