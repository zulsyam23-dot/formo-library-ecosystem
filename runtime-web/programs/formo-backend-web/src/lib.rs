use formo_ir::{Backend, BackendOutput, IrProgram, OutputFile};

mod css;
mod html;
mod runtime;

pub struct WebBackend;

impl Backend for WebBackend {
    fn emit(&self, ir: &IrProgram) -> Result<BackendOutput, String> {
        let state_json = serde_json::to_string_pretty(ir).map_err(|e| e.to_string())?;
        let html = html::render_index_html(&ir.entry, &state_json, runtime::dev_bootstrap_script());
        let css = css::render_css(ir);
        let js = runtime::app_js();

        Ok(BackendOutput {
            files: vec![
                OutputFile {
                    path: "index.html".to_string(),
                    content: html,
                },
                OutputFile {
                    path: "app.css".to_string(),
                    content: css,
                },
                OutputFile {
                    path: "app.js".to_string(),
                    content: js,
                },
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use formo_ir::{IrComponent, IrNode, SourceLoc, Target, Value};
    use std::collections::BTreeMap;

    fn source() -> SourceLoc {
        SourceLoc {
            file: "main.fm".to_string(),
            line: 1,
            col: 1,
        }
    }

    fn string_value(text: &str) -> Value {
        Value {
            t: "string".to_string(),
            v: serde_json::Value::String(text.to_string()),
        }
    }

    fn sample_ir_with_for_loop() -> IrProgram {
        let mut page_props = BTreeMap::new();
        let mut for_props = BTreeMap::new();
        let mut text_props = BTreeMap::new();

        page_props.insert("id".to_string(), string_value("root"));
        for_props.insert("each".to_string(), string_value("items"));
        for_props.insert("as".to_string(), string_value("item"));
        text_props.insert("value".to_string(), string_value("item"));

        let page = IrNode {
            id: "n_page".to_string(),
            kind: "node".to_string(),
            name: "Page".to_string(),
            props: page_props,
            style_refs: vec![],
            children: vec!["n_for".to_string()],
            source: source(),
        };

        let for_node = IrNode {
            id: "n_for".to_string(),
            kind: "node".to_string(),
            name: "For".to_string(),
            props: for_props,
            style_refs: vec![],
            children: vec!["n_text".to_string()],
            source: source(),
        };

        let text = IrNode {
            id: "n_text".to_string(),
            kind: "node".to_string(),
            name: "Text".to_string(),
            props: text_props,
            style_refs: vec![],
            children: vec![],
            source: source(),
        };

        let component = IrComponent {
            id: "c_app".to_string(),
            name: "App".to_string(),
            root_node_id: "n_page".to_string(),
            exports: true,
            source: source(),
        };

        IrProgram {
            ir_version: "0.1.0".to_string(),
            entry: "App".to_string(),
            target: Target::Web,
            tokens: BTreeMap::new(),
            components: vec![component],
            nodes: vec![page, for_node, text],
            styles: vec![],
            diagnostics: vec![],
        }
    }

    fn emitted_js(ir: &IrProgram) -> String {
        let backend = WebBackend;
        let output = backend.emit(ir).expect("web emit should succeed");
        output
            .files
            .iter()
            .find(|file| file.path == "app.js")
            .map(|file| file.content.clone())
            .expect("app.js output should exist")
    }

    #[test]
    fn runtime_emits_keyed_for_update_helpers() {
        let js = emitted_js(&sample_ir_with_for_loop());
        assert!(js.contains("function canPatchForStateKey("));
        assert!(js.contains("function updateForBindingsForKey("));
        assert!(js.contains("function deriveForItemKey("));
    }

    #[test]
    fn runtime_uses_for_containers_for_keyed_patch() {
        let js = emitted_js(&sample_ir_with_for_loop());
        assert!(js.contains("element = el(\"div\", \"fm-for\")"));
        assert!(js.contains("element.style.display = \"contents\""));
        assert!(js.contains("wrapper.style.display = \"contents\""));
    }

    #[test]
    fn runtime_emits_action_error_boundary_helpers() {
        let js = emitted_js(&sample_ir_with_for_loop());
        assert!(js.contains("function reportRuntimeError("));
        assert!(js.contains("function runWithEventBoundary("));
        assert!(js.contains("window.formoRuntimeErrors"));
    }

    #[test]
    fn runtime_emits_modal_accessibility_helpers() {
        let js = emitted_js(&sample_ir_with_for_loop());
        assert!(js.contains("element.setAttribute(\"role\", \"dialog\")"));
        assert!(js.contains("element.setAttribute(\"aria-modal\", \"true\")"));
        assert!(js.contains("function trapTabInContainer("));
        assert!(js.contains("focusFirstInContainer(panel, closeBtn)"));
    }
}
