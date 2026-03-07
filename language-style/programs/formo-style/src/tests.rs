use crate::compile_styles;
use crate::parser::parse_style_module;
use crate::StyledProgram;
use formo_parser::parse;
use formo_resolver::resolve;
use formo_typer::type_check;
use serde_json::json;
use std::collections::BTreeMap;
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

fn compile_project_styles(root: &Path, entry: &str) -> StyledProgram {
    let entry_path = root.join(entry);
    let source = fs::read_to_string(&entry_path).expect("entry source should exist");
    let ast = parse(&source).expect("parser should succeed");
    let resolved = resolve(ast, entry_path.to_str().expect("entry path should be utf-8"))
        .expect("resolver should succeed");
    let typed = type_check(resolved).expect("type-check should succeed");
    compile_styles(typed).expect("style compilation should succeed")
}

#[test]
fn parse_style_file() {
    let src = r#"
style HeaderCard {
  background: #0A84FF;
  padding: 12dp;
}

style HeaderCard:title {
  color: #FFFFFF;
}
"#;
    let parsed = parse_style_module(src, "test.fs", &BTreeMap::new()).expect("parse ok");
    assert_eq!(parsed.styles.len(), 2);
    assert_eq!(parsed.styles[0].id, "HeaderCard");
    assert_eq!(parsed.styles[1].id, "HeaderCard:title");
    assert!(parsed.styles[0].decls.contains_key("background"));
}

#[test]
fn parse_tokens_and_resolve_in_style() {
    let src = r#"
token {
  color.primary = #0A84FF;
  space.md = 12dp;
}

style Heading {
  color: token(color.primary);
  padding: token(space.md);
}
"#;
    let parsed = parse_style_module(src, "test.fs", &BTreeMap::new()).expect("parse ok");
    assert_eq!(parsed.tokens.len(), 2);
    assert_eq!(parsed.styles.len(), 1);

    let color = parsed.styles[0].decls.get("color").expect("color exists");
    assert_eq!(color.t, "color");
    assert_eq!(color.v.as_str(), Some("#0A84FF"));
}

#[test]
fn parse_token_with_literal_fallback() {
    let src = r#"
style Heading {
  color: token(color.primary, #0A84FF);
}
"#;
    let parsed = parse_style_module(src, "test.fs", &BTreeMap::new()).expect("parse ok");
    let color = parsed.styles[0].decls.get("color").expect("color exists");
    assert_eq!(color.t, "color");
    assert_eq!(color.v.as_str(), Some("#0A84FF"));
}

#[test]
fn parse_token_with_token_fallback() {
    let src = r#"
token {
  color.fallback = #2266AA;
}

style Heading {
  color: token(color.primary, token(color.fallback));
}
"#;
    let parsed = parse_style_module(src, "test.fs", &BTreeMap::new()).expect("parse ok");
    let color = parsed.styles[0].decls.get("color").expect("color exists");
    assert_eq!(color.t, "color");
    assert_eq!(color.v.as_str(), Some("#2266AA"));
}

#[test]
fn reject_unknown_token_without_fallback() {
    let src = r#"
style Heading {
  color: token(color.primary);
}
"#;
    let err =
        parse_style_module(src, "test.fs", &BTreeMap::new()).expect_err("unknown token should fail");
    assert!(
        err.contains("unknown token `color.primary`"),
        "unexpected error: {err}"
    );
}

#[test]
fn reject_unknown_style_property() {
    let src = r#"
style HeaderCard {
  not-supported: #0A84FF;
}
"#;
    let err = parse_style_module(src, "test.fs", &BTreeMap::new())
        .expect_err("unknown style property should fail");
    assert!(
        err.contains("unknown style property `not-supported`"),
        "unexpected error: {err}"
    );
}

#[test]
fn reject_unused_token() {
    let src = r#"
token {
  space.md = 12dp;
}

style Heading {
  color: #112233;
}
"#;
    let err =
        parse_style_module(src, "test.fs", &BTreeMap::new()).expect_err("unused token should fail");
    assert!(err.contains("E1304"), "expected E1304, got: {err}");
    assert!(err.contains("space.md"), "unexpected error: {err}");
}

#[test]
fn compile_styles_snapshot_is_stable() {
    let workspace = TempWorkspace::new("formo_style_snapshot");
    write_file(
        &workspace.root,
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
        &workspace.root,
        "styles/base.fs",
        r#"token {
  space.md = 12dp;
  color.primary = #0A84FF;
}

style HeaderCard {
  padding: token(space.md);
  color: token(color.primary);
}

style HeaderCard:title {
  font-size: 16;
  font-weight: 700;
}
"#,
    );

    let styled_first = compile_project_styles(&workspace.root, "main.fm");
    let styled_second = compile_project_styles(&workspace.root, "main.fm");

    let snapshot_first = serde_json::to_string_pretty(&json!({
        "tokens": styled_first.tokens,
        "styles": styled_first.styles,
    }))
    .expect("snapshot should serialize");

    let snapshot_second = serde_json::to_string_pretty(&json!({
        "tokens": styled_second.tokens,
        "styles": styled_second.styles,
    }))
    .expect("snapshot should serialize");

    assert_eq!(
        snapshot_first, snapshot_second,
        "snapshot must be deterministic"
    );

    let expected = r##"{
  "styles": [
    {
      "decls": {
        "color": {
          "t": "color",
          "v": "#0A84FF"
        },
        "padding": {
          "t": "len",
          "v": {
            "unit": "dp",
            "value": 12.0
          }
        }
      },
      "id": "HeaderCard",
      "selector": {
        "component": "HeaderCard",
        "part": "root"
      }
    },
    {
      "decls": {
        "font-size": {
          "t": "int",
          "v": 16
        },
        "font-weight": {
          "t": "int",
          "v": 700
        }
      },
      "id": "HeaderCard:title",
      "selector": {
        "component": "HeaderCard",
        "part": "title"
      }
    }
  ],
  "tokens": {
    "color.primary": {
      "t": "color",
      "v": "#0A84FF"
    },
    "space.md": {
      "t": "len",
      "v": {
        "unit": "dp",
        "value": 12.0
      }
    }
  }
}"##;
    assert_eq!(snapshot_first, expected);
}
