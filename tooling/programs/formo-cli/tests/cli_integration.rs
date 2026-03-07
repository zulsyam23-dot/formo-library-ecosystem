use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), stamp));
        fs::create_dir_all(&root).expect("should create temp workspace");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("should create parent directory");
    }
    fs::write(path, content).expect("should write file");
}

fn formo_bin() -> PathBuf {
    static BIN_PATH: OnceLock<PathBuf> = OnceLock::new();
    BIN_PATH
        .get_or_init(|| {
            if let Ok(path) = std::env::var("CARGO_BIN_EXE_formo") {
                return PathBuf::from(path);
            }

            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_root = manifest_dir
                .parent()
                .and_then(|p| p.parent())
                .expect("workspace root should be resolvable");

            let status = Command::new("cargo")
                .current_dir(workspace_root)
                .args(["build", "-p", "formo-cli", "--bin", "formo"])
                .status()
                .expect("should run cargo build for formo binary");
            assert!(status.success(), "cargo build for formo binary should pass");

            let mut candidate = workspace_root.join("target").join("debug");
            if cfg!(windows) {
                candidate.push("formo.exe");
            } else {
                candidate.push("formo");
            }
            candidate
        })
        .clone()
}

fn run_formo(workspace: &Path, args: &[&str]) -> Output {
    Command::new(formo_bin())
        .current_dir(workspace)
        .args(args)
        .output()
        .expect("should run formo command")
}

fn assert_no_rust_panic(output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "unexpected rust panic in stderr: {stderr}"
    );
    assert!(
        !stderr.contains("panicked at"),
        "unexpected panic marker in stderr: {stderr}"
    );
}

fn create_multifile_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "views/header.fm" as Header;
import "styles/base.fs" as Base;

component App() {
  <Page>
    <Header title="Halo"/>
    <Text value="isi" style=BodyText/>
  </Page>
}
"#,
    );

    write_file(
        root,
        "views/header.fm",
        r#"component Header(title: string) {
  <Text value=title/>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r#"style BodyText {
  color: #112233;
}
"#,
    );
}

fn create_cycle_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "a.fm" as A;

component App() {
  <Page/>
}
"#,
    );

    write_file(
        root,
        "a.fm",
        r#"import "b.fm" as B;

component AComp() {
  <Page/>
}
"#,
    );

    write_file(
        root,
        "b.fm",
        r#"import "a.fm" as A;

component BComp() {
  <Page/>
}
"#,
    );
}

fn create_unknown_style_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="Halo" style=MissingStyle/>
  </Page>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r#"style BodyText {
  color: #111111;
}
"#,
    );
}

fn create_invalid_style_module_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="Halo"/>
  </Page>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r#"style BodyText {
  color #111111;
}
"#,
    );
}

fn create_unused_token_style_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="Halo"/>
  </Page>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r##"token {
  color.brand = #0A84FF;
}

style BodyText {
  color: #111111;
}
"##,
    );
}

fn create_recursive_component_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"component App() {
  <A/>
}

component A() {
  <B/>
}

component B() {
  <A/>
}
"#,
    );
}

fn create_parser_recovery_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"@@@
component Broken() {
  <Page>
    text bebas
  </Page>
}

!!!
component App() {
  <Page/>
}
"#,
    );
}

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

#[test]
fn build_web_multifile_project_generates_files() {
    let workspace = TempWorkspace::new("formo_cli_build_web");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build", "--target", "web", "--input", "main.fm", "--out", "dist",
        ])
        .output()
        .expect("should run formo build");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace.path().join("dist/index.html").exists(),
        "expected dist/index.html to exist"
    );
    assert!(
        workspace.path().join("dist/app.js").exists(),
        "expected dist/app.js to exist"
    );
    assert!(
        workspace.path().join("dist/app.css").exists(),
        "expected dist/app.css to exist"
    );
}

#[test]
fn build_desktop_project_generates_bundle_and_ir() {
    let workspace = TempWorkspace::new("formo_cli_build_desktop");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build",
            "--target",
            "desktop",
            "--input",
            "main.fm",
            "--out",
            "dist-desktop",
        ])
        .output()
        .expect("should run formo build desktop");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        workspace.path().join("dist-desktop/index.html").exists(),
        "expected dist-desktop/index.html to exist"
    );
    assert!(
        workspace.path().join("dist-desktop/app.js").exists(),
        "expected dist-desktop/app.js to exist"
    );
    assert!(
        workspace.path().join("dist-desktop/app.css").exists(),
        "expected dist-desktop/app.css to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/desktop-bridge.js")
            .exists(),
        "expected dist-desktop/desktop-bridge.js to exist"
    );
    assert!(
        workspace.path().join("dist-desktop/app.ir.json").exists(),
        "expected dist-desktop/app.ir.json to exist"
    );
}

#[test]
fn build_web_prod_generates_minified_assets() {
    let workspace = TempWorkspace::new("formo_cli_build_web_prod");
    create_multifile_sample(workspace.path());

    let dev_output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-dev",
        ])
        .output()
        .expect("should run formo build dev");
    assert!(
        dev_output.status.success(),
        "expected dev success, stderr={}",
        String::from_utf8_lossy(&dev_output.stderr)
    );

    let prod_output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-prod",
            "--prod",
        ])
        .output()
        .expect("should run formo build prod");
    assert!(
        prod_output.status.success(),
        "expected prod success, stderr={}",
        String::from_utf8_lossy(&prod_output.stderr)
    );

    let dev_js = fs::read_to_string(workspace.path().join("dist-dev/app.js"))
        .expect("dev app.js should exist");
    let prod_js = fs::read_to_string(workspace.path().join("dist-prod/app.js"))
        .expect("prod app.js should exist");
    assert!(
        prod_js.len() < dev_js.len(),
        "expected prod app.js smaller (dev={}, prod={})",
        dev_js.len(),
        prod_js.len()
    );

    let dev_css = fs::read_to_string(workspace.path().join("dist-dev/app.css"))
        .expect("dev app.css should exist");
    let prod_css = fs::read_to_string(workspace.path().join("dist-prod/app.css"))
        .expect("prod app.css should exist");
    assert!(
        prod_css.len() < dev_css.len(),
        "expected prod app.css smaller (dev={}, prod={})",
        dev_css.len(),
        prod_css.len()
    );
}

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
