use super::*;

#[test]
fn parse_component_tree() {
    let src = r#"
component App() {
  <Page>
<Text value="Hello"/>
  </Page>
}
"#;
    let ast = parse(src).expect("parse ok");
    assert_eq!(ast.components.len(), 1);
    assert_eq!(ast.components[0].name, "App");
    assert_eq!(ast.components[0].nodes[0].name, "Page");
    assert_eq!(ast.components[0].nodes[0].children[0].name, "Text");
}

#[test]
fn parse_component_params_typed_and_optional() {
    let src = r#"
component Header(title: string, subtitle?: string) {
  <Text value=title/>
}
"#;
    let ast = parse(src).expect("parse ok");
    let params = &ast.components[0].params;
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "title");
    assert_eq!(params[0].ty.as_deref(), Some("string"));
    assert!(!params[0].optional);
    assert_eq!(params[1].name, "subtitle");
    assert_eq!(params[1].ty.as_deref(), Some("string"));
    assert!(params[1].optional);
}

#[test]
fn parse_list_literal_attribute() {
    let src = r#"
component App() {
  <For each=["A", "B", 3, true] as=item>
<Text value=item/>
  </For>
}
"#;
    let ast = parse(src).expect("parse ok");
    let for_node = &ast.components[0].nodes[0];
    let each_attr = for_node
        .attributes
        .iter()
        .find(|attr| attr.name == "each")
        .expect("each attr exists");
    match &each_attr.value {
        AstValue::List(items) => assert_eq!(items.len(), 4),
        other => panic!("expected list literal, got {other:?}"),
    }
}

#[test]
fn parse_object_literal_in_list() {
    let src = r#"
component App() {
  <For each=[{name: "A", active: true}, {name: "B", active: false}] as=item>
<Text value=item.name/>
  </For>
}
"#;
    let ast = parse(src).expect("parse ok");
    let for_node = &ast.components[0].nodes[0];
    let each_attr = for_node
        .attributes
        .iter()
        .find(|attr| attr.name == "each")
        .expect("each attr exists");
    match &each_attr.value {
        AstValue::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], AstValue::Object(_)));
        }
        other => panic!("expected list literal, got {other:?}"),
    }
}

#[test]
fn reject_raw_text_node() {
    let src = r#"
component App() {
  <Page>
halo
  </Page>
}
"#;
    let err = parse(src).expect_err("raw text should fail");
    assert!(err.contains("raw text node is not supported"));
}

#[test]
fn reject_closing_tag_mismatch() {
    let src = r#"
component App() {
  <Page>
<Text value="x"></Page>
  </Text>
}
"#;
    let err = parse(src).expect_err("mismatched closing tag should fail");
    assert!(err.contains("closing tag mismatch"));
}

#[test]
fn reject_list_trailing_comma() {
    let src = r#"
component App() {
  <For each=["A",] as=item>
<Text value=item/>
  </For>
}
"#;
    let err = parse(src).expect_err("list trailing comma should fail");
    assert!(err.contains("invalid trailing comma in list literal"));
}

#[test]
fn reject_object_duplicate_key() {
    let src = r#"
component App() {
  <For each=[{name: "A", name: "B"}] as=item>
<Text value=item.name/>
  </For>
}
"#;
    let err = parse(src).expect_err("duplicate key should fail");
    assert!(err.contains("duplicate object key"));
}

#[test]
fn reject_object_trailing_comma() {
    let src = r#"
component App() {
  <For each=[{name: "A",}] as=item>
<Text value=item.name/>
  </For>
}
"#;
    let err = parse(src).expect_err("object trailing comma should fail");
    assert!(err.contains("invalid trailing comma in object literal"));
}

#[test]
fn reject_empty_source() {
    let err = parse("   ").expect_err("empty source should fail");
    assert!(err.contains("input source is empty"));
}

#[test]
fn reject_import_without_semicolon() {
    let src = r#"
import "views/header.fm" as Header
component App() {
  <Page/>
}
"#;
    let err = parse(src).expect_err("import without semicolon should fail");
    assert!(err.contains("expected `;`"));
}

