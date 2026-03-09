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
        }],
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
fn native_scaffold_uses_egui_and_embeds_bundle_path() {
    let output = DesktopBackend
        .emit(&sample_ir())
        .expect("desktop emit should succeed");

    let cargo = file(&output, "native-app/Cargo.toml");
    let main_rs = file(&output, "native-app/src/main.rs");
    let app_rs = file(&output, "native-app/src/app.rs");
    let model_rs = file(&output, "native-app/src/model.rs");
    let style_rs = file(&output, "native-app/src/style.rs");
    let render_mod_rs = file(&output, "native-app/src/render/mod.rs");
    let render_shared_rs = file(&output, "native-app/src/render/shared.rs");
    let render_state_rs = file(&output, "native-app/src/render/state.rs");
    let render_flow_rs = file(&output, "native-app/src/render/flow.rs");
    let render_controls_rs = file(&output, "native-app/src/render/controls.rs");
    let render_media_rs = file(&output, "native-app/src/render/media.rs");

    assert!(cargo.contains("eframe"));
    assert!(main_rs.contains("mod app;"));
    assert!(main_rs.contains("mod style;"));
    assert!(main_rs.contains("mod render;"));
    assert!(app_rs.contains("include_str!(\"../../app.native.json\")"));
    assert!(app_rs.contains("struct FormoNativeApp"));
    assert!(app_rs.contains("impl eframe::App for FormoNativeApp"));
    assert!(app_rs.contains("fn configure_theme("));
    assert!(app_rs.contains("ctx.set_visuals(visuals);"));
    assert!(app_rs.contains("visuals.widgets.inactive.rounding"));
    assert!(app_rs.contains("TextStyle::Body"));
    assert!(app_rs.contains("Desktop parity warnings:"));
    assert!(model_rs.contains("pub diagnostics: Vec<NativeDiagnostic>"));
    assert!(style_rs.contains("pub struct RenderStyle"));
    assert!(style_rs.contains("fn parse_rgb_color("));
    assert!(style_rs.contains("fn parse_font_weight("));
    assert!(style_rs.contains("fn parse_len_px("));
    assert!(style_rs.contains("fn parse_box_shadow("));
    assert!(style_rs.contains("fn parse_border_shorthand("));
    assert!(style_rs.contains("map_or("));
    assert!(style_rs.contains("parse_align(raw).unwrap_or(AlignMode::Start)"));
    assert!(style_rs.contains("\"baseline\""));
    assert!(style_rs.contains("\"self-start\""));
    assert!(style_rs.contains("\"self-end\""));
    assert!(style_rs.contains("pub display_flex: bool"));
    assert!(render_mod_rs.contains("\"Checkbox\" =>"));
    assert!(render_mod_rs.contains("\"Switch\" =>"));
    assert!(render_mod_rs.contains("\"Modal\" =>"));
    assert!(render_mod_rs.contains("\"Image\" =>"));
    assert!(render_mod_rs.contains("\"If\" =>"));
    assert!(render_mod_rs.contains("\"For\" =>"));
    assert!(render_mod_rs.contains("\"Stack\" =>"));
    assert!(render_mod_rs.contains("flow::render_block("));
    assert!(render_mod_rs.contains("\"Scroll\" => flow::render_scroll("));
    assert!(render_shared_rs.contains("fn layout_from_style("));
    assert!(render_shared_rs.contains("let has_explicit_main_size = match flow"));
    assert!(render_shared_rs.contains("JustifyMode::Space if has_explicit_main_size"));
    assert!(render_shared_rs.contains("fn apply_gap("));
    assert!(render_state_rs.contains("fn emit_action("));
    assert!(render_state_rs.contains("fn derive_for_item_key("));
    assert!(render_state_rs.contains("fn prop_usize("));
    assert!(render_flow_rs.contains("fn render_block("));
    assert!(render_flow_rs.contains("fn render_scroll("));
    assert!(render_flow_rs.contains("apply_gap(ui, style.gap, None);"));
    assert!(render_flow_rs.contains("fn render_if("));
    assert!(render_flow_rs.contains("fn render_for("));
    assert!(render_flow_rs.contains("{}Key"));
    assert!(render_controls_rs.contains("fn render_input("));
    assert!(render_media_rs.contains("fn render_modal("));
    assert!(render_media_rs.contains("egui::Key::Escape"));
    assert!(render_media_rs.contains("egui::Sense::click()"));
    assert!(render_media_rs.contains(".max_width(max_modal_width)"));
    assert!(render_media_rs.contains("ScrollArea::vertical()"));
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
