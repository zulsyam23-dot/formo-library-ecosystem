use crate::model::{NativeBundle, NativeNode};
use crate::render::{render_node, RenderScope};
use crate::style::runtime_css;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const FORMO_NATIVE_JSON: &str = include_str!("../../app.native.json");
const FORMO_COMPILED_ENTRY: &str = "{{ENTRY_COMPONENT}}";

pub fn run() {
    dioxus_desktop::launch(AppRoot);
}

fn native_bundle() -> &'static NativeBundle {
    static BUNDLE: OnceLock<NativeBundle> = OnceLock::new();
    BUNDLE.get_or_init(|| {
        serde_json::from_str(FORMO_NATIVE_JSON).expect("app.native.json must be valid")
    })
}

fn entry_root(bundle: &NativeBundle) -> Option<&NativeNode> {
    if let Some(component) = bundle
        .components
        .iter()
        .find(|component| component.name == bundle.entry_component)
    {
        return Some(&component.root_node);
    }
    bundle.components.first().map(|component| &component.root_node)
}

fn parity_messages(bundle: &NativeBundle) -> Vec<String> {
    bundle
        .diagnostics
        .iter()
        .filter(|diag| diag.level.eq_ignore_ascii_case("warning"))
        .map(|diag| {
            format!(
                "[{}] {} ({}:{}:{})",
                diag.code, diag.message, diag.source.file, diag.source.line, diag.source.col
            )
        })
        .collect()
}

#[component]
fn AppRoot() -> Element {
    let bundle = native_bundle().clone();
    let state_store = use_signal(|| BTreeMap::<String, JsonValue>::new());
    let action_log = use_signal(Vec::<String>::new);

    let scope = RenderScope::new();
    let root = entry_root(&bundle).cloned();

    let parity_items = parity_messages(&bundle);
    let parity_label = format!("Desktop parity warnings: {}", parity_items.len());

    let action_preview = {
        let items = action_log.read();
        if items.is_empty() {
            "<none>".to_string()
        } else {
            items
                .iter()
                .rev()
                .take(20)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        }
    };

    let content = if let Some(root_node) = root {
        rsx! {
            div {
                class: "formo-native-root",
                {render_node(&root_node, &scope, state_store, action_log)}
            }
        }
    } else {
        rsx! {
            div {
                class: "formo-empty",
                "No entry component found in app.native.json"
            }
        }
    };

    rsx! {
        style { "{runtime_css()}" }
        div {
            class: "formo-native-shell",
            div {
                class: "formo-native-header",
                h1 { "Formo Desktop - {bundle.entry_component}" }
                p { "Compiled entry: {FORMO_COMPILED_ENTRY}" }
            }
            div {
                class: "formo-parity-banner",
                "{parity_label}"
            }
            {content}
            details {
                class: "formo-parity-details",
                summary { "Parity diagnostics" }
                ul {
                    if parity_items.is_empty() {
                        li { "<none>" }
                    } else {
                        for item in &parity_items {
                            li { "{item}" }
                        }
                    }
                }
            }
            details {
                class: "formo-parity-details",
                summary { "Action log" }
                pre { "{action_preview}" }
            }
        }
    }
}
