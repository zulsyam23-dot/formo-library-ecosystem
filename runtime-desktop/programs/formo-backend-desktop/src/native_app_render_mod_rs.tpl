mod controls;
mod flow;
mod media;
mod shared;
mod state;

use crate::model::NativeNode;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

pub(crate) type NativeState = BTreeMap<String, JsonValue>;
pub(crate) type ActionLog = Vec<String>;
pub(crate) type RenderScope = BTreeMap<String, JsonValue>;

pub fn render_node(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    match node.widget.as_str() {
        "Page" | "Column" | "Row" | "Stack" | "Card" | "Window" | "Scroll" => {
            flow::render_container_html(node, scope, state_store, action_log)
        }
        "Fragment" => flow::render_fragment_html(node, scope, state_store, action_log),
        "If" => flow::render_if_html(node, scope, state_store, action_log),
        "For" => flow::render_for_html(node, scope, state_store, action_log),
        "Text" => media::render_text_html(node, scope, state_store),
        "Image" => media::render_image_html(node, scope, state_store),
        "Spacer" => media::render_spacer_html(node),
        "Modal" => media::render_modal_html(node, scope, state_store, action_log),
        "Button" => controls::render_button_html(node, scope, state_store, action_log),
        "Input" => controls::render_input_html(node, scope, state_store, action_log),
        "Checkbox" => controls::render_checkbox_html(node, scope, state_store, action_log),
        "Switch" => controls::render_switch_html(node, scope, state_store, action_log),
        _ => flow::render_fallback_html(node, scope, state_store, action_log),
    }
}
