use crate::model::NativeNode;
use crate::style::{parse_hex_color, Flow, Overflow, RenderStyle};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value as JsonValue;

use super::shared::{
    apply_gap, apply_text_style, layout_from_style, show_text, with_style_container, FrameDefaults,
};
use super::state::{derive_for_item_key, prop_bool, prop_literal_string, prop_string, prop_usize};
use super::{render_tree_scoped, ActionLog, NativeState, RenderScope};

pub(super) fn render_flex(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let flow = style
        .flow
        .unwrap_or(if node.widget == "Row" { Flow::Row } else { Flow::Column });
    let page_defaults = if node.widget == "Page" {
        FrameDefaults {
            padding: Some(crate::style::Edges::same(12.0)),
            ..FrameDefaults::default()
        }
    } else {
        FrameDefaults::default()
    };

    with_style_container(ui, style, page_defaults, |ui| {
        apply_gap(ui, style.gap, Some(8.0));
        let layout = layout_from_style(flow, style);
        let should_scroll = node.widget == "Scroll"
            || (style.overflow == Overflow::Scroll && style.overflow != Overflow::Hidden);
        if should_scroll {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.with_layout(layout, |ui| {
                    for child in &node.children {
                        render_tree_scoped(ui, child, state, action_log, scope);
                    }
                });
            });
        } else {
            ui.with_layout(layout, |ui| {
                for child in &node.children {
                    render_tree_scoped(ui, child, state, action_log, scope);
                }
            });
        }
    });
}

pub(super) fn render_block(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        for child in &node.children {
            render_tree_scoped(ui, child, state, action_log, scope);
        }
    });
}

pub(super) fn render_scroll(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        apply_gap(ui, style.gap, None);
        egui::ScrollArea::both().show(ui, |ui| {
            if style.display_flex || style.flow.is_some() {
                let flow = style.flow.unwrap_or(Flow::Column);
                let layout = layout_from_style(flow, style);
                ui.with_layout(layout, |ui| {
                    for child in &node.children {
                        render_tree_scoped(ui, child, state, action_log, scope);
                    }
                });
            } else {
                for child in &node.children {
                    render_tree_scoped(ui, child, state, action_log, scope);
                }
            }
        });
    });
}

pub(super) fn render_frame_container(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let defaults = if node.widget == "Window" {
        FrameDefaults {
            fill: parse_hex_color("#ffffff"),
            border_color: parse_hex_color("#d4d8e5"),
            border_width: Some(1.0),
            border_radius: Some(14.0),
            shadow: Some(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 8.0),
                blur: 24.0,
                spread: 0.0,
                color: Color32::from_rgba_unmultiplied(0, 0, 0, 18),
            }),
            ..FrameDefaults::default()
        }
    } else {
        FrameDefaults {
            fill: parse_hex_color("#ffffff"),
            border_color: parse_hex_color("#d7dbe8"),
            border_width: Some(1.0),
            border_radius: Some(10.0),
            padding: Some(crate::style::Edges::same(10.0)),
            ..FrameDefaults::default()
        }
    };

    with_style_container(ui, style, defaults, |ui| {
        apply_gap(ui, style.gap, None);
        if let Some(title) = prop_string(node, "title", scope) {
            if node.widget == "Window" {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: 16.0,
                        right: 16.0,
                        top: 12.0,
                        bottom: 12.0,
                    })
                    .show(ui, |ui| {
                        show_text(
                            ui,
                            apply_text_style(
                                RichText::new(title).size(16.0).strong(),
                                style,
                                parse_hex_color("#151515"),
                                Some(700.0),
                                false,
                            ),
                            style.text_align,
                        );
                    });
                ui.separator();
            } else {
                show_text(
                    ui,
                    apply_text_style(
                        RichText::new(title).size(16.0).strong(),
                        style,
                        parse_hex_color("#151515"),
                        Some(700.0),
                        false,
                    ),
                    style.text_align,
                );
            }
        }
        for child in &node.children {
            render_tree_scoped(ui, child, state, action_log, scope);
        }
    });
}

pub(super) fn render_fragment(
    ui: &mut egui::Ui,
    node: &NativeNode,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    for child in &node.children {
        render_tree_scoped(ui, child, state, action_log, scope);
    }
}

pub(super) fn render_if(
    ui: &mut egui::Ui,
    node: &NativeNode,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let has_when = node.props.contains_key("when");
    let has_condition = node.props.contains_key("condition");
    let should_show = if has_when {
        prop_bool(node, "when", state, scope, false)
    } else if has_condition {
        prop_bool(node, "condition", state, scope, false)
    } else {
        false
    };

    if should_show {
        render_fragment(ui, node, state, action_log, scope);
    }
}

pub(super) fn render_for(
    ui: &mut egui::Ui,
    node: &NativeNode,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let each_items = super::state::prop_list(node, "each", state, scope);
    let alias = prop_literal_string(node, "as").unwrap_or_else(|| "item".to_string());
    if each_items.is_empty() {
        if let Some(count) = prop_usize(node, "count", scope) {
            for index in 0..count {
                let mut next_scope = scope.clone();
                next_scope.insert(alias.clone(), JsonValue::from(index as i64));
                next_scope.insert(format!("{}Index", alias), JsonValue::from(index as i64));
                next_scope.insert(
                    format!("{}Key", alias),
                    JsonValue::String(format!("idx:{index}")),
                );
                for child in &node.children {
                    render_tree_scoped(ui, child, state, action_log, &next_scope);
                }
            }
        }
        return;
    }

    for (index, item) in each_items.iter().enumerate() {
        let mut next_scope = scope.clone();
        next_scope.insert(alias.clone(), item.clone());
        next_scope.insert(format!("{}Index", alias), JsonValue::from(index as i64));
        next_scope.insert(
            format!("{}Key", alias),
            JsonValue::String(derive_for_item_key(item, index)),
        );
        for child in &node.children {
            render_tree_scoped(ui, child, state, action_log, &next_scope);
        }
    }
}

pub(super) fn render_fallback(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    with_style_container(
        ui,
        style,
        FrameDefaults {
            border_color: parse_hex_color("#c9cedf"),
            border_width: Some(1.0),
            border_radius: Some(8.0),
            padding: Some(crate::style::Edges::same(8.0)),
            ..FrameDefaults::default()
        },
        |ui| {
            let label = apply_text_style(
                RichText::new(format!("{} [{}]", node.widget, node.id))
                    .size(style.font_size.unwrap_or(14.0)),
                style,
                parse_hex_color("#48506b"),
                None,
                true,
            );
            show_text(ui, label, style.text_align);
            apply_gap(ui, style.gap, Some(8.0));
            for child in &node.children {
                render_tree_scoped(ui, child, state, action_log, scope);
            }
        },
    );
}
