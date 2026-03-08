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
