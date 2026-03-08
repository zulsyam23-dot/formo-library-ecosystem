use crate::model::{FormoValue, NativeNode};
use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{ActionLog, NativeState, RenderScope};

pub(super) fn prop_record<'a>(node: &'a NativeNode, key: &str) -> Option<&'a FormoValue> {
    node.props.get(key)
}

pub(super) fn prop_literal_string(node: &NativeNode, key: &str) -> Option<String> {
    let prop = prop_record(node, key)?;
    let raw = prop.v.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }
    Some(raw.to_string())
}

pub(super) fn prop_string(node: &NativeNode, key: &str, scope: &RenderScope) -> Option<String> {
    let value = prop_value(node, key, scope)?;
    match value {
        JsonValue::Null => None,
        JsonValue::String(v) => {
            let trimmed = v.trim();
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
    state: &NativeState,
    scope: &RenderScope,
    fallback: bool,
) -> bool {
    let value = match prop_value(node, key, scope) {
        Some(v) => v,
        None => return fallback,
    };
    bool_from_json(&value, state, fallback)
}

pub(super) fn prop_list(
    node: &NativeNode,
    key: &str,
    state: &NativeState,
    scope: &RenderScope,
) -> Vec<JsonValue> {
    let value = match prop_value(node, key, scope) {
        Some(v) => v,
        None => return Vec::new(),
    };

    if let JsonValue::Array(items) = value {
        return items.clone();
    }

    if let JsonValue::String(text) = value {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        if let Some(state_value) = state.get(trimmed) {
            if let JsonValue::Array(items) = state_value {
                return items.clone();
            }
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
                if let JsonValue::Array(items) = parsed {
                    return items;
                }
            }
        }
    }

    Vec::new()
}

pub(super) fn read_state_string(state: &NativeState, key: &str) -> Option<String> {
    let value = state.get(key)?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    Some(value.to_string())
}

pub(super) fn read_state_bool(state: &NativeState, key: &str) -> Option<bool> {
    state.get(key).map(|v| json_truthy(v, state))
}

pub(super) fn json_truthy(value: &JsonValue, state: &NativeState) -> bool {
    bool_from_json(value, state, false)
}

pub(super) fn prop_len(node: &NativeNode, key: &str, scope: &RenderScope) -> Option<f32> {
    let value = prop_value(node, key, scope)?;
    parse_len_json(value)
}

pub(super) fn prop_usize(node: &NativeNode, key: &str, scope: &RenderScope) -> Option<usize> {
    let value = prop_value(node, key, scope)?;
    parse_usize_json(value)
}

pub(super) fn emit_action(
    action_log: &mut ActionLog,
    action_name: &str,
    node: &NativeNode,
    payload: JsonValue,
    state: &NativeState,
    scope: &RenderScope,
) {
    let name = action_name.trim();
    if name.is_empty() {
        return;
    }
    let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| "null".to_string());
    let state_json = serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string());
    let scope_json = serde_json::to_string(scope).unwrap_or_else(|_| "{}".to_string());
    let log = format!(
        "action={} node={} widget={} payload={} scope={} state={}",
        name, node.id, node.widget, payload_json, scope_json, state_json
    );
    action_log.push(log.clone());
    println!("{log}");
}

pub(super) fn resolve_scoped_value(raw: &JsonValue, scope: &RenderScope) -> JsonValue {
    let text = match raw.as_str() {
        Some(v) => v,
        None => return raw.clone(),
    };

    if let Some(exact) = scope.get(text) {
        return exact.clone();
    }

    if text.contains('.') {
        let mut parts = text.split('.').filter(|p| !p.is_empty());
        if let Some(base) = parts.next() {
            if let Some(base_value) = scope.get(base) {
                if let Some(scoped) = resolve_path_from_root(base_value, &parts.collect::<Vec<_>>()) {
                    return scoped;
                }
            }
        }
    }

    raw.clone()
}

fn prop_value(node: &NativeNode, key: &str, scope: &RenderScope) -> Option<JsonValue> {
    let prop = prop_record(node, key)?;
    Some(resolve_scoped_value(&prop.v, scope))
}

fn resolve_path_from_root(root: &JsonValue, parts: &[&str]) -> Option<JsonValue> {
    let mut cursor = root;
    for part in parts {
        match cursor {
            JsonValue::Array(items) => {
                let idx = part.parse::<usize>().ok()?;
                cursor = items.get(idx)?;
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
        JsonValue::Bool(v) => *v,
        JsonValue::Number(v) => {
            if let Some(i) = v.as_i64() {
                i != 0
            } else if let Some(f) = v.as_f64() {
                f.abs() > f64::EPSILON
            } else {
                fallback
            }
        }
        JsonValue::String(v) => {
            let text = v.trim().to_ascii_lowercase();
            if text == "true" {
                return true;
            }
            if text == "false" {
                return false;
            }
            if let Some(state_value) = state.get(v.trim()) {
                return json_truthy(state_value, state);
            }
            fallback
        }
        JsonValue::Null => fallback,
        JsonValue::Array(v) => !v.is_empty(),
        JsonValue::Object(v) => !v.is_empty(),
    }
}

fn parse_len_json(value: JsonValue) -> Option<f32> {
    if let Some(i) = value.as_i64() {
        return Some(i as f32);
    }
    if let Some(f) = value.as_f64() {
        return Some(f as f32);
    }
    if let Some(obj) = value.as_object() {
        return parse_len_object(obj);
    }
    value.as_str().and_then(parse_len_text)
}

fn parse_len_object(obj: &JsonMap<String, JsonValue>) -> Option<f32> {
    let raw = obj.get("value")?.as_f64()? as f32;
    let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
    match unit {
        "px" | "dp" => Some(raw),
        _ => None,
    }
}

fn parse_len_text(raw: &str) -> Option<f32> {
    let text = raw.trim();
    if let Some(px) = text.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if let Some(dp) = text.strip_suffix("dp") {
        return dp.trim().parse::<f32>().ok();
    }
    text.parse::<f32>().ok()
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

fn parse_usize_json(value: JsonValue) -> Option<usize> {
    if let Some(i) = value.as_i64() {
        return Some(i.max(0) as usize);
    }
    if let Some(f) = value.as_f64() {
        return Some(f.max(0.0).floor() as usize);
    }
    value.as_str().and_then(|raw| raw.trim().parse::<usize>().ok())
}

fn json_key_fragment(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        JsonValue::Number(v) => v.to_string(),
        JsonValue::String(v) => v.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "<json>".to_string())
        }
    }
}
