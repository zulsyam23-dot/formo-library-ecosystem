use crate::model::NativeNode;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

use super::shared::{node_class, node_style};
use super::state::{dispatch_action, prop_bool, prop_string, read_state_bool, write_state};
use super::{render_node, ActionLog, NativeState, RenderScope};

pub(super) fn render_text_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
) -> Element {
    let class = node_class(node, "formo-text");
    let style = node_style(node);
    let text = prop_string(node, "value", state_store, scope)
        .or_else(|| prop_string(node, "text", state_store, scope))
        .unwrap_or_else(|| "Text".to_string());

    rsx! {
        p {
            class: "{class}",
            style: "{style}",
            "{text}"
        }
    }
}

pub(super) fn render_image_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
) -> Element {
    let class = node_class(node, "formo-image");
    let style = node_style(node);
    let src = prop_string(node, "src", state_store, scope).unwrap_or_default();
    let alt = prop_string(node, "alt", state_store, scope)
        .or_else(|| prop_string(node, "label", state_store, scope))
        .unwrap_or_else(|| "Image".to_string());
    rsx! {
        img {
            class: "{class}",
            style: "{style}",
            src: "{src}",
            alt: "{alt}"
        }
    }
}

pub(super) fn render_spacer_html(node: &NativeNode) -> Element {
    let class = node_class(node, "formo-spacer");
    let style = node_style(node);
    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            aria_hidden: "true"
        }
    }
}

pub(super) fn render_modal_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let open_key = prop_string(node, "open", state_store, scope);
    let is_open = if let Some(key) = open_key.as_ref() {
        read_state_bool(state_store, key).unwrap_or(false)
    } else {
        prop_bool(node, "open", state_store, scope, false)
    };

    if !is_open {
        return rsx! { Fragment {} };
    }

    let title = prop_string(node, "title", state_store, scope).unwrap_or_else(|| "Modal".to_string());
    let on_close = prop_string(node, "onClose", state_store, scope).unwrap_or_default();

    let node_snapshot = node.clone();
    let scope_snapshot = scope.clone();

    rsx! {
        div {
            class: "formo-node formo-modal",
            div {
                class: "formo-modal-body",
                h3 { "{title}" }
                button {
                    class: "formo-button",
                    onclick: move |_| {
                        if let Some(key) = open_key.as_ref() {
                            write_state(state_store, key, JsonValue::Bool(false));
                        }
                        dispatch_action(
                            action_log,
                            &on_close,
                            &node_snapshot,
                            JsonValue::Null,
                            state_store,
                            &scope_snapshot,
                        );
                    },
                    "Close"
                }
                for child in &node.children {
                    {render_node(child, scope, state_store, action_log)}
                }
            }
        }
    }
}
