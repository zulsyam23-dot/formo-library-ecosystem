use crate::model::NativeNode;
use crate::render::{NativeState, RenderScope};
use dioxus::prelude::Signal;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub struct ActionEvent {
    pub name: String,
    pub payload: JsonValue,
    pub node_id: String,
    pub node_widget: String,
    pub scope: RenderScope,
    pub state: NativeState,
}

pub type ActionHandler = fn(ActionEvent, Signal<NativeState>);

pub fn invoke(
    action_name: &str,
    node: &NativeNode,
    payload: JsonValue,
    scope: &RenderScope,
    state_store: Signal<NativeState>,
) -> Result<bool, String> {
    let handler: ActionHandler = match action_name {
{{ACTION_MATCH_ARMS}}
        _ => return Ok(false),
    };

    let event = ActionEvent {
        name: action_name.to_string(),
        payload,
        node_id: node.id.clone(),
        node_widget: node.widget.clone(),
        scope: scope.clone(),
        state: state_store.read().clone(),
    };

    run_handler(handler, event, state_store)
}

fn run_handler(
    handler: ActionHandler,
    event: ActionEvent,
    state_store: Signal<NativeState>,
) -> Result<bool, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handler(event, state_store);
    })) {
        Ok(_) => Ok(true),
        Err(_) => Err("action handler panicked".to_string()),
    }
}

pub fn set_state(state_store: Signal<NativeState>, key: &str, value: JsonValue) {
    if key.trim().is_empty() {
        return;
    }
    state_store.write().insert(key.to_string(), value);
}

pub fn eval_set_expression(
    state_store: Signal<NativeState>,
    operands: &[(&str, &str)],
    operators: &[&str],
) -> Option<JsonValue> {
    if operands.is_empty() {
        return None;
    }

    let mut values = Vec::with_capacity(operands.len());
    for (kind, raw) in operands {
        values.push(resolve_operand_value(state_store.clone(), kind, raw)?);
    }

    if values.len() == 1 {
        return values.into_iter().next();
    }

    if operators.len() + 1 != values.len() {
        return None;
    }

    let mut acc = values[0].clone();
    for (index, operator) in operators.iter().enumerate() {
        let rhs = values[index + 1].clone();
        acc = apply_binary_operator(&acc, operator, &rhs)?;
    }

    Some(acc)
}

pub fn eval_set_expression_rpn(
    state_store: Signal<NativeState>,
    tokens: &[(&str, &str)],
) -> Option<JsonValue> {
    if tokens.is_empty() {
        return None;
    }

    let mut stack = Vec::with_capacity(tokens.len());
    for (kind, raw) in tokens {
        if *kind == "operator" {
            let rhs = stack.pop()?;
            let lhs = stack.pop()?;
            let next = apply_binary_operator(&lhs, raw, &rhs)?;
            stack.push(next);
            continue;
        }

        let value = resolve_operand_value(state_store.clone(), kind, raw)?;
        stack.push(value);
    }

    if stack.len() == 1 {
        stack.pop()
    } else {
        None
    }
}

fn resolve_operand_value(
    state_store: Signal<NativeState>,
    kind: &str,
    raw: &str,
) -> Option<JsonValue> {
    match kind {
        "stateRef" => state_store.read().get(raw).cloned(),
        "boolLiteral" => Some(JsonValue::Bool(raw.eq_ignore_ascii_case("true"))),
        "stringLiteral" => Some(JsonValue::String(raw.to_string())),
        "intLiteral" => raw.parse::<i64>().ok().map(JsonValue::from),
        "floatLiteral" => raw.parse::<f64>().ok().map(JsonValue::from),
        _ => None,
    }
}

fn apply_binary_operator(lhs: &JsonValue, op: &str, rhs: &JsonValue) -> Option<JsonValue> {
    match op {
        "add" => {
            if lhs.is_string() || rhs.is_string() {
                let text = format!("{}{}", json_stringify(lhs), json_stringify(rhs));
                Some(JsonValue::String(text))
            } else {
                Some(JsonValue::from(to_f64(lhs)? + to_f64(rhs)?))
            }
        }
        "sub" => Some(JsonValue::from(to_f64(lhs)? - to_f64(rhs)?)),
        "mul" => Some(JsonValue::from(to_f64(lhs)? * to_f64(rhs)?)),
        "div" => {
            let denom = to_f64(rhs)?;
            if denom.abs() < f64::EPSILON {
                None
            } else {
                Some(JsonValue::from(to_f64(lhs)? / denom))
            }
        }
        "mod" => {
            let denom = to_f64(rhs)?;
            if denom.abs() < f64::EPSILON {
                None
            } else {
                Some(JsonValue::from(to_f64(lhs)? % denom))
            }
        }
        "eq" => Some(JsonValue::Bool(loose_eq(lhs, rhs))),
        "notEq" => Some(JsonValue::Bool(!loose_eq(lhs, rhs))),
        "lt" => Some(JsonValue::Bool(to_f64(lhs)? < to_f64(rhs)?)),
        "ltEq" => Some(JsonValue::Bool(to_f64(lhs)? <= to_f64(rhs)?)),
        "gt" => Some(JsonValue::Bool(to_f64(lhs)? > to_f64(rhs)?)),
        "gtEq" => Some(JsonValue::Bool(to_f64(lhs)? >= to_f64(rhs)?)),
        "and" => Some(JsonValue::Bool(json_truthy(lhs) && json_truthy(rhs))),
        "or" => Some(JsonValue::Bool(json_truthy(lhs) || json_truthy(rhs))),
        _ => None,
    }
}

fn to_f64(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(num) => num.as_f64(),
        JsonValue::Bool(flag) => Some(if *flag { 1.0 } else { 0.0 }),
        JsonValue::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn json_truthy(value: &JsonValue) -> bool {
    match value {
        JsonValue::Bool(flag) => *flag,
        JsonValue::Number(num) => num.as_f64().is_some_and(|v| v.abs() > f64::EPSILON),
        JsonValue::String(text) => {
            let lowered = text.trim().to_ascii_lowercase();
            !lowered.is_empty() && lowered != "false" && lowered != "0"
        }
        JsonValue::Null => false,
        JsonValue::Array(items) => !items.is_empty(),
        JsonValue::Object(map) => !map.is_empty(),
    }
}

fn loose_eq(lhs: &JsonValue, rhs: &JsonValue) -> bool {
    if let (Some(a), Some(b)) = (to_f64(lhs), to_f64(rhs)) {
        return (a - b).abs() < f64::EPSILON;
    }
    if let (Some(a), Some(b)) = (lhs.as_bool(), rhs.as_bool()) {
        return a == b;
    }
    json_stringify(lhs) == json_stringify(rhs)
}

fn json_stringify(value: &JsonValue) -> String {
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

{{ACTION_HANDLER_STUBS}}
