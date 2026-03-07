use formo_ir::Value;
use formo_parser::AstValue;
use std::collections::HashMap;

pub(super) fn resolve_ast_value(
    value: &AstValue,
    env: &HashMap<String, AstValue>,
    depth: usize,
) -> Result<AstValue, String> {
    if depth > 32 {
        return Err("identifier expansion depth exceeded limit".to_string());
    }

    match value {
        AstValue::Identifier(name) => {
            if let Some(bound) = resolve_identifier_path_from_env(name, env) {
                resolve_ast_value(bound, env, depth + 1)
            } else {
                Ok(AstValue::Identifier(name.clone()))
            }
        }
        _ => Ok(value.clone()),
    }
}

fn resolve_identifier_path_from_env<'a>(
    raw: &str,
    env: &'a HashMap<String, AstValue>,
) -> Option<&'a AstValue> {
    if let Some(bound) = env.get(raw) {
        return Some(bound);
    }

    let mut parts = raw.split('.');
    let base = parts.next()?;
    let mut current = env.get(base)?;

    for part in parts {
        current = match current {
            AstValue::Object(entries) => entries.get(part)?,
            AstValue::List(items) => {
                let index = part.parse::<usize>().ok()?;
                items.get(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

pub(super) fn lower_value(value: &AstValue) -> Result<Value, String> {
    let lowered = match value {
        AstValue::String(v) => Value {
            t: "string".to_string(),
            v: serde_json::Value::String(v.clone()),
        },
        AstValue::Bool(v) => Value {
            t: "bool".to_string(),
            v: serde_json::Value::Bool(*v),
        },
        AstValue::Int(v) => Value {
            t: "int".to_string(),
            v: serde_json::Value::Number((*v).into()),
        },
        AstValue::Float(v) => {
            let number = serde_json::Number::from_f64(*v)
                .ok_or_else(|| format!("invalid float literal: {v}"))?;
            Value {
                t: "float".to_string(),
                v: serde_json::Value::Number(number),
            }
        }
        AstValue::Identifier(v) => Value {
            t: "string".to_string(),
            v: serde_json::Value::String(v.clone()),
        },
        AstValue::List(items) => Value {
            t: "list".to_string(),
            v: serde_json::Value::Array(lower_list_items(items)?),
        },
        AstValue::Object(entries) => Value {
            t: "object".to_string(),
            v: serde_json::Value::Object(lower_object_entries(entries)?),
        },
    };
    Ok(lowered)
}

fn lower_list_items(items: &[AstValue]) -> Result<Vec<serde_json::Value>, String> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let value = match item {
            AstValue::String(v) => serde_json::Value::String(v.clone()),
            AstValue::Bool(v) => serde_json::Value::Bool(*v),
            AstValue::Int(v) => serde_json::Value::Number((*v).into()),
            AstValue::Float(v) => {
                let number = serde_json::Number::from_f64(*v)
                    .ok_or_else(|| format!("invalid float literal in list: {v}"))?;
                serde_json::Value::Number(number)
            }
            AstValue::Identifier(v) => serde_json::Value::String(v.clone()),
            AstValue::List(nested) => serde_json::Value::Array(lower_list_items(nested)?),
            AstValue::Object(entries) => serde_json::Value::Object(lower_object_entries(entries)?),
        };
        out.push(value);
    }
    Ok(out)
}

fn lower_object_entries(
    entries: &std::collections::BTreeMap<String, AstValue>,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut out = serde_json::Map::new();
    for (key, value) in entries {
        let lowered = match value {
            AstValue::String(v) => serde_json::Value::String(v.clone()),
            AstValue::Bool(v) => serde_json::Value::Bool(*v),
            AstValue::Int(v) => serde_json::Value::Number((*v).into()),
            AstValue::Float(v) => {
                let number = serde_json::Number::from_f64(*v)
                    .ok_or_else(|| format!("invalid float literal in object: {v}"))?;
                serde_json::Value::Number(number)
            }
            AstValue::Identifier(v) => serde_json::Value::String(v.clone()),
            AstValue::List(items) => serde_json::Value::Array(lower_list_items(items)?),
            AstValue::Object(nested) => serde_json::Value::Object(lower_object_entries(nested)?),
        };
        out.insert(key.clone(), lowered);
    }
    Ok(out)
}
