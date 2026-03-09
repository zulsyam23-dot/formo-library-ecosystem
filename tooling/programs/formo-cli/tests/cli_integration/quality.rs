use super::support::*;

#[test]
fn doctor_json_success_contains_checks_and_pipeline() {
    let workspace = TempWorkspace::new("formo_cli_doctor_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["doctor", "--input", "main.fm", "--json-schema"])
        .output()
        .expect("should run formo doctor");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert_eq!(payload["checks"]["inputFile"], Value::Bool(true));
    assert_eq!(payload["pipeline"]["ok"], Value::Bool(true));
    assert_eq!(
        payload["schema"]["id"],
        Value::String("https://formo.dev/schema/doctor-result/1".to_string())
    );
}

#[test]
fn doctor_json_failure_for_missing_input_file() {
    let workspace = TempWorkspace::new("formo_cli_doctor_missing_input");

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["doctor", "--input", "missing.fm", "--json"])
        .output()
        .expect("should run formo doctor");

    assert!(
        !output.status.success(),
        "expected failure for missing input file"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["checks"]["inputFile"], Value::Bool(false));
    assert_eq!(
        payload["pipeline"]["stage"],
        Value::String("preflight".to_string())
    );
    assert!(payload["pipeline"]["errorMeta"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("input file not found"));
}

#[test]
fn fmt_rewrites_file_to_canonical_layout() {
    let workspace = TempWorkspace::new("formo_cli_fmt_write");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component   App( title:string ){
<Page><Text value = "Halo"/></Page>
}
"#,
    );

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["fmt", "--input", "main.fm"])
        .output()
        .expect("should run formo fmt");

    assert!(
        output.status.success(),
        "expected fmt success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let formatted =
        fs::read_to_string(workspace.path().join("main.fm")).expect("formatted file should exist");
    assert_eq!(
        formatted,
        "component App(title: string) {\n  <Page>\n    <Text value=\"Halo\"/>\n  </Page>\n}\n"
    );
}

#[test]
fn fmt_check_fails_when_file_is_not_canonical() {
    let workspace = TempWorkspace::new("formo_cli_fmt_check_fail");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component App(){<Page><Text value="A"/></Page>}"#,
    );

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args(["fmt", "--input", "main.fm", "--check"])
        .output()
        .expect("should run formo fmt --check");

    assert!(
        !output.status.success(),
        "expected fmt --check to fail for unformatted file"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fmt check failed: main.fm"),
        "expected failure message in stdout, got: {stdout}"
    );
}

#[test]
fn bench_generates_json_report_with_compile_and_render_metrics() {
    let workspace = TempWorkspace::new("formo_cli_bench_ok");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "bench",
            "--input",
            "main.fm",
            "--iterations",
            "3",
            "--warmup",
            "1",
            "--nodes",
            "128",
            "--out",
            "dist/bench.json",
            "--json-pretty",
        ])
        .output()
        .expect("should run formo bench");

    assert!(
        output.status.success(),
        "expected bench success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = workspace.path().join("dist/bench.json");
    assert!(
        report_path.exists(),
        "expected benchmark report to exist at dist/bench.json"
    );

    let report_text = fs::read_to_string(report_path).expect("benchmark report should be readable");
    let payload: Value =
        serde_json::from_str(&report_text).expect("benchmark report should be json");
    assert_eq!(payload["ok"], Value::Bool(true));
    assert_eq!(payload["budget"]["enabled"], Value::Bool(false));
    assert!(
        payload["benchmark"]["compileMs"]["count"]
            .as_u64()
            .unwrap_or_default()
            >= 1,
        "expected compile sample count >= 1"
    );
    assert!(
        payload["benchmark"]["firstRenderMs"]["count"]
            .as_u64()
            .unwrap_or_default()
            >= 1,
        "expected first-render sample count >= 1"
    );
}

#[test]
fn bench_fails_when_budget_threshold_is_exceeded() {
    let workspace = TempWorkspace::new("formo_cli_bench_budget_fail");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "bench",
            "--input",
            "main.fm",
            "--iterations",
            "3",
            "--warmup",
            "1",
            "--nodes",
            "128",
            "--out",
            "dist/bench-budget.json",
            "--max-compile-p95-ms",
            "0.0001",
            "--max-first-render-p95-ms",
            "0.0001",
        ])
        .output()
        .expect("should run formo bench");

    assert!(
        !output.status.success(),
        "expected bench to fail when budget threshold is too strict"
    );
    assert_no_rust_panic(&output);

    let report_path = workspace.path().join("dist/bench-budget.json");
    assert!(
        report_path.exists(),
        "expected benchmark report to still be generated when budget fails"
    );

    let report_text = fs::read_to_string(report_path).expect("benchmark report should be readable");
    let payload: Value =
        serde_json::from_str(&report_text).expect("benchmark report should be json");
    assert_eq!(payload["ok"], Value::Bool(false));
    assert_eq!(payload["budget"]["enabled"], Value::Bool(true));
    assert_eq!(payload["budget"]["ok"], Value::Bool(false));
    assert_eq!(payload["budget"]["compileP95"]["pass"], Value::Bool(false));
}

