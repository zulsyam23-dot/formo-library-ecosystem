use super::support::*;

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
fn build_desktop_project_generates_native_bundle_and_ir() {
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
        workspace
            .path()
            .join("dist-desktop/app.native.json")
            .exists(),
        "expected dist-desktop/app.native.json to exist"
    );
    assert!(
        workspace.path().join("dist-desktop/app.native.rs").exists(),
        "expected dist-desktop/app.native.rs to exist"
    );
    assert!(
        !workspace.path().join("dist-desktop/index.html").exists(),
        "did not expect dist-desktop/index.html for native desktop target"
    );
    assert!(
        workspace.path().join("dist-desktop/app.ir.json").exists(),
        "expected dist-desktop/app.ir.json to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/Cargo.toml")
            .exists(),
        "expected dist-desktop/native-app/Cargo.toml to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/main.rs")
            .exists(),
        "expected dist-desktop/native-app/src/main.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/actions.rs")
            .exists(),
        "expected dist-desktop/native-app/src/actions.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/app.rs")
            .exists(),
        "expected dist-desktop/native-app/src/app.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/model.rs")
            .exists(),
        "expected dist-desktop/native-app/src/model.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/style.rs")
            .exists(),
        "expected dist-desktop/native-app/src/style.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/mod.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/mod.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/flow.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/flow.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/controls.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/controls.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/media.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/media.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/shared.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/shared.rs to exist"
    );
    assert!(
        workspace
            .path()
            .join("dist-desktop/native-app/src/render/state.rs")
            .exists(),
        "expected dist-desktop/native-app/src/render/state.rs to exist"
    );
}

#[test]
fn build_desktop_generates_actions_registry_from_ir_props() {
    let workspace = TempWorkspace::new("formo_cli_build_desktop_actions");
    create_desktop_actions_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "desktop",
            "--input",
            "main.fm",
            "--out",
            "dist-desktop",
        ],
    );
    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actions_rs = fs::read_to_string(
        workspace
            .path()
            .join("dist-desktop/native-app/src/actions.rs"),
    )
    .expect("expected desktop generated actions.rs");

    assert!(actions_rs.contains("\"run\" => handle_run"));
    assert!(actions_rs.contains("\"search\" => handle_search"));
    assert!(actions_rs.contains("fn handle_run("));
    assert!(actions_rs.contains("fn handle_search("));
}

#[test]
fn build_desktop_syncs_actions_registry_with_logic_events() {
    let workspace = TempWorkspace::new("formo_cli_build_desktop_actions_logic_sync");
    create_desktop_actions_sample_with_logic(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "desktop",
            "--input",
            "main.fm",
            "--out",
            "dist-desktop",
        ],
    );
    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actions_rs = fs::read_to_string(
        workspace
            .path()
            .join("dist-desktop/native-app/src/actions.rs"),
    )
    .expect("expected desktop generated actions.rs");

    assert!(actions_rs.contains("\"run\" => handle_run"));
    assert!(actions_rs.contains("\"search\" => handle_search"));
    assert!(actions_rs.contains("\"syncCache\" => handle_sync_cache"));
    assert!(actions_rs.contains("fn handle_sync_cache("));
    assert!(actions_rs.contains("FL contract:"));
    assert!(
        actions_rs.contains("if let Some(next_value) = state_store.read().get(\"query\").cloned()")
    );
    assert!(actions_rs.contains("set_state(state_store.clone(), \"query\", next_value);"));
}

#[test]
fn build_desktop_generates_set_expression_evaluator_for_logic_event() {
    let workspace = TempWorkspace::new("formo_cli_build_desktop_actions_expression");
    create_desktop_actions_expression_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "desktop",
            "--input",
            "main.fm",
            "--out",
            "dist-desktop",
        ],
    );
    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actions_rs = fs::read_to_string(
        workspace
            .path()
            .join("dist-desktop/native-app/src/actions.rs"),
    )
    .expect("expected desktop generated actions.rs");

    assert!(actions_rs.contains("\"increment\" => handle_increment"));
    assert!(actions_rs.contains("\"weighted\" => handle_weighted"));
    assert!(actions_rs.contains("fn handle_increment("));
    assert!(actions_rs.contains("fn handle_weighted("));
    assert!(actions_rs.contains("eval_set_expression_rpn(state_store.clone(),"));
    assert!(
        actions_rs.contains(
            "(\"stateRef\", \"count\"), (\"intLiteral\", \"1\"), (\"intLiteral\", \"2\"), (\"operator\", \"mul\"), (\"operator\", \"add\")"
        )
    );
    assert!(
        actions_rs.contains(
            "(\"stateRef\", \"count\"), (\"intLiteral\", \"1\"), (\"operator\", \"add\"), (\"intLiteral\", \"2\"), (\"operator\", \"mul\")"
        )
    );
}

