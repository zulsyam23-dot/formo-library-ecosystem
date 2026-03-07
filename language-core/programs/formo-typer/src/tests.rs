use super::error_codes as code;
use super::*;
use formo_parser::parse;
use formo_resolver::resolve;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn check_source(source: &str) -> Result<TypedProgram, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("formo_typer_{stamp}.fm"));
    fs::write(&path, source).expect("should write temporary source");

    let ast = parse(source).expect("parser should succeed for test source");
    let path_str = path.to_string_lossy().to_string();
    let resolved = resolve(ast, &path_str).expect("resolver should succeed for test source");
    let result = type_check(resolved);
    let _ = fs::remove_file(path);
    result
}

fn assert_error_has_code(result: Result<TypedProgram, String>, expected_code: &str) {
    let err = result.expect_err("expected type_check error");
    assert!(
        err.contains(expected_code),
        "expected error code {expected_code}, got: {err}"
    );
}

#[test]
fn for_alias_can_be_used_as_text_value() {
    let source = r#"
component App(items: list<string>) {
  <For each=items as=item>
<Text value=item/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn for_alias_index_can_be_used_for_int_prop() {
    let source = r#"
component App(items: list<string>) {
  <For each=items as=item>
<Text value="row" maxLines=itemIndex/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn for_alias_cannot_be_used_as_action() {
    let source = r#"
component App(items: list<string>) {
  <For each=items as=item>
<Button label="Run" onPress=item/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_err(), "expected type error");
}

#[test]
fn for_alias_from_literal_keeps_item_type() {
    let source = r#"
component App() {
  <For each=["A", "B"] as=item>
<Text value="row" maxLines=item/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_err(), "expected type error");
}

#[test]
fn for_alias_object_field_can_be_used() {
    let source = r#"
component App() {
  <For each=[{name: "A", active: true}, {name: "B", active: false}] as=item>
<Text value=item.name/>
<If when=item.active>
  <Text value="active"/>
</If>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn for_alias_object_field_type_is_checked() {
    let source = r#"
component App() {
  <For each=[{name: "A"}, {name: "B"}] as=item>
<Text value="row" maxLines=item.name/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_err(), "expected type error");
}

#[test]
fn for_alias_object_list_index_can_be_used() {
    let source = r#"
component App() {
  <For each=[{tags: ["A", "B"]}, {tags: ["C", "D"]}] as=item>
<Text value=item.tags.0/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn for_alias_object_list_index_type_is_checked() {
    let source = r#"
component App() {
  <For each=[{tags: ["A", "B"]}, {tags: ["C", "D"]}] as=item>
<Text value="row" maxLines=item.tags.0/>
  </For>
}
"#;

    let result = check_source(source);
    assert!(result.is_err(), "expected type error");
}

#[test]
fn typed_list_index_can_be_used_for_int_prop() {
    let source = r#"
component App(items: list<int>) {
  <Text value="row" maxLines=items.0/>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn typed_list_index_type_is_checked() {
    let source = r#"
component App(items: list<string>) {
  <Text value="row" maxLines=items.0/>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_INVALID_PROP_TYPE);
}

#[test]
fn typed_nested_list_index_can_be_used() {
    let source = r#"
component App(matrix: list<list<string>>) {
  <Text value=matrix.0.1/>
}
"#;

    let result = check_source(source);
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
}

#[test]
fn invalid_field_path_on_list_param_is_rejected() {
    let source = r#"
component App(items: list<string>) {
  <Text value=items.name/>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_INVALID_PROP_TYPE);
}

#[test]
fn detects_unknown_builtin_prop_with_code() {
    let source = r#"
component App() {
  <Text value="hello" nope="x"/>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_UNKNOWN_PROP);
}

#[test]
fn detects_missing_builtin_required_prop_with_code() {
    let source = r#"
component App() {
  <Button onPress=run/>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_REQUIRED_PROP_MISSING);
}

#[test]
fn detects_duplicate_attribute_with_code() {
    let source = r#"
component App() {
  <Text value="a" value="b"/>
}
"#;

    assert_error_has_code(check_source(source), code::ATTR_DUPLICATE);
}

#[test]
fn detects_custom_unknown_prop_with_code() {
    let source = r#"
component Header(title: string) {
  <Text value=title/>
}

component App() {
  <Header title="x" extra="y"/>
}
"#;

    assert_error_has_code(check_source(source), code::CUSTOM_UNKNOWN_PROP);
}

#[test]
fn detects_custom_inline_children_without_slot_with_code() {
    let source = r#"
component Header(title: string) {
  <Text value=title/>
}

component App() {
  <Header title="x">
    <Text value="child"/>
  </Header>
}
"#;

    assert_error_has_code(check_source(source), code::CUSTOM_INLINE_CHILDREN_FORBIDDEN);
}

#[test]
fn detects_duplicate_component_param_with_code() {
    let source = r#"
component App(title: string, title: string) {
  <Text value=title/>
}
"#;

    assert_error_has_code(check_source(source), code::COMPONENT_DUPLICATE_PARAM);
}

#[test]
fn detects_component_empty_root_with_code() {
    let source = r#"
component App() {
}
"#;

    assert_error_has_code(check_source(source), code::COMPONENT_EMPTY_ROOT);
}

#[test]
fn detects_component_multi_root_with_code() {
    let source = r#"
component App() {
  <Page/>
  <Page/>
}
"#;

    assert_error_has_code(check_source(source), code::COMPONENT_MULTI_ROOT);
}

#[test]
fn detects_node_name_must_start_uppercase_with_code() {
    let source = r#"
component App() {
  <page/>
}
"#;

    assert_error_has_code(check_source(source), code::NODE_NAME_UPPERCASE);
}

#[test]
fn detects_unknown_node_with_code() {
    let source = r#"
component App() {
  <Dashboard/>
}
"#;

    assert_error_has_code(check_source(source), code::NODE_UNKNOWN);
}

#[test]
fn detects_builtin_invalid_prop_type_with_code() {
    let source = r#"
component App() {
  <Text value="hello" maxLines="2"/>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_INVALID_PROP_TYPE);
}

#[test]
fn detects_builtin_children_forbidden_with_code() {
    let source = r#"
component App() {
  <Text value="parent">
    <Text value="child"/>
  </Text>
}
"#;

    assert_error_has_code(check_source(source), code::BUILTIN_CHILDREN_FORBIDDEN);
}

#[test]
fn detects_style_on_slot_forbidden_with_code() {
    let source = r#"
component Header() {
  <Page>
    <Slot style=BodyText/>
  </Page>
}

component App() {
  <Header/>
}
"#;

    assert_error_has_code(check_source(source), code::STYLE_SLOT_ATTR_FORBIDDEN);
}

#[test]
fn detects_empty_style_ref_with_code() {
    let source = r#"
component App() {
  <Text value="x" style="   "/>
}
"#;

    assert_error_has_code(check_source(source), code::STYLE_EMPTY);
}

#[test]
fn detects_invalid_style_type_with_code() {
    let source = r#"
component App() {
  <Text value="x" style=true/>
}
"#;

    assert_error_has_code(check_source(source), code::STYLE_INVALID_TYPE);
}

#[test]
fn detects_custom_missing_required_prop_with_code() {
    let source = r#"
component Header(title: string) {
  <Text value=title/>
}

component App() {
  <Header/>
}
"#;

    assert_error_has_code(check_source(source), code::CUSTOM_REQUIRED_PROP_MISSING);
}

#[test]
fn detects_custom_prop_type_mismatch_with_code() {
    let source = r#"
component Counter(value: int) {
  <Text value="counter"/>
}

component App() {
  <Counter value="oops"/>
}
"#;

    assert_error_has_code(check_source(source), code::CUSTOM_PROP_TYPE_MISMATCH);
}
