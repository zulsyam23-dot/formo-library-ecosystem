use crate::error::CliError;
use serde_json::json;

pub const CHECK_JSON_SCHEMA_ID: &str = "https://formo.dev/schema/check-result/1";
pub const DIAGNOSE_JSON_SCHEMA_ID: &str = "https://formo.dev/schema/diagnose-result/1";
pub const DOCTOR_JSON_SCHEMA_ID: &str = "https://formo.dev/schema/doctor-result/1";

pub fn emit_json(payload: &serde_json::Value, pretty: bool) -> Result<(), CliError> {
    let text = if pretty {
        serde_json::to_string_pretty(payload)
            .map_err(|e| CliError::new(format!("cannot serialize json output: {e}")))?
    } else {
        payload.to_string()
    };
    println!("{text}");
    Ok(())
}

pub fn attach_schema_if_enabled(payload: &mut serde_json::Value, enabled: bool, schema_id: &str) {
    if !enabled {
        return;
    }
    if let serde_json::Value::Object(map) = payload {
        map.insert(
            "schema".to_string(),
            json!({
                "id": schema_id,
                "version": 1
            }),
        );
    }
}

pub fn classify_error_stage(err: &str) -> &'static str {
    if let Some((code, _)) = err.split_once(' ') {
        if code.starts_with("E11") {
            return "parser";
        }
        if code.starts_with("E12") {
            return "resolver";
        }
        if code.starts_with("E13") {
            return "style";
        }
        if code.starts_with("E14") {
            return "lowering";
        }
        if code.starts_with("E2") {
            return "typer";
        }
    }

    if err.contains("cyclic import") || err.contains("cannot resolve path") {
        return "resolver";
    }
    if err.contains("unknown style") || err.contains("duplicate style") {
        return "style";
    }
    if err.contains("expected `") || err.contains("unterminated") || err.contains("parse error") {
        return "parser";
    }
    "pipeline"
}

pub fn build_error_meta(err: &str) -> serde_json::Value {
    if let Some((code, rest)) = split_error_code(err) {
        if let Some((file, line, col, message)) = parse_location_prefix(rest) {
            return json!({
                "code": code,
                "file": file,
                "line": line,
                "col": col,
                "message": message,
            });
        }

        return json!({
            "code": code,
            "message": rest,
        });
    }

    if let Some((file, line, col, message)) = parse_location_prefix(err) {
        return json!({
            "file": file,
            "line": line,
            "col": col,
            "message": message,
        });
    }

    if let Some((line, col, message)) = parse_at_line_col_suffix(err) {
        return json!({
            "line": line,
            "col": col,
            "message": message,
        });
    }

    json!({
        "message": err
    })
}

fn split_error_code(err: &str) -> Option<(&str, &str)> {
    let (code, rest) = err.split_once(' ')?;
    if code.len() > 1 && code.starts_with('E') && code[1..].chars().all(|ch| ch.is_ascii_digit()) {
        Some((code, rest.trim()))
    } else {
        None
    }
}

fn parse_location_prefix(input: &str) -> Option<(&str, usize, usize, &str)> {
    let (location, message) = input.split_once(' ')?;
    let mut parts = location.rsplitn(3, ':');
    let col = parts.next()?.parse::<usize>().ok()?;
    let line = parts.next()?.parse::<usize>().ok()?;
    let file = parts.next()?;
    if file.is_empty() {
        return None;
    }
    Some((file, line, col, message.trim()))
}

fn parse_at_line_col_suffix(input: &str) -> Option<(usize, usize, &str)> {
    let marker = " at ";
    let idx = input.rfind(marker)?;
    let message = input[..idx].trim();
    let location = input[idx + marker.len()..].trim();
    let (line_raw, col_raw) = location.split_once(':')?;
    let line = line_raw.parse::<usize>().ok()?;
    let col = col_raw.parse::<usize>().ok()?;
    Some((line, col, message))
}
