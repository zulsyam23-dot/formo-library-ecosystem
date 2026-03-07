use formo_ir::{Diagnostic, IrProgram};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn build_lsp_success_payload(input: &str, ir: &IrProgram) -> serde_json::Value {
    let mut grouped: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for diag in &ir.diagnostics {
        let uri = to_file_uri(&diag.source.file);
        grouped
            .entry(uri)
            .or_default()
            .push(ir_diagnostic_to_lsp(diag));
    }

    if grouped.is_empty() {
        grouped.insert(to_file_uri(input), Vec::new());
    }

    let documents = grouped
        .into_iter()
        .map(|(uri, diagnostics)| {
            json!({
                "uri": uri,
                "diagnostics": diagnostics
            })
        })
        .collect::<Vec<_>>();

    json!({
        "ok": true,
        "input": input,
        "entry": ir.entry,
        "documents": documents
    })
}

pub fn build_lsp_failure_payload(
    input: &str,
    stage: &str,
    err: &str,
    error_meta: &serde_json::Value,
) -> serde_json::Value {
    build_lsp_failure_payload_with_diagnostics(
        input,
        stage,
        err,
        error_meta,
        std::slice::from_ref(error_meta),
    )
}

pub fn build_lsp_failure_payload_with_diagnostics(
    input: &str,
    stage: &str,
    err: &str,
    error_meta: &serde_json::Value,
    diagnostics_meta: &[serde_json::Value],
) -> serde_json::Value {
    let diagnostic_items = if diagnostics_meta.is_empty() {
        vec![error_meta.clone()]
    } else {
        diagnostics_meta.to_vec()
    };

    let mut grouped: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for meta in &diagnostic_items {
        let (line, col, message) = extract_meta_position(meta, err);
        let uri = meta
            .get("file")
            .and_then(|value| value.as_str())
            .map(to_file_uri)
            .unwrap_or_else(|| to_file_uri(input));
        let code = meta
            .get("code")
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let diagnostic = lsp_diag(line, col, 1, code, message.to_string());
        grouped.entry(uri).or_default().push(diagnostic);
    }

    if grouped.is_empty() {
        grouped.insert(to_file_uri(input), Vec::new());
    }

    let documents = grouped
        .into_iter()
        .map(|(uri, diagnostics)| {
            json!({
                "uri": uri,
                "diagnostics": diagnostics
            })
        })
        .collect::<Vec<_>>();

    json!({
        "ok": false,
        "input": input,
        "stage": stage,
        "error": err,
        "errorMeta": error_meta,
        "documents": documents
    })
}

pub fn to_file_uri(path: &str) -> String {
    let path_buf = Path::new(path);
    let abs = if path_buf.is_absolute() {
        path_buf.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path_buf))
            .unwrap_or_else(|_| PathBuf::from(path))
    };
    let path_text = abs.to_string_lossy().replace('\\', "/");
    if path_text.starts_with('/') {
        format!("file://{path_text}")
    } else {
        format!("file:///{path_text}")
    }
}

fn extract_meta_position<'a>(
    meta: &'a serde_json::Value,
    fallback: &'a str,
) -> (usize, usize, &'a str) {
    let line = meta
        .get("line")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
        .unwrap_or(1)
        .saturating_sub(1);
    let col = meta
        .get("col")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
        .unwrap_or(1)
        .saturating_sub(1);
    let message = meta
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or(fallback);
    (line, col, message)
}

fn ir_diagnostic_to_lsp(diag: &Diagnostic) -> serde_json::Value {
    let severity = map_level_to_lsp_severity(&diag.level);
    let line = diag.source.line.saturating_sub(1);
    let col = diag.source.col.saturating_sub(1);
    lsp_diag(
        line,
        col,
        severity,
        Some(diag.code.clone()),
        diag.message.clone(),
    )
}

fn lsp_diag(
    line: usize,
    col: usize,
    severity: usize,
    code: Option<String>,
    message: String,
) -> serde_json::Value {
    let end_col = col.saturating_add(1);
    match code {
        Some(code_value) => json!({
            "range": {
                "start": {"line": line, "character": col},
                "end": {"line": line, "character": end_col}
            },
            "severity": severity,
            "code": code_value,
            "message": message,
            "source": "formo"
        }),
        None => json!({
            "range": {
                "start": {"line": line, "character": col},
                "end": {"line": line, "character": end_col}
            },
            "severity": severity,
            "message": message,
            "source": "formo"
        }),
    }
}

fn map_level_to_lsp_severity(level: &str) -> usize {
    match level {
        "error" => 1,
        "warning" => 2,
        "info" => 3,
        "hint" => 4,
        _ => 3,
    }
}
