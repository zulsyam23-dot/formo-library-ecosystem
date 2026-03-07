use crate::args::DoctorArgs;
use crate::error::CliError;
use crate::json_output::{
    attach_schema_if_enabled, build_error_meta, classify_error_stage, emit_json,
    DOCTOR_JSON_SCHEMA_ID,
};
use crate::pipeline::pipeline;
use crate::term::{print_error, print_warn};
use serde_json::json;
use std::env;
use std::path::Path;

pub fn run_doctor(args: &DoctorArgs) -> Result<(), CliError> {
    let cwd = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    let cargo_toml = Path::new("Cargo.toml").exists();
    let input_exists = Path::new(&args.input).exists();

    let compile_result = if input_exists {
        Some(pipeline(&args.input))
    } else {
        None
    };

    let mut ok = input_exists;
    if let Some(Err(_)) = &compile_result {
        ok = false;
    }

    if args.json {
        let mut payload = json!({
            "ok": ok,
            "cwd": cwd,
            "input": args.input,
            "checks": {
                "cargoToml": cargo_toml,
                "inputFile": input_exists,
            }
        });

        if let Some(result) = compile_result {
            match result {
                Ok(ir) => {
                    payload["pipeline"] = json!({
                        "ok": true,
                        "entry": ir.entry,
                        "stats": {
                            "components": ir.components.len(),
                            "nodes": ir.nodes.len(),
                            "styles": ir.styles.len(),
                            "tokens": ir.tokens.len(),
                            "diagnostics": ir.diagnostics.len(),
                        }
                    });
                }
                Err(err) => {
                    payload["pipeline"] = json!({
                        "ok": false,
                        "stage": classify_error_stage(&err),
                        "error": err,
                        "errorMeta": build_error_meta(&err),
                    });
                }
            }
        } else {
            payload["pipeline"] = json!({
                "ok": false,
                "stage": "preflight",
                "error": format!("input file not found: {}", args.input),
                "errorMeta": {
                    "message": format!("input file not found: {}", args.input),
                }
            });
        }

        attach_schema_if_enabled(&mut payload, args.json_schema, DOCTOR_JSON_SCHEMA_ID);
        emit_json(&payload, args.json_pretty)?;
    } else {
        println!("doctor: cwd={cwd}");
        println!(
            "check cargo.toml: {}",
            if cargo_toml { "ok" } else { "missing" }
        );
        println!(
            "check input file: {} ({})",
            if input_exists { "ok" } else { "missing" },
            args.input
        );

        match compile_result {
            Some(Ok(ir)) => {
                println!("check pipeline: ok");
                println!(
                    "stats components={} nodes={} styles={} tokens={} diagnostics={}",
                    ir.components.len(),
                    ir.nodes.len(),
                    ir.styles.len(),
                    ir.tokens.len(),
                    ir.diagnostics.len()
                );
            }
            Some(Err(err)) => {
                let stage = classify_error_stage(&err);
                print_warn("check pipeline: failed");
                print_warn(&format!("stage={stage}"));
                print_error(&err);
            }
            None => {
                print_warn("check pipeline: skipped");
                print_error(&format!("input file not found: {}", args.input));
            }
        }
    }

    if ok {
        Ok(())
    } else {
        Err(CliError::printed("doctor found issues"))
    }
}
