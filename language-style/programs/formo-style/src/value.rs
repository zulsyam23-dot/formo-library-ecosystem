use crate::allowlist::is_token_key;
use formo_ir::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn parse_style_value(
    raw: &str,
    available_tokens: &BTreeMap<String, Value>,
    referenced_tokens: &mut BTreeSet<String>,
) -> Result<Value, String> {
    parse_style_value_with_depth(raw, available_tokens, referenced_tokens, 0)
}

fn parse_style_value_with_depth(
    raw: &str,
    available_tokens: &BTreeMap<String, Value>,
    referenced_tokens: &mut BTreeSet<String>,
    depth: usize,
) -> Result<Value, String> {
    if depth > 16 {
        return Err("token fallback nesting is too deep".to_string());
    }

    if let Some(token_ref) = parse_token_ref(raw) {
        referenced_tokens.insert(token_ref.name.clone());
        if let Some(value) = available_tokens.get(token_ref.name.as_str()) {
            return Ok(value.clone());
        }

        if let Some(fallback) = token_ref.fallback {
            return parse_style_value_with_depth(
                fallback.as_str(),
                available_tokens,
                referenced_tokens,
                depth + 1,
            );
        }

        return Err(format!("unknown token `{}`", token_ref.name));
    }

    Ok(parse_literal_style_value(raw))
}

#[derive(Debug, Clone)]
struct TokenRef {
    name: String,
    fallback: Option<String>,
}

fn parse_token_ref(raw: &str) -> Option<TokenRef> {
    let trimmed = raw.trim();
    let inner = trimmed.strip_prefix("token(")?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }

    let (name_raw, fallback_raw) = split_token_args(inner);
    let name = name_raw.trim();
    if !is_token_key(name) {
        return None;
    }

    let fallback = fallback_raw
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(str::to_string);

    Some(TokenRef {
        name: name.to_string(),
        fallback,
    })
}

fn split_token_args(raw: &str) -> (&str, Option<&str>) {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if in_string && ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let left = &raw[..idx];
                let right = &raw[idx + ch.len_utf8()..];
                return (left, Some(right));
            }
            _ => {}
        }
    }

    (raw, None)
}

fn parse_literal_style_value(raw: &str) -> Value {
    if let Some(color) = parse_hex_color(raw) {
        return Value {
            t: "color".to_string(),
            v: serde_json::Value::String(color),
        };
    }

    if let Some((value, unit)) = parse_len(raw) {
        let mut obj = serde_json::Map::new();
        let number =
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0));
        obj.insert("value".to_string(), serde_json::Value::Number(number));
        obj.insert(
            "unit".to_string(),
            serde_json::Value::String(unit.to_string()),
        );
        return Value {
            t: "len".to_string(),
            v: serde_json::Value::Object(obj),
        };
    }

    if raw == "true" {
        return Value {
            t: "bool".to_string(),
            v: serde_json::Value::Bool(true),
        };
    }

    if raw == "false" {
        return Value {
            t: "bool".to_string(),
            v: serde_json::Value::Bool(false),
        };
    }

    if let Some(stripped) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return Value {
            t: "string".to_string(),
            v: serde_json::Value::String(stripped.to_string()),
        };
    }

    if let Ok(parsed) = raw.parse::<i64>() {
        return Value {
            t: "int".to_string(),
            v: serde_json::Value::Number(parsed.into()),
        };
    }

    if let Ok(parsed) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(parsed) {
            return Value {
                t: "float".to_string(),
                v: serde_json::Value::Number(num),
            };
        }
    }

    Value {
        t: "string".to_string(),
        v: serde_json::Value::String(raw.to_string()),
    }
}

fn parse_hex_color(raw: &str) -> Option<String> {
    let hex = raw.strip_prefix('#')?;
    let valid_len = matches!(hex.len(), 6 | 8);
    if !valid_len || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{hex}"))
}

fn parse_len(raw: &str) -> Option<(f64, &str)> {
    let units = ["dp", "px", "%", "vw", "vh", "rem", "em"];
    for unit in units {
        if let Some(number_raw) = raw.strip_suffix(unit) {
            let number = number_raw.trim().parse::<f64>().ok()?;
            return Some((number, unit));
        }
    }
    None
}
