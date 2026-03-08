use formo_ir::{
    Backend, BackendOutput, Diagnostic, IrNode, IrProgram, IrStyle, OutputFile, SourceLoc, Value,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const NATIVE_RUNTIME_STUB_RS: &str = include_str!("native_runtime_stub.rs");
const NATIVE_APP_CARGO_TOML: &str = include_str!("native_app_cargo_toml.tpl");
const NATIVE_APP_MAIN_RS: &str = include_str!("native_app_main_rs.tpl");
const NATIVE_APP_APP_RS: &str = include_str!("native_app_app_rs.tpl");
const NATIVE_APP_MODEL_RS: &str = include_str!("native_app_model_rs.tpl");
const NATIVE_APP_STYLE_RS: &str = include_str!("native_app_style_rs.tpl");
const NATIVE_APP_RENDER_MOD_RS: &str = include_str!("native_app_render_mod_rs.tpl");
const NATIVE_APP_RENDER_SHARED_RS: &str = include_str!("native_app_render_shared_rs.tpl");
const NATIVE_APP_RENDER_STATE_RS: &str = include_str!("native_app_render_state_rs.tpl");
const NATIVE_APP_RENDER_FLOW_RS: &str = include_str!("native_app_render_flow_rs.tpl");
const NATIVE_APP_RENDER_CONTROLS_RS: &str = include_str!("native_app_render_controls_rs.tpl");
const NATIVE_APP_RENDER_MEDIA_RS: &str = include_str!("native_app_render_media_rs.tpl");
const NATIVE_APP_README_MD: &str = include_str!("native_app_readme_md.tpl");

pub struct DesktopBackend;

impl Backend for DesktopBackend {
    fn emit(&self, ir: &IrProgram) -> Result<BackendOutput, String> {
        let ir_snapshot = serde_json::to_string_pretty(ir).map_err(|e| e.to_string())?;
        let native_bundle = NativeDesktopBundle::from_ir(ir)?;
        let native_bundle_json =
            serde_json::to_string_pretty(&native_bundle).map_err(|e| e.to_string())?;
        let runtime_stub = render_native_runtime_stub(&native_bundle.entry_component);
        let native_app_cargo_toml = render_native_app_cargo_toml(&native_bundle.entry_component);
        let native_app_main_rs = render_native_app_main_rs();
        let native_app_app_rs = render_native_app_app_rs(&native_bundle.entry_component);
        let native_app_model_rs = render_native_app_model_rs();
        let native_app_style_rs = render_native_app_style_rs();
        let native_app_render_mod_rs = render_native_app_render_mod_rs();
        let native_app_render_shared_rs = render_native_app_render_shared_rs();
        let native_app_render_state_rs = render_native_app_render_state_rs();
        let native_app_render_flow_rs = render_native_app_render_flow_rs();
        let native_app_render_controls_rs = render_native_app_render_controls_rs();
        let native_app_render_media_rs = render_native_app_render_media_rs();
        let native_app_readme = render_native_app_readme_md(&native_bundle.entry_component);

        Ok(BackendOutput {
            files: vec![
                OutputFile {
                    path: "app.native.json".to_string(),
                    content: native_bundle_json,
                },
                OutputFile {
                    path: "app.native.rs".to_string(),
                    content: runtime_stub,
                },
                OutputFile {
                    path: "app.ir.json".to_string(),
                    content: ir_snapshot,
                },
                OutputFile {
                    path: "native-app/Cargo.toml".to_string(),
                    content: native_app_cargo_toml,
                },
                OutputFile {
                    path: "native-app/src/main.rs".to_string(),
                    content: native_app_main_rs,
                },
                OutputFile {
                    path: "native-app/src/app.rs".to_string(),
                    content: native_app_app_rs,
                },
                OutputFile {
                    path: "native-app/src/model.rs".to_string(),
                    content: native_app_model_rs,
                },
                OutputFile {
                    path: "native-app/src/style.rs".to_string(),
                    content: native_app_style_rs,
                },
                OutputFile {
                    path: "native-app/src/render/mod.rs".to_string(),
                    content: native_app_render_mod_rs,
                },
                OutputFile {
                    path: "native-app/src/render/shared.rs".to_string(),
                    content: native_app_render_shared_rs,
                },
                OutputFile {
                    path: "native-app/src/render/state.rs".to_string(),
                    content: native_app_render_state_rs,
                },
                OutputFile {
                    path: "native-app/src/render/flow.rs".to_string(),
                    content: native_app_render_flow_rs,
                },
                OutputFile {
                    path: "native-app/src/render/controls.rs".to_string(),
                    content: native_app_render_controls_rs,
                },
                OutputFile {
                    path: "native-app/src/render/media.rs".to_string(),
                    content: native_app_render_media_rs,
                },
                OutputFile {
                    path: "native-app/README.md".to_string(),
                    content: native_app_readme,
                },
            ],
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeDesktopBundle {
    format_version: String,
    runtime: String,
    entry_component: String,
    components: Vec<NativeComponent>,
    tokens: BTreeMap<String, Value>,
    diagnostics: Vec<Diagnostic>,
}

impl NativeDesktopBundle {
    fn from_ir(ir: &IrProgram) -> Result<Self, String> {
        let node_by_id: BTreeMap<&str, &IrNode> = ir
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let style_by_id: BTreeMap<&str, &IrStyle> = ir
            .styles
            .iter()
            .map(|style| (style.id.as_str(), style))
            .collect();

        let mut components = Vec::with_capacity(ir.components.len());
        for component in &ir.components {
            let mut stack = BTreeSet::new();
            let root = build_native_node(
                &component.root_node_id,
                &node_by_id,
                &style_by_id,
                &mut stack,
            )?;
            components.push(NativeComponent {
                id: component.id.clone(),
                name: component.name.clone(),
                exports: component.exports,
                root_node: root,
            });
        }

        let entry_component = pick_entry_component_name(ir)
            .ok_or_else(|| "desktop backend: no components found in IR".to_string())?;
        let mut diagnostics = ir.diagnostics.clone();
        diagnostics.extend(collect_desktop_parity_diagnostics(ir));

        Ok(Self {
            format_version: "1.0.0".to_string(),
            runtime: "rust-native".to_string(),
            entry_component: entry_component.to_string(),
            components,
            tokens: ir.tokens.clone(),
            diagnostics,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeComponent {
    id: String,
    name: String,
    exports: bool,
    root_node: NativeNode,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeNode {
    id: String,
    widget: String,
    kind: String,
    props: BTreeMap<String, Value>,
    style_refs: Vec<String>,
    resolved_style: BTreeMap<String, Value>,
    children: Vec<NativeNode>,
    source: SourceLoc,
}

fn build_native_node(
    node_id: &str,
    node_by_id: &BTreeMap<&str, &IrNode>,
    style_by_id: &BTreeMap<&str, &IrStyle>,
    stack: &mut BTreeSet<String>,
) -> Result<NativeNode, String> {
    if !stack.insert(node_id.to_string()) {
        return Err(format!(
            "desktop backend: recursive node graph detected at `{}`",
            node_id
        ));
    }

    let node = node_by_id
        .get(node_id)
        .copied()
        .ok_or_else(|| format!("desktop backend: missing node `{node_id}` in IR"))?;

    let mut children = Vec::with_capacity(node.children.len());
    for child_id in &node.children {
        children.push(build_native_node(child_id, node_by_id, style_by_id, stack)?);
    }

    stack.remove(node_id);

    Ok(NativeNode {
        id: node.id.clone(),
        widget: node.name.clone(),
        kind: node.kind.clone(),
        props: node.props.clone(),
        style_refs: node.style_refs.clone(),
        resolved_style: resolve_styles(&node.style_refs, style_by_id),
        children,
        source: node.source.clone(),
    })
}

fn resolve_styles(
    style_refs: &[String],
    style_by_id: &BTreeMap<&str, &IrStyle>,
) -> BTreeMap<String, Value> {
    let mut merged = BTreeMap::new();
    for style_id in style_refs {
        if let Some(style) = style_by_id.get(style_id.as_str()) {
            for (key, value) in &style.decls {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    merged
}

fn pick_entry_component_name<'a>(ir: &'a IrProgram) -> Option<&'a str> {
    if let Some(entry_component) = ir
        .components
        .iter()
        .find(|component| component.name == ir.entry)
    {
        return Some(entry_component.name.as_str());
    }
    if let Some(exported_component) = ir.components.iter().find(|component| component.exports) {
        return Some(exported_component.name.as_str());
    }
    ir.components
        .first()
        .map(|component| component.name.as_str())
}

fn collect_desktop_parity_diagnostics(ir: &IrProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let style_sources = first_style_usage_sources(ir);

    for style in &ir.styles {
        if !style_sources.contains_key(style.id.as_str()) {
            continue;
        }
        let source = style_sources
            .get(style.id.as_str())
            .cloned()
            .unwrap_or_else(default_backend_source);
        for key in style.decls.keys() {
            if is_supported_desktop_style_prop(key) {
                continue;
            }
            diagnostics.push(Diagnostic {
                code: "W7601".to_string(),
                level: "warning".to_string(),
                message: format!(
                    "desktop native renderer does not yet fully support style property `{}` in style `{}`",
                    key, style.id
                ),
                source: source.clone(),
            });
        }
    }

    let mut seen_widgets = BTreeSet::new();
    for node in &ir.nodes {
        if is_supported_desktop_widget(node.name.as_str()) {
            continue;
        }
        if !seen_widgets.insert(node.name.clone()) {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: "W7602".to_string(),
            level: "warning".to_string(),
            message: format!(
                "desktop native renderer uses fallback for unsupported widget `{}`",
                node.name
            ),
            source: node.source.clone(),
        });
    }

    diagnostics
}

fn first_style_usage_sources(ir: &IrProgram) -> BTreeMap<&str, SourceLoc> {
    let mut out: BTreeMap<&str, SourceLoc> = BTreeMap::new();
    for node in &ir.nodes {
        for style_id in &node.style_refs {
            out.entry(style_id.as_str())
                .or_insert_with(|| node.source.clone());
        }
    }
    out
}

fn is_supported_desktop_style_prop(key: &str) -> bool {
    key.starts_with("--")
        || matches!(
            key,
            "color"
                | "background"
                | "background-color"
                | "border"
                | "border-color"
                | "border-width"
                | "border-radius"
                | "box-shadow"
                | "opacity"
                | "gap"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "width"
                | "height"
                | "min-width"
                | "min-height"
                | "max-width"
                | "max-height"
                | "padding"
                | "padding-top"
                | "padding-right"
                | "padding-bottom"
                | "padding-left"
                | "margin"
                | "margin-top"
                | "margin-right"
                | "margin-bottom"
                | "margin-left"
                | "flex-direction"
                | "align-items"
                | "justify-content"
                | "text-align"
                | "overflow"
                | "flex-wrap"
                | "display"
        )
}

fn is_supported_desktop_widget(name: &str) -> bool {
    matches!(
        name,
        "Window"
            | "Page"
            | "Row"
            | "Column"
            | "Stack"
            | "Card"
            | "Scroll"
            | "Text"
            | "Image"
            | "Spacer"
            | "Modal"
            | "Button"
            | "Input"
            | "Checkbox"
            | "Switch"
            | "Fragment"
            | "If"
            | "For"
    )
}

fn default_backend_source() -> SourceLoc {
    SourceLoc {
        file: "<desktop-backend>".to_string(),
        line: 1,
        col: 1,
    }
}

fn render_native_runtime_stub(entry_component: &str) -> String {
    NATIVE_RUNTIME_STUB_RS.replace("{{ENTRY_COMPONENT}}", entry_component)
}

fn render_native_app_cargo_toml(entry_component: &str) -> String {
    let package_name = format!("formo-{}", to_kebab_case(entry_component));
    NATIVE_APP_CARGO_TOML.replace("{{PACKAGE_NAME}}", &package_name)
}

fn render_native_app_main_rs() -> String {
    NATIVE_APP_MAIN_RS.to_string()
}

fn render_native_app_app_rs(entry_component: &str) -> String {
    NATIVE_APP_APP_RS.replace("{{ENTRY_COMPONENT}}", entry_component)
}

fn render_native_app_model_rs() -> String {
    NATIVE_APP_MODEL_RS.to_string()
}

fn render_native_app_style_rs() -> String {
    NATIVE_APP_STYLE_RS.to_string()
}

fn render_native_app_render_mod_rs() -> String {
    NATIVE_APP_RENDER_MOD_RS.to_string()
}

fn render_native_app_render_shared_rs() -> String {
    NATIVE_APP_RENDER_SHARED_RS.to_string()
}

fn render_native_app_render_state_rs() -> String {
    NATIVE_APP_RENDER_STATE_RS.to_string()
}

fn render_native_app_render_flow_rs() -> String {
    NATIVE_APP_RENDER_FLOW_RS.to_string()
}

fn render_native_app_render_controls_rs() -> String {
    NATIVE_APP_RENDER_CONTROLS_RS.to_string()
}

fn render_native_app_render_media_rs() -> String {
    NATIVE_APP_RENDER_MEDIA_RS.to_string()
}

fn render_native_app_readme_md(entry_component: &str) -> String {
    NATIVE_APP_README_MD.replace("{{ENTRY_COMPONENT}}", entry_component)
}

fn to_kebab_case(input: &str) -> String {
    let mut out = String::new();
    let mut prev_is_sep = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !out.is_empty() && !prev_is_sep {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            prev_is_sep = false;
        } else if !out.is_empty() && !prev_is_sep {
            out.push('-');
            prev_is_sep = true;
        }
    }

    if out.is_empty() {
        "formo-desktop-app".to_string()
    } else {
        out.trim_matches('-').to_string()
    }
}

#[cfg(test)]
mod tests {
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
        assert!(app_rs.contains("Desktop parity warnings:"));
        assert!(model_rs.contains("pub diagnostics: Vec<NativeDiagnostic>"));
        assert!(style_rs.contains("pub struct RenderStyle"));
        assert!(style_rs.contains("fn parse_rgb_color("));
        assert!(style_rs.contains("fn parse_font_weight("));
        assert!(style_rs.contains("fn parse_len_px("));
        assert!(style_rs.contains("fn parse_box_shadow("));
        assert!(style_rs.contains("fn parse_border_shorthand("));
        assert!(render_mod_rs.contains("\"Checkbox\" =>"));
        assert!(render_mod_rs.contains("\"Switch\" =>"));
        assert!(render_mod_rs.contains("\"Modal\" =>"));
        assert!(render_mod_rs.contains("\"Image\" =>"));
        assert!(render_mod_rs.contains("\"If\" =>"));
        assert!(render_mod_rs.contains("\"For\" =>"));
        assert!(render_shared_rs.contains("fn layout_from_style("));
        assert!(render_shared_rs.contains("fn apply_gap("));
        assert!(render_state_rs.contains("fn emit_action("));
        assert!(render_state_rs.contains("fn derive_for_item_key("));
        assert!(render_state_rs.contains("fn prop_usize("));
        assert!(render_flow_rs.contains("fn render_if("));
        assert!(render_flow_rs.contains("fn render_for("));
        assert!(render_flow_rs.contains("{}Key"));
        assert!(render_controls_rs.contains("fn render_input("));
        assert!(render_media_rs.contains("fn render_modal("));
        assert!(render_media_rs.contains("egui::Key::Escape"));
        assert!(render_media_rs.contains("egui::Sense::click()"));
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
}
