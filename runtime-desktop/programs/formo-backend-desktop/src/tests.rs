use super::*;
use formo_ir::{IrComponent, IrNode, StyleSelector, Target, Value};

fn source() -> SourceLoc {
    SourceLoc {
        file: "main.fm".to_string(),
        line: 1,
        col: 1,
    }
}

fn sample_ir() -> IrProgram {
    let mut node_props = BTreeMap::new();
    node_props.insert(
        "value".to_string(),
        Value {
            t: "string".to_string(),
            v: serde_json::Value::String("hello".to_string()),
        },
    );

    let mut style_decls = BTreeMap::new();
    style_decls.insert(
        "color".to_string(),
        Value {
            t: "string".to_string(),
            v: serde_json::Value::String("#112233".to_string()),
        },
    );

    IrProgram {
        ir_version: "0.3.0".to_string(),
        entry: "App".to_string(),
        target: Target::Desktop,
        tokens: BTreeMap::new(),
        components: vec![IrComponent {
            id: "c_app".to_string(),
            name: "App".to_string(),
            root_node_id: "n_root".to_string(),
            exports: true,
            source: source(),
        }],
        nodes: vec![IrNode {
            id: "n_root".to_string(),
            kind: "element".to_string(),
            name: "Text".to_string(),
            props: node_props,
            style_refs: vec!["TextBase".to_string()],
            children: vec![],
            source: source(),
        }],
        styles: vec![formo_ir::IrStyle {
            id: "TextBase".to_string(),
            selector: StyleSelector {
                component: "Text".to_string(),
                part: "root".to_string(),
            },
            decls: style_decls,
            canonical_decls: BTreeMap::new(),
        }],
        diagnostics: vec![],
    }
}

fn sample_ir_with_parity_gaps() -> IrProgram {
    let mut style_decls = BTreeMap::new();
    style_decls.insert(
        "position".to_string(),
        Value {
            t: "string".to_string(),
            v: serde_json::Value::String("absolute".to_string()),
        },
    );

    IrProgram {
        ir_version: "0.3.0".to_string(),
        entry: "App".to_string(),
        target: Target::Desktop,
        tokens: BTreeMap::new(),
        components: vec![IrComponent {
            id: "c_app".to_string(),
            name: "App".to_string(),
            root_node_id: "n_root".to_string(),
            exports: true,
            source: source(),
        }],
        nodes: vec![IrNode {
            id: "n_root".to_string(),
            kind: "element".to_string(),
            name: "UnknownPanel".to_string(),
            props: BTreeMap::new(),
            style_refs: vec!["GapStyle".to_string()],
            children: vec![],
            source: source(),
        }],
        styles: vec![formo_ir::IrStyle {
            id: "GapStyle".to_string(),
            selector: StyleSelector {
                component: "UnknownPanel".to_string(),
                part: "root".to_string(),
            },
            decls: style_decls,
            canonical_decls: BTreeMap::new(),
        }],
        diagnostics: vec![],
    }
}

fn sample_ir_with_actions() -> IrProgram {
    let button_props = BTreeMap::from([
        (
            "label".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("Save".to_string()),
            },
        ),
        (
            "onPress".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("saveForm".to_string()),
            },
        ),
    ]);

    let input_props = BTreeMap::from([
        (
            "name".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("query".to_string()),
            },
        ),
        (
            "onChange".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("search:update".to_string()),
            },
        ),
    ]);

    let button_dup_props = BTreeMap::from([(
        "onClick".to_string(),
        Value {
            t: "string".to_string(),
            v: serde_json::Value::String("saveForm".to_string()),
        },
    )]);

    IrProgram {
        ir_version: "0.3.0".to_string(),
        entry: "App".to_string(),
        target: Target::Desktop,
        tokens: BTreeMap::new(),
        components: vec![IrComponent {
            id: "c_app".to_string(),
            name: "App".to_string(),
            root_node_id: "n_root".to_string(),
            exports: true,
            source: source(),
        }],
        nodes: vec![
            IrNode {
                id: "n_root".to_string(),
                kind: "element".to_string(),
                name: "Page".to_string(),
                props: BTreeMap::new(),
                style_refs: vec![],
                children: vec![
                    "n_button".to_string(),
                    "n_input".to_string(),
                    "n_button_dup".to_string(),
                ],
                source: source(),
            },
            IrNode {
                id: "n_button".to_string(),
                kind: "element".to_string(),
                name: "Button".to_string(),
                props: button_props,
                style_refs: vec![],
                children: vec![],
                source: source(),
            },
            IrNode {
                id: "n_input".to_string(),
                kind: "element".to_string(),
                name: "Input".to_string(),
                props: input_props,
                style_refs: vec![],
                children: vec![],
                source: source(),
            },
            IrNode {
                id: "n_button_dup".to_string(),
                kind: "element".to_string(),
                name: "Button".to_string(),
                props: button_dup_props,
                style_refs: vec![],
                children: vec![],
                source: source(),
            },
        ],
        styles: vec![],
        diagnostics: vec![],
    }
}

