use crate::model::NativeNode;
use crate::style::{parse_hex_color, Edges, RenderStyle};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value as JsonValue;

use super::shared::{
    apply_gap, apply_text_style, show_text, with_style_container, FrameDefaults,
};
use super::state::{
    emit_action, prop_bool, prop_len, prop_string, read_state_bool,
};
use super::{render_tree_scoped, ActionLog, NativeState, RenderScope};

pub(super) fn render_text(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    scope: &RenderScope,
) {
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        let text = prop_string(node, "value", scope).unwrap_or_else(|| "Text".to_string());
        let rich = apply_text_style(
            RichText::new(text).size(style.font_size.unwrap_or(16.0)),
            style,
            parse_hex_color("#151515"),
            None,
            false,
        );
        show_text(ui, rich, style.text_align);
        if let Some(lh) = style.line_height {
            let extra = ((lh.max(1.0) - 1.0) * style.font_size.unwrap_or(16.0)).max(0.0);
            if extra > 0.0 {
                ui.add_space(extra);
            }
        }
    });
}

pub(super) fn render_image(
    ui: &mut egui::Ui,
    node: &NativeNode,
    mut style: RenderStyle,
    scope: &RenderScope,
) {
    if style.width.is_none() {
        style.width = prop_len(node, "width", scope);
    }
    if style.height.is_none() {
        style.height = prop_len(node, "height", scope);
    }
    let width = style.width.unwrap_or(160.0).max(0.0);
    let height = style.height.unwrap_or(96.0).max(0.0);
    let alt = prop_string(node, "alt", scope).unwrap_or_else(|| "Image".to_string());
    let src = prop_string(node, "src", scope).unwrap_or_default();

    with_style_container(
        ui,
        style,
        FrameDefaults {
            border_color: style.border_color.or_else(|| parse_hex_color("#c9cedf")),
            border_width: style.border_width.or(Some(1.0)),
            border_radius: style.border_radius.or(Some(8.0)),
            padding: if style.has_padding {
                Some(style.padding)
            } else {
                Some(Edges::same(8.0))
            },
            ..FrameDefaults::default()
        },
        |ui| {
            ui.set_min_size(egui::vec2(width, height));
            show_text(
                ui,
                apply_text_style(
                    RichText::new(alt).size(style.font_size.unwrap_or(14.0)),
                    style,
                    parse_hex_color("#48506b"),
                    None,
                    true,
                ),
                style.text_align,
            );
            if !src.is_empty() {
                ui.small(src);
            }
        },
    );
}

pub(super) fn render_spacer(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    scope: &RenderScope,
) {
    let size = prop_len(node, "size", scope)
        .or(style.width)
        .or(style.height)
        .unwrap_or(8.0)
        .max(0.0);
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    });
}

pub(super) fn render_modal(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let open_key = prop_string(node, "open", scope);
    let is_open = if let Some(key) = open_key.as_ref() {
        read_state_bool(state, key).unwrap_or(false)
    } else {
        prop_bool(node, "open", state, scope, false)
    };
    if !is_open {
        return;
    }

    let on_close = prop_string(node, "onClose", scope).unwrap_or_default();
    let title = prop_string(node, "title", scope).unwrap_or_else(|| "Modal".to_string());
    let screen_rect = ui.ctx().screen_rect();
    let modal_id = format!("formo-modal-{}", node.id);
    let focus_state_key = format!("__modal_focused::{}", node.id);

    let backdrop_layer =
        egui::LayerId::new(egui::Order::Foreground, egui::Id::new(format!("{modal_id}-backdrop")));
    ui.ctx().layer_painter(backdrop_layer).rect_filled(
        screen_rect,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 115),
    );

    let backdrop_clicked = egui::Area::new(egui::Id::new(format!("{modal_id}-hit")))
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ui.ctx(), |ui| {
            let (_rect, response) = ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
            response.clicked()
        })
        .inner;

    let escape_pressed = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
    let mut window_open = true;
    let mut close_clicked = false;
    let mut should_mark_focused = false;
    let should_focus = !state
        .get(&focus_state_key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let window_output = egui::Window::new(title)
        .id(egui::Id::new(format!("{modal_id}-window")))
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .open(&mut window_open)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(style.width.unwrap_or(360.0).max(280.0))
        .show(ui.ctx(), |ui| {
            with_style_container(
                ui,
                style,
                FrameDefaults {
                    fill: style.background.or_else(|| parse_hex_color("#ffffff")),
                    border_color: style.border_color.or_else(|| parse_hex_color("#d7dbe8")),
                    border_width: style.border_width.or(Some(1.0)),
                    border_radius: style.border_radius.or(Some(12.0)),
                    padding: if style.has_padding {
                        Some(style.padding)
                    } else {
                        Some(Edges::same(14.0))
                    },
                    shadow: style.shadow.or(Some(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 10.0),
                        blur: 28.0,
                        spread: 0.0,
                        color: Color32::from_rgba_unmultiplied(0, 0, 0, 61),
                    })),
                    ..FrameDefaults::default()
                },
                |ui| {
                    let close = ui.button("Close");
                    if should_focus {
                        close.request_focus();
                        should_mark_focused = true;
                    }
                    if close.clicked() {
                        close_clicked = true;
                    }
                    apply_gap(ui, style.gap, Some(10.0));
                    for child in &node.children {
                        render_tree_scoped(ui, child, state, action_log, scope);
                    }
                },
            );
        });

    if should_mark_focused {
        state.insert(focus_state_key.clone(), JsonValue::Bool(true));
    }

    let pointer_pos = ui.ctx().pointer_interact_pos();
    let mut window_rect = None;
    if let Some(out) = window_output {
        window_rect = Some(out.response.rect);
    }
    let outside_click = if backdrop_clicked {
        match (pointer_pos, window_rect) {
            (Some(pos), Some(rect)) => !rect.contains(pos),
            _ => true,
        }
    } else {
        false
    };

    if close_clicked || escape_pressed || outside_click || !window_open {
        state.remove(&focus_state_key);
        if let Some(key) = open_key.as_ref() {
            state.insert(key.clone(), JsonValue::Bool(false));
        }
        emit_action(action_log, &on_close, node, JsonValue::Null, state, scope);
    }
}
