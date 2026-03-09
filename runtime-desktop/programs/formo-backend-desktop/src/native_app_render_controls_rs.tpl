use crate::model::NativeNode;
use crate::style::{parse_hex_color, Edges, RenderStyle};
use eframe::egui::{self, RichText};
use serde_json::Value as JsonValue;

use super::shared::{apply_gap, apply_text_style, resolve_length, with_style_container, FrameDefaults};
use super::state::{
    emit_action, prop_bool, prop_string, read_state_bool, read_state_string,
};
use super::{ActionLog, NativeState, RenderScope};

pub(super) fn render_button(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    let disabled = prop_bool(node, "disabled", state, scope, false);
    let label = prop_string(node, "label", scope)
        .or_else(|| prop_string(node, "text", scope))
        .unwrap_or_else(|| "Button".to_string());
    let text = apply_text_style(
        RichText::new(label).size(style.font_size.unwrap_or(14.0)),
        style,
        parse_hex_color("#151515"),
        Some(600.0),
        false,
    );

    let mut button = egui::Button::new(text);
    if let Some(fill) = style.background.or_else(|| parse_hex_color("#f4f7ff")) {
        button = button.fill(crate::style::with_opacity(fill, style.opacity));
    }
    if let Some(color) = style.border_color.or_else(|| parse_hex_color("#b8bfd8")) {
        button = button.stroke(egui::Stroke::new(
            style.border_width.unwrap_or(1.0).max(0.0),
            crate::style::with_opacity(color, style.opacity),
        ));
    }
    button = button.rounding(egui::Rounding::same(
        style.border_radius.unwrap_or(8.0).max(0.0),
    ));

    let response = ui
        .add_enabled_ui(!disabled, |ui| {
            let available = ui.available_size_before_wrap();
            let width = resolve_length(
                style.width.or(style.min_width),
                style.width_pct.or(style.min_width_pct),
                available.x,
            );
            let height = style.height;
            match (width, height) {
                (Some(w), Some(h)) => ui.add_sized([w.max(0.0), h.max(0.0)], button),
                (Some(w), None) => ui.add_sized([w.max(0.0), ui.spacing().interact_size.y], button),
                (None, Some(h)) => ui.add_sized([ui.spacing().interact_size.x, h.max(0.0)], button),
                (None, None) => ui.horizontal(|ui| ui.add(button)).inner,
            }
        })
        .inner;

    if response.clicked() {
        let action_name = prop_string(node, "onPress", scope)
            .or_else(|| prop_string(node, "onClick", scope))
            .or_else(|| prop_string(node, "action", scope))
            .unwrap_or_default();
        emit_action(action_log, &action_name, node, JsonValue::Null, state, scope);
    }
}

pub(super) fn render_input(
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
            fill: style.background.or_else(|| parse_hex_color("#ffffff")),
            border_color: style.border_color.or_else(|| parse_hex_color("#b8bfd8")),
            border_width: style.border_width.or(Some(1.0)),
            border_radius: style.border_radius.or(Some(8.0)),
            padding: if style.has_padding {
                Some(style.padding)
            } else {
                Some(Edges {
                    top: 8.0,
                    right: 10.0,
                    bottom: 8.0,
                    left: 10.0,
                })
            },
            ..FrameDefaults::default()
        },
        |ui| {
            let explicit_state_key = prop_string(node, "value", scope)
                .or_else(|| prop_string(node, "bind", scope))
                .or_else(|| prop_string(node, "name", scope));
            let storage_key = explicit_state_key
                .clone()
                .unwrap_or_else(|| format!("__local_input::{}", node.id));
            let disabled = prop_bool(node, "disabled", state, scope, false);
            let on_change = prop_string(node, "onChange", scope).unwrap_or_default();
            let input_type = prop_string(node, "inputType", scope)
                .map(|v| v.to_ascii_lowercase())
                .unwrap_or_else(|| "text".to_string());

            let mut text = read_state_string(state, &storage_key).unwrap_or_default();
            if text.is_empty() {
                if let Some(v) = prop_string(node, "defaultValue", scope) {
                    text = v;
                }
            }

            let placeholder = prop_string(node, "placeholder", scope).unwrap_or_default();
            let available = ui.available_size_before_wrap();
            let width = resolve_length(
                style.width.or(style.min_width),
                style.width_pct.or(style.min_width_pct),
                available.x,
            )
            .unwrap_or(220.0)
            .max(0.0);
            let height = style.height;
            let response = ui
                .add_enabled_ui(!disabled, |ui| {
                    let mut edit = egui::TextEdit::singleline(&mut text).frame(false);
                    if !placeholder.is_empty() {
                        edit = edit.hint_text(placeholder.clone());
                    }
                    if input_type == "password" {
                        edit = edit.password(true);
                    }

                    if let Some(height) = height {
                        ui.add_sized([width, height.max(0.0)], edit)
                    } else {
                        ui.add_sized([width, ui.spacing().interact_size.y], edit)
                    }
                })
                .inner;

            if response.changed() {
                state.insert(storage_key, JsonValue::String(text.clone()));
                if explicit_state_key.is_some() {
                    emit_action(
                        action_log,
                        &on_change,
                        node,
                        JsonValue::String(text),
                        state,
                        scope,
                    );
                }
            }
        },
    );
}

