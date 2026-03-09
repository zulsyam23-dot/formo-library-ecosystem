pub(crate) use serde_json::Value;
pub(crate) use std::fs;
use std::path::{Path, PathBuf};
pub(crate) use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    pub(crate) fn new(prefix: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), stamp));
        fs::create_dir_all(&root).expect("should create temp workspace");
        Self { root }
    }
    pub(crate) fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn write_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("should create parent directory");
    }
    fs::write(path, content).expect("should write file");
}

pub(crate) fn formo_bin() -> PathBuf {
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

pub(crate) fn run_formo(workspace: &Path, args: &[&str]) -> Output {
    Command::new(formo_bin())
        .current_dir(workspace)
        .args(args)
        .output()
        .expect("should run formo command")
}

pub(crate) fn assert_no_rust_panic(output: &Output) {
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

pub(crate) fn create_multifile_sample(root: &Path) {
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

pub(crate) fn create_multifile_sample_with_logic(root: &Path) {
    create_multifile_sample(root);
    write_file(
        root,
        "logic/controllers/app_controller.fl",
        r#"module AppController;

logic AppController {
  event startApp {
    action emit "READY";
  }
}
"#,
    );
}

pub(crate) fn create_desktop_parity_gap_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="halo" style=GapStyle/>
  </Page>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r#"style GapStyle {
  position: absolute;
}
"#,
    );
}

pub(crate) fn create_cycle_sample(root: &Path) {
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

pub(crate) fn create_unknown_style_sample(root: &Path) {
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

pub(crate) fn create_invalid_style_module_sample(root: &Path) {
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

pub(crate) fn create_unused_token_style_sample(root: &Path) {
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

pub(crate) fn create_recursive_component_sample(root: &Path) {
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

pub(crate) fn create_canonical_style_parity_sample(root: &Path) {
    write_file(
        root,
        "main.fm",
        r#"import "styles/base.fs" as Base;

component App() {
  <Page>
    <Row style=FeedRow>
      <Text value="A"/>
      <Text value="B"/>
    </Row>
  </Page>
}
"#,
    );

    write_file(
        root,
        "styles/base.fs",
        r#"style FeedRow {
  align-items: baseline;
  justify-content: space-around;
}
"#,
    );
}

pub(crate) fn create_parser_recovery_sample(root: &Path) {
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
