use crate::model::{FormoValue, NativeNode};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

const LENGTH_KEYS: &[&str] = &[
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "gap",
    "border-width",
    "border-radius",
    "font-size",
    "line-height",
    "top",
    "right",
    "bottom",
    "left",
    "flex-basis",
];

pub fn runtime_css() -> &'static str {
    r#":root {
  color-scheme: light;
  font-family: "Segoe UI", "SF Pro Text", "Noto Sans", sans-serif;
}

* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  padding: 0;
  background: #f5f7fb;
  color: #151515;
}

body {
  font-size: 14px;
}

.formo-native-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
}

.formo-native-header h1 {
  margin: 0;
  font-size: 18px;
}

.formo-native-header p {
  margin: 4px 0 0;
  color: #51607a;
}

.formo-parity-banner {
  background: #fff6dd;
  color: #7f4c00;
  border: 1px solid #f0d998;
  border-radius: 8px;
  padding: 8px 10px;
}

.formo-parity-details {
  background: #ffffff;
  border: 1px solid #d7dbe8;
  border-radius: 10px;
  padding: 8px 12px;
}

.formo-native-root {
  background: #ffffff;
  border: 1px solid #d7dbe8;
  border-radius: 12px;
  padding: 14px;
}

.formo-node {
  position: relative;
}

.formo-page,
.formo-column {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.formo-row {
  display: flex;
  flex-direction: row;
  gap: 8px;
  align-items: center;
}

.formo-scroll {
  overflow: auto;
}

.formo-card,
.formo-window {
  background: #ffffff;
  border: 1px solid #d7dbe8;
  border-radius: 10px;
  padding: 10px;
}

.formo-text {
  margin: 0;
}

.formo-button {
  cursor: pointer;
}

.formo-input,
.formo-checkbox,
.formo-switch {
  accent-color: #3d6ff5;
}

.formo-modal {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
  z-index: 999;
}

.formo-modal-body {
  width: min(680px, 92vw);
  max-height: 82vh;
  overflow: auto;
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #d7dbe8;
  padding: 14px;
}

.formo-fallback {
  border: 1px dashed #c9cedf;
  border-radius: 8px;
  padding: 8px;
  color: #4b566d;
}

.formo-empty {
  color: #51607a;
  padding: 12px;
}
"#
}

pub fn widget_class(widget: &str) -> &'static str {
    match widget {
        "Page" => "formo-page",
        "Column" => "formo-column",
        "Row" => "formo-row",
        "Scroll" => "formo-scroll",
        "Card" => "formo-card",
        "Window" => "formo-window",
        "Text" => "formo-text",
        "Button" => "formo-button",
        "Input" => "formo-input",
        "Checkbox" => "formo-checkbox",
        "Switch" => "formo-switch",
        "Modal" => "formo-modal",
        "Image" => "formo-image",
        "Spacer" => "formo-spacer",
        "Fragment" => "formo-fragment",
        "If" => "formo-if",
        "For" => "formo-for",
        _ => "formo-fallback",
    }
}

pub fn style_attr(node: &NativeNode) -> String {
    let mut css = BTreeMap::new();

    for (key, value) in &node.resolved_style {
        let css_key = normalize_css_key(key);
        if let Some(css_value) = value_to_css(&css_key, value) {
            css.insert(css_key, css_value);
        }
    }

    css.into_iter()
        .map(|(key, value)| format!("{key}:{value};"))
        .collect::<Vec<_>>()
        .join("")
}

fn value_to_css(css_key: &str, value: &FormoValue) -> Option<String> {
    match &value.v {
        JsonValue::Null => None,
        JsonValue::Bool(flag) => Some(if *flag { "true" } else { "false" }.to_string()),
        JsonValue::Number(num) => Some(number_to_css(css_key, num)),
        JsonValue::String(raw) => string_to_css(raw),
        JsonValue::Array(items) => {
            let joined = items
                .iter()
                .map(json_scalar_to_text)
                .collect::<Vec<_>>()
                .join(" ");
            if joined.trim().is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        JsonValue::Object(obj) => object_to_css_value(css_key, obj),
    }
}

fn object_to_css_value(
    css_key: &str,
    obj: &serde_json::Map<String, JsonValue>,
) -> Option<String> {
    let raw_value = obj.get("value")?;
    let unit = obj
        .get("unit")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    match raw_value {
        JsonValue::Number(num) => {
            let base = number_to_css(css_key, num);
            if unit.is_empty() {
                Some(base)
            } else if base.ends_with("px") || base.ends_with('%') {
                Some(base)
            } else {
                Some(format!("{}{}", base, unit))
            }
        }
        JsonValue::String(text) => {
            let value = text.trim();
            if value.is_empty() {
                None
            } else if unit.is_empty() {
                Some(value.to_string())
            } else {
                Some(format!("{}{}", value, unit))
            }
        }
        _ => None,
    }
}

fn number_to_css(css_key: &str, num: &serde_json::Number) -> String {
    let value = num
        .as_f64()
        .map(|v| {
            if (v - v.round()).abs() < f64::EPSILON {
                format!("{}", v as i64)
            } else {
                format!("{v}")
            }
        })
        .unwrap_or_else(|| num.to_string());

    if is_length_key(css_key) {
        format!("{value}px")
    } else {
        value
    }
}

fn string_to_css(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn normalize_css_key(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn is_length_key(key: &str) -> bool {
    LENGTH_KEYS.contains(&key)
}

fn json_scalar_to_text(value: &JsonValue) -> String {
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
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}