pub(super) fn render_checkbox(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        apply_gap(ui, style.gap, Some(8.0));
        let explicit_state_key = prop_string(node, "checked", scope);
        let storage_key = explicit_state_key
            .clone()
            .unwrap_or_else(|| format!("__local_check::{}", node.id));
        let disabled = prop_bool(node, "disabled", state, scope, false);
        let on_change = prop_string(node, "onChange", scope).unwrap_or_default();
        let label = prop_string(node, "label", scope).unwrap_or_default();

        let mut checked = read_state_bool(state, &storage_key)
            .unwrap_or_else(|| prop_bool(node, "checked", state, scope, false));
        let response = ui
            .add_enabled_ui(!disabled, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = style.gap.unwrap_or(8.0).max(0.0);
                    let resp =
                        ui.add_sized([16.0, 16.0], egui::Checkbox::without_text(&mut checked));
                    if !label.is_empty() {
                        let rich = apply_text_style(
                            RichText::new(label).size(style.font_size.unwrap_or(14.0)),
                            style,
                            parse_hex_color("#151515"),
                            None,
                            false,
                        );
                        ui.label(rich);
                    }
                    resp
                })
                .inner
            })
            .inner;
        if response.changed() {
            state.insert(storage_key, JsonValue::Bool(checked));
            if explicit_state_key.is_some() {
                emit_action(
                    action_log,
                    &on_change,
                    node,
                    JsonValue::Bool(checked),
                    state,
                    scope,
                );
            }
        }
    });
}

pub(super) fn render_switch(
    ui: &mut egui::Ui,
    node: &NativeNode,
    style: RenderStyle,
    state: &mut NativeState,
    action_log: &mut ActionLog,
    scope: &RenderScope,
) {
    with_style_container(ui, style, FrameDefaults::default(), |ui| {
        apply_gap(ui, style.gap, Some(8.0));
        let explicit_state_key = prop_string(node, "checked", scope);
        let storage_key = explicit_state_key
            .clone()
            .unwrap_or_else(|| format!("__local_switch::{}", node.id));
        let disabled = prop_bool(node, "disabled", state, scope, false);
        let on_change = prop_string(node, "onChange", scope).unwrap_or_default();
        let label = prop_string(node, "label", scope).unwrap_or_else(|| "Switch".to_string());

        let mut checked = read_state_bool(state, &storage_key)
            .unwrap_or_else(|| prop_bool(node, "checked", state, scope, false));

        let response = ui
            .add_enabled_ui(!disabled, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = style.gap.unwrap_or(8.0).max(0.0);
                    let resp =
                        ui.add_sized([16.0, 16.0], egui::Checkbox::without_text(&mut checked));
                    let rich = apply_text_style(
                        RichText::new(label).size(style.font_size.unwrap_or(14.0)),
                        style,
                        parse_hex_color("#151515"),
                        None,
                        false,
                    );
                    ui.label(rich);
                    resp
                })
                .inner
            })
            .inner;

        if response.changed() {
            state.insert(storage_key, JsonValue::Bool(checked));
            if explicit_state_key.is_some() {
                emit_action(
                    action_log,
                    &on_change,
                    node,
                    JsonValue::Bool(checked),
                    state,
                    scope,
                );
            }
        }
    });
}
