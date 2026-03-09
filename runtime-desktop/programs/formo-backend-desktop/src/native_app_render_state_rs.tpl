use crate::model::{FormoValue, NativeNode};
use dioxus::prelude::*;
use serde_json::Value as JsonValue;

use super::{ActionLog, NativeState, RenderScope};

pub(super) fn prop_record<'a>(node: &'a NativeNode, key: &str) -> Option<&'a FormoValue> {
    node.props.get(key)
}

pub(super) fn prop_literal_string(node: &NativeNode, key: &str) -> Option<String> {
    let prop = prop_record(node, key)?;
    let raw = prop.v.as_str()?.trim();
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

pub(super) fn prop_string(
    node: &NativeNode,
    key: &str,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
) -> Option<String> {
    let value = prop_value(node, key, state_store, scope)?;
    match value {
        JsonValue::Null => None,
        JsonValue::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        other => Some(other.to_string()),
    }
}

pub(super) fn prop_bool(
    node: &NativeNode,
    key: &str,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
    fallback: bool,
) -> bool {
    let value = match prop_value(node, key, state_store, scope) {
        Some(value) => value,
        None => return fallback,
    };

    bool_from_json(&value, &state_store.read(), fallback)
}

pub(super) fn prop_list(
    node: &NativeNode,
    key: &str,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
) -> Vec<JsonValue> {
    let value = match prop_value(node, key, state_store, scope) {
        Some(value) => value,
        None => return Vec::new(),
    };

    if let JsonValue::Array(items) = value {
        return items;
    }

    if let JsonValue::String(raw) = value {
        let trimmed = raw.trim();
        if let Some(state_value) = state_store.read().get(trimmed) {
            if let JsonValue::Array(items) = state_value {
                return items.clone();
            }
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return serde_json::from_str::<JsonValue>(trimmed)
                .ok()
                .and_then(|parsed| parsed.as_array().cloned())
                .unwrap_or_default();
        }
    }

    Vec::new()
}

pub(super) fn read_state_string(state_store: Signal<NativeState>, key: &str) -> Option<String> {
    let state = state_store.read();
    let value = state.get(key)?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    Some(value.to_string())
}

pub(super) fn read_state_bool(state_store: Signal<NativeState>, key: &str) -> Option<bool> {
    let state = state_store.read();
    state.get(key).map(|value| bool_from_json(value, &state, false))
}

pub(super) fn write_state(state_store: Signal<NativeState>, key: &str, value: JsonValue) {
    if key.trim().is_empty() {
        return;
    }
    state_store.write().insert(key.to_string(), value);
}

pub(super) fn dispatch_action(
    action_log: Signal<ActionLog>,
    action_name: &str,
    node: &NativeNode,
    payload: JsonValue,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
) {
    let name = action_name.trim();
    if name.is_empty() {
        return;
    }

    let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| "null".to_string());
    let scope_json = serde_json::to_string(scope).unwrap_or_else(|_| "{}".to_string());
    let state_before =
        serde_json::to_string(&*state_store.read()).unwrap_or_else(|_| "{}".to_string());

    let invoke_result = crate::actions::invoke(name, node, payload, scope, state_store.clone());

    let (status, error_message) = match invoke_result {
        Ok(true) => ("handled", String::new()),
        Ok(false) => ("unhandled", String::new()),
        Err(err) => ("error", err),
    };

    let state_after =
        serde_json::to_string(&*state_store.read()).unwrap_or_else(|_| "{}".to_string());

    let mut log = format!(
        "action={} node={} widget={} status={} payload={} scope={} stateBefore={} stateAfter={}",
        name, node.id, node.widget, status, payload_json, scope_json, state_before, state_after
    );

    if !error_message.is_empty() {
        log.push_str(&format!(" error={error_message}"));
        eprintln!("[formo] action `{name}` failed: {error_message}");
    }

    println!("{log}");
    action_log.write().push(log);
}

pub(super) fn resolve_scoped_value(
    raw: &JsonValue,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
) -> JsonValue {
    let text = match raw.as_str() {
        Some(text) => text,
        None => return raw.clone(),
    };

    if let Some(exact) = scope.get(text) {
        return exact.clone();
    }

    if text.contains('.') {
        let mut parts = text.split('.').filter(|part| !part.is_empty());
        if let Some(base) = parts.next() {
            if let Some(base_value) = scope.get(base) {
                if let Some(value) = resolve_path_from_root(base_value, &parts.collect::<Vec<_>>()) {
                    return value;
                }
            }
        }
    }

    if let Some(state_value) = state_store.read().get(text) {
        return state_value.clone();
    }

    raw.clone()
}

pub(super) fn derive_for_item_key(item: &JsonValue, index: usize) -> String {
    if let Some(obj) = item.as_object() {
        if let Some(id) = obj.get("id") {
            if !id.is_null() {
                return format!("id:{}", json_key_fragment(id));
            }
        }
        if let Some(key) = obj.get("key") {
            if !key.is_null() {
                return format!("key:{}", json_key_fragment(key));
            }
        }
    }
    format!("idx:{index}")
}

fn prop_value(
    node: &NativeNode,
    key: &str,
    state_store: Signal<NativeState>,
    scope: &RenderScope,
) -> Option<JsonValue> {
    let prop = prop_record(node, key)?;
    Some(resolve_scoped_value(&prop.v, state_store, scope))
}

fn resolve_path_from_root(root: &JsonValue, parts: &[&str]) -> Option<JsonValue> {
    let mut cursor = root;
    for part in parts {
        match cursor {
            JsonValue::Array(items) => {
                let index = part.parse::<usize>().ok()?;
                cursor = items.get(index)?;
            }
            JsonValue::Object(map) => {
                cursor = map.get(*part)?;
            }
            _ => return None,
        }
    }
    Some(cursor.clone())
}

fn bool_from_json(value: &JsonValue, state: &NativeState, fallback: bool) -> bool {
    match value {
        JsonValue::Bool(flag) => *flag,
        JsonValue::Number(num) => {
            if let Some(int) = num.as_i64() {
                int != 0
            } else if let Some(float) = num.as_f64() {
                float.abs() > f64::EPSILON
            } else {
                fallback
            }
        }
        JsonValue::String(raw) => {
            let text = raw.trim().to_ascii_lowercase();
            if text == "true" {
                return true;
            }
            if text == "false" {
                return false;
            }
            if let Some(state_value) = state.get(raw.trim()) {
                return bool_from_json(state_value, state, fallback);
            }
            fallback
        }
        JsonValue::Null => fallback,
        JsonValue::Array(items) => !items.is_empty(),
        JsonValue::Object(map) => !map.is_empty(),
    }
}

fn json_key_fragment(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(flag) => {
            if *flag {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        JsonValue::Number(num) => num.to_string(),
        JsonValue::String(text) => text.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "<json>".to_string())
        }
    }
}
