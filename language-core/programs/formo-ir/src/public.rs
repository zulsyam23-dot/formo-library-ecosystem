use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const IR_VERSION: &str = "0.3.0";
pub const IR_SCHEMA_ID: &str = "https://formo.dev/schema/ir/0.3.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Web,
    Desktop,
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SourceLoc {
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Value {
    pub t: String,
    pub v: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub style_refs: Vec<String>,
    pub children: Vec<String>,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrComponent {
    pub id: String,
    pub name: String,
    pub root_node_id: String,
    pub exports: bool,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StyleSelector {
    pub component: String,
    pub part: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrStyle {
    pub id: String,
    pub selector: StyleSelector,
    pub decls: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub canonical_decls: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub level: String,
    pub message: String,
    pub source: SourceLoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IrProgram {
    pub ir_version: String,
    pub entry: String,
    pub target: Target,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    pub components: Vec<IrComponent>,
    pub nodes: Vec<IrNode>,
    pub styles: Vec<IrStyle>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct OutputFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct BackendOutput {
    pub files: Vec<OutputFile>,
}

pub trait Backend {
    fn emit(&self, ir: &IrProgram) -> Result<BackendOutput, String>;
}

pub fn effective_style_decls(style: &IrStyle) -> BTreeMap<String, Value> {
    if !style.canonical_decls.is_empty() {
        return style.canonical_decls.clone();
    }
    normalize_style_decls(&style.decls)
}

pub fn normalize_style_decls(decls: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();

    // Apply aliases first, then canonical keys so explicitly canonical keys win deterministically.
    for pass in [false, true] {
        for (raw_key, raw_value) in decls {
            let key = canonical_style_key(raw_key);
            if key.is_empty() {
                continue;
            }

            let is_canonical = raw_key.trim() == key;
            if is_canonical != pass {
                continue;
            }

            out.insert(key.clone(), canonical_style_value(&key, raw_value));
        }
    }

    out
}

fn canonical_style_key(raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return String::new();
    }
    if text.starts_with("--") {
        return text.to_string();
    }

    let mut out = String::new();
    let mut prev_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_uppercase() {
            if !out.is_empty() && !prev_dash {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
            continue;
        }
        if ch == '_' || ch == ' ' {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
            continue;
        }
        if ch == '-' {
            if !prev_dash && !out.is_empty() {
                out.push('-');
            }
            prev_dash = true;
            continue;
        }

        out.push(ch.to_ascii_lowercase());
        prev_dash = false;
    }
    out
}

fn canonical_style_value(key: &str, value: &Value) -> Value {
    let Some(raw_text) = value.v.as_str() else {
        return value.clone();
    };
    let normalized = raw_text.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return value.clone();
    }

    let mapped = match key {
        "align-items" => canonical_align_value(&normalized),
        "align-self" => canonical_align_value(&normalized),
        "justify-content" => canonical_justify_value(&normalized),
        "text-align" => canonical_text_align_value(&normalized),
        "overflow" => canonical_overflow_value(&normalized),
        "flex-wrap" => canonical_wrap_value(&normalized),
        "flex-direction" => canonical_direction_value(&normalized),
        "display" => canonical_display_value(&normalized),
        _ => return value.clone(),
    };

    Value {
        t: value.t.clone(),
        v: serde_json::Value::String(mapped.to_string()),
    }
}

fn canonical_align_value(raw: &str) -> &'static str {
    match raw {
        "start"
        | "flex-start"
        | "left"
        | "top"
        | "baseline"
        | "normal"
        | "self-start"
        | "safe start"
        | "unsafe start" => "start",
        "center" => "center",
        "end" | "flex-end" | "right" | "bottom" | "self-end" | "safe end" | "unsafe end" => {
            "end"
        }
        "stretch" => "stretch",
        _ => "start",
    }
}

fn canonical_justify_value(raw: &str) -> &'static str {
    match raw {
        "start" | "flex-start" | "left" | "top" => "start",
        "center" => "center",
        "end" | "flex-end" | "right" | "bottom" => "end",
        "space-between" | "space-around" | "space-evenly" => "space-between",
        _ => "start",
    }
}

fn canonical_text_align_value(raw: &str) -> &'static str {
    match raw {
        "left" | "start" => "start",
        "center" => "center",
        "right" | "end" => "end",
        _ => "start",
    }
}

fn canonical_overflow_value(raw: &str) -> &'static str {
    match raw {
        "hidden" => "hidden",
        "scroll" | "auto" | "overlay" => "scroll",
        "visible" => "visible",
        _ => "visible",
    }
}

fn canonical_wrap_value(raw: &str) -> &'static str {
    match raw {
        "wrap" | "wrap-reverse" => "wrap",
        _ => "nowrap",
    }
}

fn canonical_direction_value(raw: &str) -> &'static str {
    match raw {
        "row" | "row-reverse" => "row",
        "column" | "column-reverse" => "column",
        _ => "column",
    }
}

fn canonical_display_value(raw: &str) -> &'static str {
    match raw {
        "none" => "none",
        "flex" | "inline-flex" => "flex",
        "block" | "inline" | "inline-block" => "block",
        _ => "block",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_style_decls_canonicalizes_keys_and_shared_values() {
        let mut decls = BTreeMap::new();
        decls.insert(
            "alignItems".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("baseline".to_string()),
            },
        );
        decls.insert(
            "justify-content".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("space-around".to_string()),
            },
        );
        decls.insert(
            "alignSelf".to_string(),
            Value {
                t: "string".to_string(),
                v: serde_json::Value::String("self-end".to_string()),
            },
        );

        let out = normalize_style_decls(&decls);
        assert_eq!(
            out.get("align-items")
                .and_then(|v| v.v.as_str())
                .unwrap_or(""),
            "start"
        );
        assert_eq!(
            out.get("justify-content")
                .and_then(|v| v.v.as_str())
                .unwrap_or(""),
            "space-between"
        );
        assert_eq!(
            out.get("align-self")
                .and_then(|v| v.v.as_str())
                .unwrap_or(""),
            "end"
        );
    }
}
