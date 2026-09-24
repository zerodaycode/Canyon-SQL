use std::path::PathBuf;
use std::process::{Command, Output};

fn check_fixture(binary: &str) -> Output {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap();

    Command::new(env!("CARGO"))
        .args([
            "check",
            "--quiet",
            "--manifest-path",
            "ui/Cargo.toml",
            "--bin",
            binary,
        ])
        .current_dir(&manifest_dir)
        .env(
            "CARGO_TARGET_DIR",
            workspace_dir.join("target/compile-tests"),
        )
        .output()
        .expect("failed to invoke Cargo for a compile-test fixture")
}

fn assert_compiles(binary: &str) {
    let output = check_fixture(binary);
    assert!(
        output.status.success(),
        "fixture `{binary}` failed to compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_compile_error(binary: &str, expected_diagnostics: &[&str]) {
    let output = check_fixture(binary);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "fixture `{binary}` unexpectedly compiled"
    );
    for diagnostic in expected_diagnostics {
        assert!(
            stderr.contains(diagnostic),
            "fixture `{binary}` did not emit `{diagnostic}`:\n{stderr}"
        );
    }
}

#[test]
fn generated_relationship_methods_are_inherent() {
    assert_compiles("inherent_relationship_methods");
}

#[test]
fn runtime_entity_operations_can_be_derived_selectively() {
    assert_compiles("selective_entity_operations");
}

#[test]
fn runtime_entity_operations_reject_the_wrong_entity_type() {
    assert_compile_error(
        "runtime_entity_type_mismatch",
        &[
            "mismatched types",
            "expected `&mut User`",
            "found `&mut Team`",
        ],
    );
}

#[test]
fn derives_reject_unnamed_entities_with_a_clear_diagnostic() {
    assert_compile_error(
        "unnamed_entity",
        &["Canyon derives require a struct with named fields"],
    );
}

#[test]
fn relationships_reject_missing_referenced_fields() {
    assert_compile_error(
        "invalid_foreign_field",
        &["no field `missing` on type `&Team`"],
    );
}

#[test]
fn relationships_do_not_ignore_malformed_annotations() {
    assert_compile_error(
        "malformed_foreign_key",
        &["Expected a typed foreign-key reference such as `Team::id`"],
    );
}
