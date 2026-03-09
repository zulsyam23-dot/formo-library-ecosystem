use formo_ir::{
    effective_style_decls, Backend, BackendOutput, Diagnostic, IrNode, IrProgram, IrStyle,
    OutputFile, SourceLoc, Value,
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
        let readable_bundle_components = to_pretty_json(&native_bundle.components)?;
        let readable_bundle_tokens = to_pretty_json(&native_bundle.tokens)?;
        let readable_bundle_diagnostics = to_pretty_json(&native_bundle.diagnostics)?;
        let readable_ir_components = to_pretty_json(&ir.components)?;
        let readable_ir_nodes = to_pretty_json(&ir.nodes)?;
        let readable_ir_styles = to_pretty_json(&ir.styles)?;
        let readable_ir_tokens = to_pretty_json(&ir.tokens)?;
        let readable_ir_diagnostics = to_pretty_json(&ir.diagnostics)?;
        let readable_readme = render_readable_artifacts_readme();

        let mut files = vec![
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
            OutputFile {
                path: "readable/README.md".to_string(),
                content: readable_readme,
            },
            OutputFile {
                path: "readable/native/components.json".to_string(),
                content: readable_bundle_components,
            },
            OutputFile {
                path: "readable/native/tokens.json".to_string(),
                content: readable_bundle_tokens,
            },
            OutputFile {
                path: "readable/native/diagnostics.json".to_string(),
                content: readable_bundle_diagnostics,
            },
            OutputFile {
                path: "readable/ir/components.json".to_string(),
                content: readable_ir_components,
            },
            OutputFile {
                path: "readable/ir/nodes.json".to_string(),
                content: readable_ir_nodes,
            },
            OutputFile {
                path: "readable/ir/styles.json".to_string(),
                content: readable_ir_styles,
            },
            OutputFile {
                path: "readable/ir/tokens.json".to_string(),
                content: readable_ir_tokens,
            },
            OutputFile {
                path: "readable/ir/diagnostics.json".to_string(),
                content: readable_ir_diagnostics,
            },
        ];

        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(BackendOutput { files })
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
            let decls = effective_style_decls(style);
            for (key, value) in decls {
                merged.insert(key, value);
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
        let decls = effective_style_decls(style);
        for key in decls.keys() {
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

fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

fn render_readable_artifacts_readme() -> String {
    [
        "# Desktop Artifacts (Readable)",
        "",
        "File utama runtime desktop tetap:",
        "- `app.native.json`",
        "- `app.native.rs`",
        "- `app.ir.json`",
        "",
        "Folder `readable/` menyediakan data terpecah agar mudah dibaca manusia dan AI:",
        "- `readable/native/components.json`",
        "- `readable/native/tokens.json`",
        "- `readable/native/diagnostics.json`",
        "- `readable/ir/components.json`",
        "- `readable/ir/nodes.json`",
        "- `readable/ir/styles.json`",
        "- `readable/ir/tokens.json`",
        "- `readable/ir/diagnostics.json`",
        "",
    ]
    .join("\n")
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
mod tests;
