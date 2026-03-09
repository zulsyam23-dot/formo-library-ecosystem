use crate::model::NativeNode;
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

use super::shared::{node_class, node_style};
use super::state::{derive_for_item_key, prop_bool, prop_list, prop_literal_string};
use super::{render_node, ActionLog, NativeState, RenderScope};

pub(super) fn render_container_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, container_class(node));
    let style = node_style(node);

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            for child in &node.children {
                {render_node(child, scope, state_store, action_log)}
            }
        }
    }
}

pub(super) fn render_fragment_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    rsx! {
        Fragment {
            for child in &node.children {
                {render_node(child, scope, state_store, action_log)}
            }
        }
    }
}

pub(super) fn render_if_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let has_when = node.props.contains_key("when");
    let has_condition = node.props.contains_key("condition");

    let should_show = if has_when {
        prop_bool(node, "when", state_store, scope, false)
    } else if has_condition {
        prop_bool(node, "condition", state_store, scope, false)
    } else {
        false
    };

    if should_show {
        render_fragment_html(node, scope, state_store, action_log)
    } else {
        rsx! { Fragment {} }
    }
}

pub(super) fn render_for_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let each_items = prop_list(node, "each", state_store, scope);
    if each_items.is_empty() {
        return rsx! { Fragment {} };
    }

    let alias = prop_literal_string(node, "as").unwrap_or_else(|| "item".to_string());
    let class = node_class(node, "formo-for-container");
    let style = node_style(node);

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            for (index, item) in each_items.iter().enumerate() {
                let mut next_scope = scope.clone();
                next_scope.insert(alias.clone(), item.clone());
                next_scope.insert(format!("{}Index", alias), JsonValue::from(index as i64));
                next_scope.insert(
                    format!("{}Key", alias),
                    JsonValue::String(derive_for_item_key(item, index)),
                );

                let item_key = next_scope
                    .get(&format!("{}Key", alias))
                    .and_then(|value| value.as_str())
                    .unwrap_or("idx:0")
                    .to_string();

                div {
                    key: "{item_key}",
                    class: "formo-for-item",
                    for child in &node.children {
                        {render_node(child, &next_scope, state_store, action_log)}
                    }
                }
            }
        }
    }
}

pub(super) fn render_fallback_html(
    node: &NativeNode,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
    action_log: Signal<ActionLog>,
) -> Element {
    let class = node_class(node, "formo-fallback");
    let style = node_style(node);

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            p {
                class: "formo-fallback-label",
                "{node.widget} [{node.id}]"
            }
            for child in &node.children {
                {render_node(child, scope, state_store, action_log)}
            }
        }
    }
}

fn container_class(node: &NativeNode) -> &'static str {
    match node.widget.as_str() {
        "Page" => "formo-page",
        "Column" => "formo-column",
        "Row" => "formo-row",
        "Stack" => "formo-stack",
        "Card" => "formo-card",
        "Window" => "formo-window",
        "Scroll" => "formo-scroll",
        _ => "",
    }
}