#[test]
fn build_web_syncs_logic_actions_into_runtime_js() {
    let workspace = TempWorkspace::new("formo_cli_build_web_actions_logic_sync");
    create_desktop_actions_expression_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-web",
        ],
    );
    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actions_js = fs::read_to_string(
        workspace
            .path()
            .join("dist-web/runtime/app/50_actions_state.js"),
    )
    .expect("expected generated web runtime actions file");
    assert!(actions_js.contains("const formoGeneratedActions = {"));
    assert!(actions_js.contains("\"increment\": function(event) {"));
    assert!(actions_js.contains("\"weighted\": function(event) {"));
    assert!(actions_js.contains("const action = resolveActionHandler(actionName);"));
    assert!(actions_js.contains("function evalSetExpressionRpn(tokens) {"));
    assert!(
        actions_js.contains(
            "[\"stateRef\", \"count\"], [\"intLiteral\", \"1\"], [\"intLiteral\", \"2\"], [\"operator\", \"mul\"], [\"operator\", \"add\"]"
        )
    );
    assert!(
        actions_js.contains(
            "[\"stateRef\", \"count\"], [\"intLiteral\", \"1\"], [\"operator\", \"add\"], [\"intLiteral\", \"2\"], [\"operator\", \"mul\"]"
        )
    );

    let bundle_js =
        fs::read_to_string(workspace.path().join("dist-web/app.js")).expect("expected app.js");
    assert!(bundle_js.contains("const formoGeneratedActions = {"));
    assert!(bundle_js.contains("const action = resolveActionHandler(actionName);"));
}

#[test]
fn build_web_and_desktop_share_canonical_style_semantics() {
    let workspace = TempWorkspace::new("formo_cli_build_canonical_style_parity");
    create_canonical_style_parity_sample(workspace.path());

    let web_output = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-web",
        ],
    );
    assert!(
        web_output.status.success(),
        "expected web build success, stderr={}",
        String::from_utf8_lossy(&web_output.stderr)
    );

    let desktop_output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "desktop",
            "--input",
            "main.fm",
            "--out",
            "dist-desktop",
        ],
    );
    assert!(
        desktop_output.status.success(),
        "expected desktop build success, stderr={}",
        String::from_utf8_lossy(&desktop_output.stderr)
    );

    let css = fs::read_to_string(workspace.path().join("dist-web/app.css"))
        .expect("expected web css output");
    assert!(
        css.contains("align-items: start;"),
        "expected canonical align-items value in web css, got: {css}"
    );
    assert!(
        css.contains("justify-content: space-between;"),
        "expected canonical justify-content value in web css, got: {css}"
    );

    let native_raw = fs::read_to_string(workspace.path().join("dist-desktop/app.native.json"))
        .expect("expected desktop native json output");
    let native: Value =
        serde_json::from_str(&native_raw).expect("desktop native json should parse");
    let root = &native["components"][0]["rootNode"];
    let row = find_widget_node(root, "Row").expect("expected Row node in desktop native json");
    assert_eq!(
        row["resolvedStyle"]["align-items"]["v"],
        Value::String("start".to_string()),
        "desktop resolved style should use canonical align-items value"
    );
    assert_eq!(
        row["resolvedStyle"]["justify-content"]["v"],
        Value::String("space-between".to_string()),
        "desktop resolved style should use canonical justify-content value"
    );
}

#[test]
fn build_web_writes_engine_bridge_manifest() {
    let workspace = TempWorkspace::new("formo_cli_build_engine_manifest");
    create_multifile_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-web",
        ],
    );
    assert!(
        output.status.success(),
        "expected web build success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest_path = workspace.path().join("dist-web/engine.bridge.json");
    assert!(
        manifest_path.exists(),
        "expected engine bridge manifest to exist"
    );
    let raw = fs::read_to_string(manifest_path).expect("engine bridge manifest should be readable");
    let manifest: Value = serde_json::from_str(&raw).expect("engine bridge manifest should parse");
    assert_eq!(
        manifest["engineProfile"]["id"],
        Value::String("fm-fs-fl-bridge".to_string())
    );
    assert_eq!(
        manifest["standard"]["fs"]["canonicalCoveragePct"],
        Value::from(100.0)
    );
}

fn find_widget_node<'a>(node: &'a Value, widget: &str) -> Option<&'a Value> {
    if node
        .get("widget")
        .and_then(Value::as_str)
        .map(|v| v == widget)
        .unwrap_or(false)
    {
        return Some(node);
    }

    let children = node.get("children").and_then(Value::as_array)?;
    for child in children {
        if let Some(found) = find_widget_node(child, widget) {
            return Some(found);
        }
    }
    None
}

#[test]
fn build_desktop_reports_parity_warnings_in_stdout() {
    let workspace = TempWorkspace::new("formo_cli_build_desktop_parity");
    create_desktop_parity_gap_sample(workspace.path());

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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("desktop parity warnings: total=1 style=1 widget=0"),
        "expected parity warning summary in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("desktop parity details: dist-desktop/app.native.json"),
        "expected parity diagnostics path in stdout, got: {stdout}"
    );
}

