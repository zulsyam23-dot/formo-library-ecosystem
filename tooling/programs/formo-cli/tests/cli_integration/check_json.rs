use super::support::*;

#[test]
fn check_json_success_for_multifile_project() {
    let workspace = TempWorkspace::new("formo_cli_json_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert_eq!(payload["input"], Value::String("main.fm".to_string()));
    assert_eq!(payload["entry"], Value::String("App".to_string()));
}

#[test]
fn check_json_schema_includes_schema_metadata() {
    let workspace = TempWorkspace::new("formo_cli_json_schema");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json-schema"])
        .output()
        .expect("should run formo check");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert_eq!(
        payload["schema"]["id"],
        Value::String("https://formo.dev/schema/check-result/1".to_string())
    );
}

#[test]
fn check_json_pretty_success_is_multiline_json() {
    let workspace = TempWorkspace::new("formo_cli_json_pretty");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json-pretty"])
        .output()
        .expect("should run formo check");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('\n'),
        "expected pretty json output to contain newline"
    );

    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
}

#[test]
fn check_json_failure_contains_error_payload() {
    let workspace = TempWorkspace::new("formo_cli_json_err");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component App() {
  <Text value="ok" nope="bad"/>
}
"#,
    );

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for invalid source"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("typer".to_string()));

    let error_text = payload["error"]
        .as_str()
        .expect("error payload should be string");
    assert!(
        error_text.contains("E2250"),
        "expected builtin unknown prop code, got: {error_text}"
    );
    assert_eq!(
        payload["errorMeta"]["code"],
        Value::String("E2250".to_string())
    );
    assert_eq!(payload["errorMeta"]["line"], Value::Number(2.into()));
    assert_eq!(payload["errorMeta"]["col"], Value::Number(20.into()));
}

#[test]
fn check_json_failure_contains_import_cycle_error() {
    let workspace = TempWorkspace::new("formo_cli_json_cycle");
    create_cycle_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for cyclic import"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("resolver".to_string()));

    let error_text = payload["error"]
        .as_str()
        .expect("error payload should be string");
    assert!(
        error_text.contains("cyclic import detected"),
        "expected import cycle message, got: {error_text}"
    );
    assert!(payload["errorMeta"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("cyclic import detected"));
}

#[test]
fn check_json_failure_contains_unknown_style_error() {
    let workspace = TempWorkspace::new("formo_cli_json_style");
    create_unknown_style_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for unknown style reference"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));

    let error_text = payload["error"]
        .as_str()
        .expect("error payload should be string");
    assert!(
        error_text.contains("unknown style"),
        "expected unknown style message, got: {error_text}"
    );
}

#[test]
fn check_json_failure_contains_style_error_code_and_stage() {
    let workspace = TempWorkspace::new("formo_cli_json_style_code");
    create_invalid_style_module_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for invalid style module"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("style".to_string()));
    assert_eq!(
        payload["errorMeta"]["code"],
        Value::String("E1301".to_string())
    );
}

#[test]
fn check_json_failure_contains_unused_token_error_code_and_stage() {
    let workspace = TempWorkspace::new("formo_cli_json_style_unused_token");
    create_unused_token_style_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for unused style token"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("style".to_string()));
    assert_eq!(
        payload["errorMeta"]["code"],
        Value::String("E1304".to_string())
    );
}

#[test]
fn check_json_failure_contains_lowering_error_code_and_stage() {
    let workspace = TempWorkspace::new("formo_cli_json_lowering_code");
    create_recursive_component_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["check", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo check");

    assert!(
        !output.status.success(),
        "expected failure for recursive component expansion"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("lowering".to_string()));
    assert_eq!(
        payload["errorMeta"]["code"],
        Value::String("E1403".to_string())
    );
    assert!(
        payload["errorMeta"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("recursive component expansion detected"),
        "expected recursive expansion message in errorMeta"
    );
}
