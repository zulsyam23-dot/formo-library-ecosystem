use super::support::*;

#[test]
fn diagnose_json_success_contains_stats() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert!(
        payload["stats"]["nodes"].as_u64().unwrap_or(0) > 0,
        "expected nodes > 0 in stats"
    );
}

#[test]
fn diagnose_json_failure_contains_stage() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_fail");
    create_cycle_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--json-schema"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        !output.status.success(),
        "expected failure for cyclic import"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("resolver".to_string()));
    assert_eq!(
        payload["schema"]["id"],
        Value::String("https://formo.dev/schema/diagnose-result/1".to_string())
    );
}

#[test]
fn diagnose_json_parser_recovery_reports_multiple_diagnostics() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_recovery_json");
    create_parser_recovery_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--json"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        !output.status.success(),
        "expected failure for parser recovery sample"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("parser".to_string()));
    assert!(
        payload["diagnostics"].is_array(),
        "diagnostics should be array on parser recovery failure"
    );
    assert!(
        payload["diagnostics"].as_array().map_or(0, Vec::len) >= 2,
        "expected multiple parser diagnostics"
    );
    assert_eq!(
        payload["diagnostics"][0]["code"],
        Value::String("E1100".to_string())
    );
}

#[test]
fn diagnose_lsp_parser_recovery_emits_multiple_diagnostics() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_recovery_lsp");
    create_parser_recovery_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--lsp"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        !output.status.success(),
        "expected failure for parser recovery sample"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["stage"], Value::String("parser".to_string()));
    assert!(payload["documents"].is_array(), "documents should be array");
    let diagnostics = payload["documents"][0]["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        diagnostics.len() >= 2,
        "expected multiple lsp diagnostics for parser recovery"
    );
}

#[test]
fn diagnose_lsp_success_contains_documents_payload() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_lsp_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--lsp"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert!(payload["documents"].is_array(), "documents should be array");
    assert!(
        payload["documents"][0]["uri"]
            .as_str()
            .unwrap_or_default()
            .starts_with("file://"),
        "uri should be file URI"
    );
}

#[test]
fn diagnose_lsp_failure_contains_diagnostics_range() {
    let workspace = TempWorkspace::new("formo_cli_diagnose_lsp_fail");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component App() {
  <Page>
}
"#,
    );

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["diagnose", "--input", "main.fm", "--lsp"])
        .output()
        .expect("should run formo diagnose");

    assert!(
        !output.status.success(),
        "expected failure for invalid source"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert!(payload["documents"].is_array(), "documents should be array");

    let diag = &payload["documents"][0]["diagnostics"][0];
    assert_eq!(diag["severity"], Value::Number(1.into()));
    assert!(
        diag["range"]["start"]["line"].as_u64().is_some(),
        "line should exist in range.start"
    );
    assert!(
        diag["message"]
            .as_str()
            .unwrap_or_default()
            .contains("expected `<` before EOF"),
        "message should contain parser error"
    );
}

#[test]
fn lsp_command_success_emits_publish_diagnostics_notification() {
    let workspace = TempWorkspace::new("formo_cli_lsp_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["lsp", "--input", "main.fm"])
        .output()
        .expect("should run formo lsp");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|text| !text.trim().is_empty())
        .expect("stdout should contain at least one json line");
    let payload: Value = serde_json::from_str(line).expect("line should be json");
    assert_eq!(
        payload["method"],
        Value::String("textDocument/publishDiagnostics".to_string())
    );
    assert!(
        payload["params"]["uri"]
            .as_str()
            .unwrap_or_default()
            .starts_with("file://"),
        "uri should be file URI"
    );
}

#[test]
fn lsp_command_failure_emits_error_diagnostic_notification() {
    let workspace = TempWorkspace::new("formo_cli_lsp_fail");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component App() {
  <Page>
}
"#,
    );

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["lsp", "--input", "main.fm"])
        .output()
        .expect("should run formo lsp");

    assert!(
        output.status.success(),
        "expected success (adapter mode), stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|text| !text.trim().is_empty())
        .expect("stdout should contain at least one json line");
    let payload: Value = serde_json::from_str(line).expect("line should be json");
    let diag = &payload["params"]["diagnostics"][0];
    assert_eq!(diag["severity"], Value::Number(1.into()));
    assert!(
        diag["message"]
            .as_str()
            .unwrap_or_default()
            .contains("expected `<` before EOF"),
        "parser error message should be present"
    );
}