#[test]
fn invalid_inputs_fail_as_diagnostics_not_rust_panic() {
    let workspace = TempWorkspace::new("formo_cli_no_panic_invalid");
    write_file(
        workspace.path(),
        "main.fm",
        r#"component App() {
  <Page>
}
"#,
    );
    fs::write(workspace.path().join("bad_utf8.fm"), [0x66u8, 0x6f, 0x80])
        .expect("should write bad utf8 file");

    let check_json = run_formo(workspace.path(), &["check", "--input", "main.fm", "--json"]);
    assert!(!check_json.status.success(), "check must fail");
    assert_no_rust_panic(&check_json);
    let check_payload: Value =
        serde_json::from_slice(&check_json.stdout).expect("check --json should return JSON");
    assert_eq!(check_payload["ok"], Value::Bool(false));

    let diagnose_json = run_formo(
        workspace.path(),
        &["diagnose", "--input", "main.fm", "--json"],
    );
    assert!(!diagnose_json.status.success(), "diagnose must fail");
    assert_no_rust_panic(&diagnose_json);
    let diagnose_payload: Value =
        serde_json::from_slice(&diagnose_json.stdout).expect("diagnose --json should return JSON");
    assert_eq!(diagnose_payload["ok"], Value::Bool(false));

    let doctor_json = run_formo(
        workspace.path(),
        &["doctor", "--input", "main.fm", "--json"],
    );
    assert!(
        !doctor_json.status.success(),
        "doctor must fail on invalid pipeline"
    );
    assert_no_rust_panic(&doctor_json);
    let doctor_payload: Value =
        serde_json::from_slice(&doctor_json.stdout).expect("doctor --json should return JSON");
    assert_eq!(doctor_payload["ok"], Value::Bool(false));

    let build = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-bad",
        ],
    );
    assert!(!build.status.success(), "build must fail");
    assert_no_rust_panic(&build);

    let bench = run_formo(
        workspace.path(),
        &[
            "bench",
            "--input",
            "main.fm",
            "--iterations",
            "2",
            "--warmup",
            "0",
            "--nodes",
            "16",
            "--out",
            "dist/bench.json",
        ],
    );
    assert!(!bench.status.success(), "bench must fail");
    assert_no_rust_panic(&bench);

    let check_bad_utf8 = run_formo(
        workspace.path(),
        &["check", "--input", "bad_utf8.fm", "--json"],
    );
    assert!(!check_bad_utf8.status.success(), "bad utf8 check must fail");
    assert_no_rust_panic(&check_bad_utf8);
    let bad_utf8_payload: Value = serde_json::from_slice(&check_bad_utf8.stdout)
        .expect("check --json bad utf8 should return JSON");
    assert_eq!(bad_utf8_payload["ok"], Value::Bool(false));
}

#[test]
fn commands_fail_fast_for_invalid_input_extensions() {
    let workspace = TempWorkspace::new("formo_cli_invalid_input_extension");
    write_file(
        workspace.path(),
        "main.txt",
        r#"component App() {
  <Page/>
}
"#,
    );
    write_file(
        workspace.path(),
        "logic/main.txt",
        r#"module AppController;

logic AppController {
  event startApp {
    action emit "READY";
  }
}
"#,
    );

    let check = run_formo(workspace.path(), &["check", "--input", "main.txt"]);
    assert!(!check.status.success(), "check must fail for .txt input");
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        check_stderr.contains("`check` expects input file with `.fm` extension"),
        "expected extension guard in stderr, got: {check_stderr}"
    );
    assert_no_rust_panic(&check);

    let logic = run_formo(workspace.path(), &["logic", "--input", "logic/main.txt"]);
    assert!(!logic.status.success(), "logic must fail for .txt input");
    let logic_stderr = String::from_utf8_lossy(&logic.stderr);
    assert!(
        logic_stderr.contains("`logic` expects input file with `.fl` extension"),
        "expected extension guard in stderr, got: {logic_stderr}"
    );
    assert_no_rust_panic(&logic);
}
