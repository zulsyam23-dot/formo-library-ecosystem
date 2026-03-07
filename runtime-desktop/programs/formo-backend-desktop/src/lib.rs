use formo_backend_web::WebBackend;
use formo_ir::{Backend, BackendOutput, IrProgram, OutputFile};

const DESKTOP_BRIDGE_JS: &str = include_str!("desktop_bridge.js");

pub struct DesktopBackend;

impl Backend for DesktopBackend {
    fn emit(&self, ir: &IrProgram) -> Result<BackendOutput, String> {
        let mut web_output = WebBackend.emit(ir)?;
        patch_web_files_for_desktop(&mut web_output);

        let payload = serde_json::to_string_pretty(ir).map_err(|e| e.to_string())?;
        web_output.files.push(OutputFile {
            path: "app.ir.json".to_string(),
            content: payload,
        });
        web_output.files.push(OutputFile {
            path: "desktop-bridge.js".to_string(),
            content: DESKTOP_BRIDGE_JS.to_string(),
        });

        Ok(web_output)
    }
}

fn patch_web_files_for_desktop(output: &mut BackendOutput) {
    for file in &mut output.files {
        if file.path == "index.html" {
            file.content = patch_html_for_desktop(&file.content);
        }
    }
}

fn patch_html_for_desktop(html: &str) -> String {
    let with_marker = if html.contains("name=\"formo-runtime\"") {
        html.to_string()
    } else {
        html.replacen(
            "</head>",
            "  <meta name=\"formo-runtime\" content=\"desktop\">\n</head>",
            1,
        )
    };

    if with_marker.contains("desktop-bridge.js") {
        return with_marker;
    }

    with_marker.replacen(
        "<script src=\"app.js\"></script>",
        "<script src=\"desktop-bridge.js\"></script>\n  <script src=\"app.js\"></script>",
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use formo_ir::{IrComponent, IrNode, SourceLoc, StyleSelector, Target, Value};
    use std::collections::BTreeMap;

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
                style_refs: vec![],
                children: vec![],
                source: source(),
            }],
            styles: vec![formo_ir::IrStyle {
                id: "Text".to_string(),
                selector: StyleSelector {
                    component: "Text".to_string(),
                    part: "root".to_string(),
                },
                decls: BTreeMap::new(),
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
    fn desktop_backend_emits_webview_bundle_plus_ir() {
        let output = DesktopBackend
            .emit(&sample_ir())
            .expect("desktop emit should succeed");
        assert!(output.files.iter().any(|f| f.path == "index.html"));
        assert!(output.files.iter().any(|f| f.path == "app.js"));
        assert!(output.files.iter().any(|f| f.path == "app.css"));
        assert!(output.files.iter().any(|f| f.path == "desktop-bridge.js"));
        assert!(output.files.iter().any(|f| f.path == "app.ir.json"));
    }

    #[test]
    fn desktop_html_contains_runtime_marker_and_bridge_script() {
        let output = DesktopBackend
            .emit(&sample_ir())
            .expect("desktop emit should succeed");
        let html = file(&output, "index.html");
        assert!(html.contains("name=\"formo-runtime\" content=\"desktop\""));
        assert!(html.contains("<script src=\"desktop-bridge.js\"></script>"));
        assert!(html.contains("<script src=\"app.js\"></script>"));
    }

    #[test]
    fn desktop_bridge_exposes_host_adapter() {
        let output = DesktopBackend
            .emit(&sample_ir())
            .expect("desktop emit should succeed");
        let bridge = file(&output, "desktop-bridge.js");
        assert!(bridge.contains("window.formoDesktopHost"));
        assert!(bridge.contains("window.formoDesktop"));
        assert!(bridge.contains("new Proxy"));
    }
}
