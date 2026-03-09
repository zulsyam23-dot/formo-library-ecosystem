use crate::model::NativeNode;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

use super::shared::{node_class, node_style};
use super::state::{
    dispatch_action, prop_bool, prop_string, read_state_bool, read_state_string, write_state,
};
use super::{ActionLog, NativeState, RenderScope};

pub(super) fn render_button_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, "formo-button");
    let style = node_style(node);
    let disabled = prop_bool(node, "disabled", state_store, scope, false);
    let label = prop_string(node, "label", state_store, scope)
        .or_else(|| prop_string(node, "text", state_store, scope))
        .unwrap_or_else(|| "Button".to_string());

    let action_name = prop_string(node, "onPress", state_store, scope)
        .or_else(|| prop_string(node, "onClick", state_store, scope))
        .or_else(|| prop_string(node, "action", state_store, scope))
        .unwrap_or_default();

    let node_snapshot = node.clone();
    let scope_snapshot = scope.clone();

    rsx! {
        button {
            class: "{class}",
            style: "{style}",
            disabled: disabled,
            onclick: move |_| {
                dispatch_action(
                    action_log,
                    &action_name,
                    &node_snapshot,
                    JsonValue::Null,
                    state_store,
                    &scope_snapshot,
                );
            },
            "{label}"
        }
    }
}

pub(super) fn render_input_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, "formo-input");
    let style = node_style(node);
    let disabled = prop_bool(node, "disabled", state_store, scope, false);

    let explicit_state_key = prop_string(node, "value", state_store, scope)
        .or_else(|| prop_string(node, "bind", state_store, scope))
        .or_else(|| prop_string(node, "name", state_store, scope));

    let state_key = explicit_state_key
        .clone()
        .unwrap_or_else(|| format!("__local_input::{}", node.id));

    let default_value = prop_string(node, "defaultValue", state_store, scope).unwrap_or_default();
    let current_value = read_state_string(state_store, &state_key).unwrap_or(default_value);

    let placeholder = prop_string(node, "placeholder", state_store, scope).unwrap_or_default();
    let input_type = prop_string(node, "inputType", state_store, scope)
        .unwrap_or_else(|| "text".to_string())
        .to_ascii_lowercase();

    let action_name = prop_string(node, "onChange", state_store, scope).unwrap_or_default();
    let node_snapshot = node.clone();
    let scope_snapshot = scope.clone();

    rsx! {
        input {
            class: "{class}",
            style: "{style}",
            disabled: disabled,
            r#type: "{input_type}",
            placeholder: "{placeholder}",
            value: "{current_value}",
            oninput: move |evt| {
                let value = evt.value();
                write_state(state_store, &state_key, JsonValue::String(value.clone()));
                if explicit_state_key.is_some() {
                    dispatch_action(
                        action_log,
                        &action_name,
                        &node_snapshot,
                        JsonValue::String(value),
                        state_store,
                        &scope_snapshot,
                    );
                }
            }
        }
    }
}

pub(super) fn render_checkbox_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, "formo-check-wrap");
    let style = node_style(node);
    let disabled = prop_bool(node, "disabled", state_store, scope, false);

    let explicit_state_key = prop_string(node, "checked", state_store, scope);
    let state_key = explicit_state_key
        .clone()
        .unwrap_or_else(|| format!("__local_check::{}", node.id));

    let initial = prop_bool(node, "checked", state_store, scope, false);
    let checked = read_state_bool(state_store, &state_key).unwrap_or(initial);
    let action_name = prop_string(node, "onChange", state_store, scope).unwrap_or_default();
    let label = prop_string(node, "label", state_store, scope).unwrap_or_default();

    let node_snapshot = node.clone();
    let scope_snapshot = scope.clone();

    rsx! {
        button {
            class: "{class}",
            style: "{style}",
            disabled: disabled,
            onclick: move |_| {
                let next = !checked;
                write_state(state_store, &state_key, JsonValue::Bool(next));
                if explicit_state_key.is_some() {
                    dispatch_action(
                        action_log,
                        &action_name,
                        &node_snapshot,
                        JsonValue::Bool(next),
                        state_store,
                        &scope_snapshot,
                    );
                }
            },
            span { class: "formo-checkbox", "{if checked { \"[x]\" } else { \"[ ]\" }}" }
            span { "{label}" }
        }
    }
}

pub(super) fn render_switch_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, "formo-switch-wrap");
    let style = node_style(node);
    let disabled = prop_bool(node, "disabled", state_store, scope, false);

    let explicit_state_key = prop_string(node, "checked", state_store, scope);
    let state_key = explicit_state_key
        .clone()
        .unwrap_or_else(|| format!("__local_switch::{}", node.id));

    let initial = prop_bool(node, "checked", state_store, scope, false);
    let checked = read_state_bool(state_store, &state_key).unwrap_or(initial);
    let action_name = prop_string(node, "onChange", state_store, scope).unwrap_or_default();
    let label = prop_string(node, "label", state_store, scope)
        .unwrap_or_else(|| "Switch".to_string());

    let node_snapshot = node.clone();
    let scope_snapshot = scope.clone();

    rsx! {
        button {
            class: "{class}",
            style: "{style}",
            disabled: disabled,
            onclick: move |_| {
                let next = !checked;
                write_state(state_store, &state_key, JsonValue::Bool(next));
                if explicit_state_key.is_some() {
                    dispatch_action(
                        action_log,
                        &action_name,
                        &node_snapshot,
                        JsonValue::Bool(next),
                        state_store,
                        &scope_snapshot,
                    );
                }
            },
            span { class: "formo-switch", "{if checked { \"ON\" } else { \"OFF\" }}" }
            span { "{label}" }
        }
    }
}
