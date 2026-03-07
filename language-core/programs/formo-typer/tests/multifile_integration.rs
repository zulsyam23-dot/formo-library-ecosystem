use formo_parser::parse;
use formo_resolver::resolve;
use formo_typer::{type_check, TypedProgram};
use std::fs;
use std::path::{Path, PathBuf};
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

fn type_check_project(root: &Path, entry: &str) -> Result<TypedProgram, String> {
    let entry_path = root.join(entry);
    let source = fs::read_to_string(&entry_path).expect("entry source should exist");
    let ast = parse(&source).expect("parser should succeed for test source");
    let resolved = resolve(
        ast,
        entry_path
            .to_str()
            .expect("entry path should be valid UTF-8"),
    )?;
    type_check(resolved)
}

#[test]
fn multifile_success_with_style_import() {
    let workspace = TempWorkspace::new("formo_typer_multifile_ok");

    write_file(
        workspace.path(),
        "main.fm",
        r#"import "views/header.fm" as Header;
import "styles/base.fs" as Base;

component App() {
  <Page>
    <Header title="Halo"/>
  </Page>
}
"#,
    );

    write_file(
        workspace.path(),
        "views/header.fm",
        r#"component Header(title: string) {
  <Text value=title/>
}
"#,
    );

    write_file(
        workspace.path(),
        "styles/base.fs",
        r#"style HeaderText {
  color: #112233;
}
"#,
    );

    let result = type_check_project(workspace.path(), "main.fm");
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn multifile_custom_prop_type_mismatch_reports_code() {
    let workspace = TempWorkspace::new("formo_typer_multifile_type_mismatch");

    write_file(
        workspace.path(),
        "main.fm",
        r#"import "views/counter.fm" as Counter;

component App() {
  <Page>
    <Counter value="oops"/>
  </Page>
}
"#,
    );

    write_file(
        workspace.path(),
        "views/counter.fm",
        r#"component Counter(value: int) {
  <Text value="counter"/>
}
"#,
    );

    let err = type_check_project(workspace.path(), "main.fm")
        .expect_err("expected type mismatch across files");
    assert!(
        err.contains("E2303"),
        "expected E2303 for custom prop type mismatch, got: {err}"
    );
}

#[test]
fn multifile_missing_required_custom_prop_reports_code() {
    let workspace = TempWorkspace::new("formo_typer_multifile_missing_prop");

    write_file(
        workspace.path(),
        "main.fm",
        r#"import "views/header.fm" as Header;

component App() {
  <Page>
    <Header/>
  </Page>
}
"#,
    );

    write_file(
        workspace.path(),
        "views/header.fm",
        r#"component Header(title: string) {
  <Text value=title/>
}
"#,
    );

    let err = type_check_project(workspace.path(), "main.fm")
        .expect_err("expected missing required prop across files");
    assert!(
        err.contains("E2301"),
        "expected E2301 for missing required custom prop, got: {err}"
    );
}