fn file<'a>(output: &'a BackendOutput, path: &str) -> &'a str {
    output
        .files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.content.as_str())
        .expect("file should exist")
}

#[test]
fn desktop_backend_emits_native_bundle_plus_ir() {
    let output = DesktopBackend
        .emit(&sample_ir())
        .expect("desktop emit should succeed");

    assert!(output.files.iter().any(|f| f.path == "app.native.json"));
    assert!(output.files.iter().any(|f| f.path == "app.native.rs"));
    assert!(output.files.iter().any(|f| f.path == "app.ir.json"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/Cargo.toml"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/main.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/actions.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/app.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/model.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/style.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/mod.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/shared.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/state.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/flow.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/controls.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "native-app/src/render/media.rs"));
    assert!(output.files.iter().any(|f| f.path == "readable/README.md"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "readable/native/components.json"));
    assert!(output
        .files
        .iter()
        .any(|f| f.path == "readable/ir/nodes.json"));

    assert!(!output.files.iter().any(|f| f.path == "index.html"));
    assert!(!output.files.iter().any(|f| f.path == "app.js"));
    assert!(!output.files.iter().any(|f| f.path == "app.css"));
    assert!(!output.files.iter().any(|f| f.path == "desktop-bridge.js"));
}

#[test]
fn native_json_contains_resolved_widget_tree() {
    let output = DesktopBackend
        .emit(&sample_ir())
        .expect("desktop emit should succeed");

    let native: serde_json::Value =
        serde_json::from_str(file(&output, "app.native.json")).expect("valid json");

    assert_eq!(
        native["runtime"],
        serde_json::Value::String("rust-native".to_string())
    );
    assert_eq!(
        native["entryComponent"],
        serde_json::Value::String("App".to_string())
    );
    assert_eq!(
        native["components"][0]["rootNode"]["widget"],
        serde_json::Value::String("Text".to_string())
    );
    assert_eq!(
        native["components"][0]["rootNode"]["resolvedStyle"]["color"]["v"],
        serde_json::Value::String("#112233".to_string())
    );
}

#[test]
fn rust_runtime_stub_exposes_host_and_state_bridge() {
    let output = DesktopBackend
        .emit(&sample_ir())
        .expect("desktop emit should succeed");

    let stub = file(&output, "app.native.rs");
    assert!(stub.contains("pub trait FormoDesktopHost"));
    assert!(stub.contains("pub struct FormoDesktopAction"));
    assert!(stub.contains("pub struct FormoDesktopState"));
    assert!(stub.contains("FORMO_ENTRY_COMPONENT"));
}

#[test]
fn native_scaffold_uses_dioxus_dom_and_embeds_bundle_path() {
    let output = DesktopBackend
        .emit(&sample_ir())
        .expect("desktop emit should succeed");

    let cargo = file(&output, "native-app/Cargo.toml");
    let main_rs = file(&output, "native-app/src/main.rs");
    let actions_rs = file(&output, "native-app/src/actions.rs");
    let app_rs = file(&output, "native-app/src/app.rs");
    let model_rs = file(&output, "native-app/src/model.rs");
    let style_rs = file(&output, "native-app/src/style.rs");
    let render_mod_rs = file(&output, "native-app/src/render/mod.rs");
    let render_shared_rs = file(&output, "native-app/src/render/shared.rs");
    let render_state_rs = file(&output, "native-app/src/render/state.rs");
    let render_flow_rs = file(&output, "native-app/src/render/flow.rs");
    let render_controls_rs = file(&output, "native-app/src/render/controls.rs");
    let render_media_rs = file(&output, "native-app/src/render/media.rs");

    assert!(cargo.contains("dioxus"));
    assert!(cargo.contains("dioxus-desktop"));
    assert!(main_rs.contains("mod app;"));
    assert!(main_rs.contains("mod actions;"));
    assert!(actions_rs.contains("pub struct ActionEvent"));
    assert!(actions_rs.contains("pub fn invoke("));
    assert!(actions_rs.contains("pub fn set_state("));
    assert!(actions_rs.contains("pub fn eval_set_expression("));
    assert!(actions_rs.contains("pub fn eval_set_expression_rpn("));
    assert!(main_rs.contains("mod style;"));
    assert!(main_rs.contains("mod render;"));
    assert!(app_rs.contains("include_str!(\"../../app.native.json\")"));
    assert!(app_rs.contains("dioxus_desktop::launch(AppRoot);"));
    assert!(app_rs.contains("use_signal"));
    assert!(app_rs.contains("Action log"));
    assert!(app_rs.contains("render_node("));
    assert!(app_rs.contains("Desktop parity warnings:"));
    assert!(model_rs.contains("pub diagnostics: Vec<NativeDiagnostic>"));
    assert!(style_rs.contains("pub fn runtime_css()"));
    assert!(style_rs.contains("pub fn widget_class("));
    assert!(style_rs.contains("pub fn style_attr("));
    assert!(render_mod_rs.contains("pub fn render_node("));
    assert!(render_mod_rs.contains("controls::render_checkbox_html("));
    assert!(render_mod_rs.contains("controls::render_switch_html("));
    assert!(render_mod_rs.contains("media::render_modal_html("));
    assert!(render_mod_rs.contains("flow::render_if_html("));
    assert!(render_mod_rs.contains("flow::render_for_html("));
    assert!(render_shared_rs.contains("pub(super) fn node_class("));
    assert!(render_shared_rs.contains("pub(super) fn node_style("));
    assert!(render_shared_rs.contains("pub(super) fn escape_attr("));
    assert!(render_state_rs.contains("fn resolve_scoped_value("));
    assert!(render_state_rs.contains("fn derive_for_item_key("));
    assert!(render_state_rs.contains("fn dispatch_action("));
    assert!(render_state_rs.contains("crate::actions::invoke("));
    assert!(render_flow_rs.contains("fn render_container_html("));
    assert!(render_flow_rs.contains("fn render_if_html("));
    assert!(render_flow_rs.contains("fn render_for_html("));
    assert!(render_flow_rs.contains("{}Key"));
    assert!(render_controls_rs.contains("fn render_input_html("));
    assert!(render_controls_rs.contains("fn render_switch_html("));
    assert!(render_controls_rs.contains("fn render_button_html("));
    assert!(render_media_rs.contains("fn render_modal_html("));
    assert!(render_media_rs.contains("formo-modal-body"));
}

#[test]
fn native_scaffold_actions_registry_is_generated_from_ir_props() {
    let output = DesktopBackend
        .emit(&sample_ir_with_actions())
        .expect("desktop emit should succeed");

    let actions_rs = file(&output, "native-app/src/actions.rs");
    assert!(actions_rs.contains("\"saveForm\" => handle_save_form"));
    assert!(actions_rs.contains("\"search:update\" => handle_search_update"));
    assert!(actions_rs.contains("fn handle_save_form("));
    assert!(actions_rs.contains("fn handle_search_update("));
    assert_eq!(actions_rs.matches("\"saveForm\" =>").count(), 1);
}

#[test]
fn native_json_includes_desktop_parity_warnings_for_unsupported_items() {
    let output = DesktopBackend
        .emit(&sample_ir_with_parity_gaps())
        .expect("desktop emit should succeed");

    let native: serde_json::Value =
        serde_json::from_str(file(&output, "app.native.json")).expect("valid json");
    let diagnostics = native["diagnostics"]
        .as_array()
        .expect("diagnostics should be array");

    assert!(
        diagnostics.iter().any(|d| d["code"] == "W7601"),
        "expected style parity warning W7601"
    );
    assert!(
        diagnostics.iter().any(|d| d["code"] == "W7602"),
        "expected widget parity warning W7602"
    );
}

#[test]
fn kebab_case_name_is_stable_for_entry_component() {
    assert_eq!(to_kebab_case("AppMain"), "app-main");
    assert_eq!(to_kebab_case("app main"), "app-main");
    assert_eq!(to_kebab_case(""), "formo-desktop-app");
}

#[test]
fn snake_case_name_is_stable_for_action_handlers() {
    assert_eq!(to_snake_case("saveForm"), "save_form");
    assert_eq!(to_snake_case("search:update"), "search_update");
    assert_eq!(to_snake_case(""), "");
}