#[test]
fn reject_import_alias_with_invalid_identifier() {
    let src = r#"
import "views/header.fm" as 2Header;
component App() {
  <Page/>
}
"#;
    let err = parse(src).expect_err("invalid import alias should fail");
    assert!(err.contains("identifier must start"));
}

#[test]
fn parse_import_with_library_uri() {
    let src = r#"
import "lib://matimatika/core.fm" as MathUi;
import "lib://matimatika/base.fs" as MathStyle;

component App() {
  <Page/>
}
"#;
    let ast = parse(src).expect("library import uri should parse");
    assert_eq!(ast.imports.len(), 2);
    assert_eq!(ast.imports[0].path, "lib://matimatika/core.fm");
    assert_eq!(ast.imports[0].alias.as_deref(), Some("MathUi"));
    assert_eq!(ast.imports[1].path, "lib://matimatika/base.fs");
    assert_eq!(ast.imports[1].alias.as_deref(), Some("MathStyle"));
}

#[test]
fn reject_component_params_trailing_comma() {
    let src = r#"
component App(title: string,) {
  <Text value=title/>
}
"#;
    let err = parse(src).expect_err("params trailing comma should fail");
    assert!(err.contains("invalid trailing comma in component params"));
}

#[test]
fn reject_component_param_missing_type_after_colon() {
    let src = r#"
component App(title:) {
  <Text value="x"/>
}
"#;
    let err = parse(src).expect_err("missing type should fail");
    assert!(err.contains("missing type for parameter `title`"));
}

#[test]
fn reject_component_param_invalid_name() {
    let src = r#"
component App(1title: string) {
  <Text value="x"/>
}
"#;
    let err = parse(src).expect_err("invalid param name should fail");
    assert!(err.contains("invalid parameter name"));
}

#[test]
fn reject_unexpected_closing_tag_at_node_start() {
    let src = r#"
component App() {
  </Page>
}
"#;
    let err = parse(src).expect_err("unexpected closing tag should fail");
    assert!(err.contains("unexpected closing tag"));
}

#[test]
fn reject_attribute_missing_value() {
    let src = r#"
component App() {
  <Text value=/>
}
"#;
    let err = parse(src).expect_err("missing attribute value should fail");
    assert!(err.contains("expected value"));
}

#[test]
fn reject_unterminated_list_literal() {
    let src = r#"
component App() {
  <For each=["A", "B" as=item>
<Text value=item/>
  </For>
}
"#;
    let err = parse(src).expect_err("unterminated list literal should fail");
    assert!(err.contains("expected `,` or `]` in list literal"));
}

#[test]
fn reject_unterminated_object_literal() {
    let src = r#"
component App() {
  <For each=[{name: "A", active: true] as=item>
<Text value=item.name/>
  </For>
}
"#;
    let err = parse(src).expect_err("unterminated object literal should fail");
    assert!(err.contains("expected `,` or `}` in object literal"));
}

#[test]
fn reject_unterminated_string_literal() {
    let src = r#"
component App() {
  <Text value="hello/>
}
"#;
    let err = parse(src).expect_err("unterminated string should fail");
    assert!(err.contains("unterminated string literal"));
}

#[test]
fn recovery_continues_after_invalid_component_and_parses_next_component() {
    let src = r#"
component Broken() {
  <Page>
teks bebas
  </Page>
}

component App() {
  <Page>
<Text value="ok"/>
  </Page>
}
"#;

    let report = parse_with_recovery(src);
    assert!(
        !report.diagnostics.is_empty(),
        "recovery should collect diagnostics"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.contains("raw text node is not supported")),
        "expected raw text diagnostic in recovery output"
    );
    assert_eq!(report.ast.components.len(), 1);
    assert_eq!(report.ast.components[0].name, "App");
}

#[test]
fn recovery_skips_invalid_top_level_tokens_and_keeps_valid_component() {
    let src = r#"
@@@
component App() {
  <Page/>
}
"#;

    let report = parse_with_recovery(src);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.contains("expected `import` or `component`")),
        "expected top-level token diagnostic"
    );
    assert_eq!(report.ast.components.len(), 1);
    assert_eq!(report.ast.components[0].name, "App");
}
