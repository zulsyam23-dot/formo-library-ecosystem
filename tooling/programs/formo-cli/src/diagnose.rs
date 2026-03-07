use crate::args::CheckArgs;
use crate::error::CliError;
use crate::json_output::{
    attach_schema_if_enabled, build_error_meta, classify_error_stage, emit_json,
    DIAGNOSE_JSON_SCHEMA_ID,
};
use crate::lsp_output::{
    build_lsp_failure_payload, build_lsp_failure_payload_with_diagnostics,
    build_lsp_success_payload,
};
use crate::pipeline::pipeline;
use crate::term::{print_error, print_warn};
use formo_parser::parse_with_recovery;
use serde_json::json;
use std::fs;

pub fn run_diagnose(args: &CheckArgs) -> Result<(), CliError> {
    if let Ok(source) = fs::read_to_string(&args.input) {
        let recovery = parse_with_recovery(&source);
        if !recovery.diagnostics.is_empty() {
            let parser_diagnostics = recovery
                .diagnostics
                .iter()
                .map(|raw| parser_recovery_diag_meta(&args.input, raw))
                .collect::<Vec<_>>();
            let error_meta = parser_diagnostics[0].clone();
            let err = format_error_from_meta(&error_meta);
            let stage = "parser";

            if args.json {
                let mut payload = if args.lsp {
                    build_lsp_failure_payload_with_diagnostics(
                        &args.input,
                        stage,
                        &err,
                        &error_meta,
                        &parser_diagnostics,
                    )
                } else {
                    json!({
                        "ok": false,
                        "input": args.input,
                        "stage": stage,
                        "error": err,
                        "errorMeta": error_meta,
                        "diagnostics": parser_diagnostics,
                    })
                };
                attach_schema_if_enabled(&mut payload, args.json_schema, DIAGNOSE_JSON_SCHEMA_ID);
                emit_json(&payload, args.json_pretty)?;
                return Err(CliError::printed(err));
            }

            print_warn(&format!("diagnose failed: {}", args.input));
            print_warn(&format!(
                "stage={stage} diagnostics={}",
                parser_diagnostics.len()
            ));
            for diag in &parser_diagnostics {
                print_error(&format_error_from_meta(diag));
            }
            return Err(CliError::printed(err));
        }
    }

    match pipeline(&args.input) {
        Ok(ir) => {
            if args.json {
                let mut payload = if args.lsp {
                    build_lsp_success_payload(&args.input, &ir)
                } else {
                    json!({
                        "ok": true,
                        "input": args.input,
                        "entry": ir.entry,
                        "stats": {
                            "components": ir.components.len(),
                            "nodes": ir.nodes.len(),
                            "styles": ir.styles.len(),
                            "tokens": ir.tokens.len(),
                            "diagnostics": ir.diagnostics.len(),
                        },
                        "diagnostics": ir.diagnostics,
                    })
                };
                attach_schema_if_enabled(&mut payload, args.json_schema, DIAGNOSE_JSON_SCHEMA_ID);
                emit_json(&payload, args.json_pretty)?;
            } else {
                println!("diagnose ok: {}", args.input);
                println!(
                    "components={} nodes={} styles={} tokens={} diagnostics={}",
                    ir.components.len(),
                    ir.nodes.len(),
                    ir.styles.len(),
                    ir.tokens.len(),
                    ir.diagnostics.len()
                );
            }
            Ok(())
        }
        Err(err) => {
            let stage = classify_error_stage(&err);
            let error_meta = build_error_meta(&err);
            if args.json {
                let mut payload = if args.lsp {
                    build_lsp_failure_payload(&args.input, stage, &err, &error_meta)
                } else {
                    json!({
                        "ok": false,
                        "input": args.input,
                        "stage": stage,
                        "error": err,
                        "errorMeta": error_meta,
                    })
                };
                attach_schema_if_enabled(&mut payload, args.json_schema, DIAGNOSE_JSON_SCHEMA_ID);
                emit_json(&payload, args.json_pretty)?;
                Err(CliError::printed(err))
            } else {
                print_warn(&format!("diagnose failed: {}", args.input));
                print_warn(&format!("stage={stage}"));
                print_error(&err);
                Err(CliError::printed(err))
            }
        }
    }
}

fn parser_recovery_diag_meta(input: &str, raw: &str) -> serde_json::Value {
    let mut meta = build_error_meta(raw);
    if let serde_json::Value::Object(map) = &mut meta {
        map.entry("code".to_string())
            .or_insert_with(|| serde_json::Value::String("E1100".to_string()));
        map.entry("file".to_string())
            .or_insert_with(|| serde_json::Value::String(input.to_string()));
        map.entry("line".to_string())
            .or_insert_with(|| serde_json::Value::Number(1.into()));
        map.entry("col".to_string())
            .or_insert_with(|| serde_json::Value::Number(1.into()));
        map.entry("message".to_string())
            .or_insert_with(|| serde_json::Value::String(raw.to_string()));
    }
    meta
}

fn format_error_from_meta(meta: &serde_json::Value) -> String {
    let code = meta
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("E1100");
    let file = meta
        .get("file")
        .and_then(|value| value.as_str())
        .unwrap_or("<input>");
    let line = meta
        .get("line")
        .and_then(|value| value.as_u64())
        .unwrap_or(1);
    let col = meta
        .get("col")
        .and_then(|value| value.as_u64())
        .unwrap_or(1);
    let message = meta
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("parser error");
    format!("{code} {file}:{line}:{col} {message}")
}
