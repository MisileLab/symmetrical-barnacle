use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn compile_flux_file(file_path: &str) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "fluxc", "--", "check", file_path])
        .output()
        .map_err(|e| format!("Failed to run fluxc: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_basic_example() {
    let result = compile_flux_file("examples/basic.flux");
    assert!(result.is_ok(), "basic.flux should type check successfully");
}

#[test]
fn test_effect_violation_example() {
    let result = compile_flux_file("examples/effect_violation.flux");
    assert!(result.is_err(), "effect_violation.flux should fail type checking");
}

#[test]
fn test_arithmetic_example() {
    let result = compile_flux_file("examples/arithmetic.flux");
    assert!(result.is_ok(), "arithmetic.flux should type check successfully");
}

#[test]
fn test_conditional_example() {
    let result = compile_flux_file("examples/conditional.flux");
    assert!(result.is_ok(), "conditional.flux should type check successfully");
}

#[test]
fn test_zero_alloc_example() {
    let result = compile_flux_file("examples/zero_alloc.flux");
    assert!(result.is_ok(), "zero_alloc.flux should type check successfully");
}