#[test]
fn build_web_strict_parity_fails_when_desktop_parity_gap_exists() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_parity_fail");
    create_desktop_parity_gap_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-web",
            "--strict-parity",
        ])
        .output()
        .expect("should run formo build web strict parity");

    assert!(
        !output.status.success(),
        "expected strict parity failure, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E7600 strict parity failed"),
        "expected strict parity error in stderr, got: {stderr}"
    );
    assert!(
        workspace
            .path()
            .join("dist-web/desktop.parity.json")
            .exists(),
        "expected parity report file for failed strict parity web build"
    );
}

#[test]
fn build_web_strict_parity_passes_when_desktop_parity_is_clean() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_parity_pass");
    create_multifile_sample(workspace.path());

    let output = Command::new(formo_bin())
        .current_dir(workspace.path())
        .args([
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-web",
            "--strict-parity",
        ])
        .output()
        .expect("should run formo build web strict parity");

    assert!(
        output.status.success(),
        "expected strict parity success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        workspace.path().join("dist-web/index.html").exists(),
        "expected web artifacts on strict parity success"
    );
    assert!(
        !workspace
            .path()
            .join("dist-web/desktop.parity.json")
            .exists(),
        "did not expect parity report file when there is no parity warning"
    );
}

#[test]
fn build_web_strict_engine_fails_when_logic_bridge_missing() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_engine_fail");
    create_multifile_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-web",
            "--strict-engine",
        ],
    );
    assert!(
        !output.status.success(),
        "expected strict engine failure, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E7700 strict engine failed"),
        "expected strict engine error in stderr, got: {stderr}"
    );
    let manifest_path = workspace.path().join("dist-web/engine.bridge.json");
    assert!(
        manifest_path.exists(),
        "engine bridge manifest should still be emitted on strict engine failure"
    );
    let raw = fs::read_to_string(manifest_path).expect("manifest should be readable");
    let manifest: Value = serde_json::from_str(&raw).expect("manifest should parse");
    let diagnostics = manifest["diagnostics"]
        .as_array()
        .expect("diagnostics should be array");
    assert!(
        diagnostics
            .iter()
            .any(|diag| diag["code"] == Value::String("W7701".to_string())),
        "expected W7701 diagnostic for missing logic bridge"
    );
}

#[test]
fn build_web_strict_engine_passes_when_logic_bridge_is_ready() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_engine_pass");
    create_multifile_sample_with_logic(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-web",
            "--strict-engine",
        ],
    );
    assert!(
        output.status.success(),
        "expected strict engine success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let raw = fs::read_to_string(workspace.path().join("dist-web/engine.bridge.json"))
        .expect("manifest should be readable");
    let manifest: Value = serde_json::from_str(&raw).expect("manifest should parse");
    assert_eq!(
        manifest["standard"]["fl"]["status"],
        Value::String("ok".to_string())
    );
    assert_eq!(
        manifest["warningCount"],
        Value::from(0),
        "strict engine passing sample should have zero warnings"
    );
}

#[test]
fn build_web_strict_engine_fails_when_fm_action_has_no_matching_fl_event() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_engine_action_mismatch");
    create_action_logic_mismatch_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build",
            "--target",
            "web",
            "--input",
            "main.fm",
            "--out",
            "dist-web",
            "--strict-engine",
        ],
    );
    assert!(
        !output.status.success(),
        "expected strict engine failure, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E7700 strict engine failed"),
        "expected strict engine error in stderr, got: {stderr}"
    );

    let raw = fs::read_to_string(workspace.path().join("dist-web/engine.bridge.json"))
        .expect("manifest should be readable");
    let manifest: Value = serde_json::from_str(&raw).expect("manifest should parse");
    let diagnostics = manifest["diagnostics"]
        .as_array()
        .expect("diagnostics should be array");
    assert!(
        diagnostics
            .iter()
            .any(|diag| diag["code"] == Value::String("W7705".to_string())),
        "expected W7705 diagnostic for FM/FL action binding mismatch"
    );
}

#[test]
fn build_web_strict_bundle_fails_when_logic_bridge_missing() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_bundle_fail");
    create_multifile_sample(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-web", "--strict",
        ],
    );
    assert!(
        !output.status.success(),
        "expected strict bundle failure, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E7700 strict engine failed"),
        "expected strict engine failure in strict bundle, got: {stderr}"
    );
}

#[test]
fn build_web_strict_bundle_passes_when_logic_bridge_is_ready() {
    let workspace = TempWorkspace::new("formo_cli_build_web_strict_bundle_pass");
    create_multifile_sample_with_logic(workspace.path());

    let output = run_formo(
        workspace.path(),
        &[
            "build", "--target", "web", "--input", "main.fm", "--out", "dist-web", "--strict",
        ],
    );
    assert!(
        output.status.success(),
        "expected strict bundle success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let raw = fs::read_to_string(workspace.path().join("dist-web/engine.bridge.json"))
        .expect("manifest should be readable");
    let manifest: Value = serde_json::from_str(&raw).expect("manifest should parse");
    assert_eq!(
        manifest["warningCount"],
        Value::from(0),
        "strict bundle passing sample should have zero warnings"
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
