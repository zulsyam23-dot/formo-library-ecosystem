mod controls;
mod flow;
mod media;
mod shared;
mod state;

use crate::model::NativeNode;
use crate::style::RenderStyle;
use eframe::egui;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

pub(crate) type NativeState = BTreeMap<String, JsonValue>;
pub(crate) type ActionLog = Vec<String>;
pub(crate) type RenderScope = BTreeMap<String, JsonValue>;

pub fn render_tree(
    ui: &mut egui::Ui,
    node: &NativeNode,
    state: &mut NativeState,
    action_log: &mut ActionLog,
) {
    let scope = RenderScope::new();
    render_tree_scoped(ui, node, state, action_log, &scope);
}

pub(super) fn render_tree_scoped(
    ui: &mut egui::Ui,
    node: &NativeNode,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let style = RenderStyle::from_node(node);
    if style.display_none {
        return;
    }

    match node.widget.as_str() {
        "Page" | "Column" | "Row" => flow::render_flex(ui, node, style, state, action_log, scope),
        "Stack" => {
            if style.display_flex || style.flow.is_some() {
                flow::render_flex(ui, node, style, state, action_log, scope)
            } else {
                flow::render_block(ui, node, style, state, action_log, scope)
            }
        }
        "Scroll" => flow::render_scroll(ui, node, style, state, action_log, scope),
        "Card" | "Window" => {
            flow::render_frame_container(ui, node, style, state, action_log, scope)
        }
        "Fragment" => flow::render_fragment(ui, node, state, action_log, scope),
        "If" => flow::render_if(ui, node, state, action_log, scope),
        "For" => flow::render_for(ui, node, state, action_log, scope),
        "Text" => media::render_text(ui, node, style, scope),
        "Image" => media::render_image(ui, node, style, scope),
        "Spacer" => media::render_spacer(ui, node, style, scope),
        "Modal" => media::render_modal(ui, node, style, state, action_log, scope),
        "Button" => controls::render_button(ui, node, style, state, action_log, scope),
        "Input" => controls::render_input(ui, node, style, state, action_log, scope),
        "Checkbox" => controls::render_checkbox(ui, node, style, state, action_log, scope),
        "Switch" => controls::render_switch(ui, node, style, state, action_log, scope),
        _ => flow::render_fallback(ui, node, style, state, action_log, scope),
    }
}
