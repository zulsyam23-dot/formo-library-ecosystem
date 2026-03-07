mod args;
mod benchmark;
mod diagnose;
mod doctor;
mod error;
mod fmt_cmd;
mod json_output;
mod lowering;
mod lsp_bridge;
mod lsp_output;
mod output;
mod pipeline;
mod term;
mod watch;

use args::{
    parse_benchmark_args, parse_build_args, parse_check_args, parse_doctor_args, parse_fmt_args,
    parse_lsp_args, print_help,
};
use error::CliError;
use json_output::{
    attach_schema_if_enabled, build_error_meta, classify_error_stage, emit_json,
    CHECK_JSON_SCHEMA_ID,
};
use pipeline::pipeline;
use serde_json::json;
use std::env;

fn main() {
    if let Err(err) = run() {
        if !err.already_printed {
            term::print_error(&err.message);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "check" => run_check(&args[2..]),
        "diagnose" => {
            let check_args = parse_check_args(&args[2..])?;
            if check_args.watch {
                watch::run_watch_loop("diagnose", &check_args.input, || {
                    diagnose::run_diagnose(&check_args)
                })
            } else {
                diagnose::run_diagnose(&check_args)
            }
        }
        "fmt" => {
            let fmt_args = parse_fmt_args(&args[2..])?;
            fmt_cmd::run_fmt(&fmt_args)
        }
        "lsp" => {
            let lsp_args = parse_lsp_args(&args[2..])?;
            lsp_bridge::run_lsp(&lsp_args)
        }
        "doctor" => {
            let doctor_args = parse_doctor_args(&args[2..])?;
            doctor::run_doctor(&doctor_args)
        }
        "bench" => {
            let bench_args = parse_benchmark_args(&args[2..])?;
            benchmark::run_benchmark(&bench_args)
        }
        "build" => run_build(&args[2..]),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        cmd => Err(CliError::new(format!("unknown command: {cmd}"))),
    }
}

fn run_check(raw_args: &[String]) -> Result<(), CliError> {
    let check_args = parse_check_args(raw_args)?;
    if check_args.lsp {
        return Err(CliError::new(
            "`--lsp` is only supported by `diagnose` command",
        ));
    }
    if check_args.watch {
        return watch::run_watch_loop("check", &check_args.input, || run_check_once(&check_args));
    }
    run_check_once(&check_args)
}

fn run_check_once(check_args: &args::CheckArgs) -> Result<(), CliError> {
    if check_args.json {
        match pipeline(&check_args.input) {
            Ok(ir) => {
                let mut payload = json!({
                    "ok": true,
                    "input": check_args.input,
                    "entry": ir.entry,
                    "diagnosticCount": ir.diagnostics.len(),
                    "diagnostics": ir.diagnostics,
                });
                attach_schema_if_enabled(
                    &mut payload,
                    check_args.json_schema,
                    CHECK_JSON_SCHEMA_ID,
                );
                emit_json(&payload, check_args.json_pretty)?;
                Ok(())
            }
            Err(err) => {
                let stage = classify_error_stage(&err);
                let mut payload = json!({
                    "ok": false,
                    "input": check_args.input,
                    "stage": stage,
                    "error": err,
                    "errorMeta": build_error_meta(&err),
                });
                attach_schema_if_enabled(
                    &mut payload,
                    check_args.json_schema,
                    CHECK_JSON_SCHEMA_ID,
                );
                emit_json(&payload, check_args.json_pretty)?;
                Err(CliError::printed(err))
            }
        }
    } else {
        pipeline(&check_args.input)?;
        println!("check passed: {}", check_args.input);
        Ok(())
    }
}

fn run_build(raw_args: &[String]) -> Result<(), CliError> {
    let build_args = parse_build_args(raw_args)?;
    if build_args.watch {
        return watch::run_watch_loop("build", &build_args.input, || run_build_once(&build_args));
    }
    run_build_once(&build_args)
}

fn run_build_once(build_args: &args::BuildArgs) -> Result<(), CliError> {
    let ir = pipeline(&build_args.input)?;
    output::emit_target(
        &ir,
        &build_args.target,
        &build_args.out_dir,
        build_args.prod,
    )?;
    let mode = if build_args.prod {
        "production"
    } else {
        "development"
    };
    println!(
        "build ok: target={} out={} mode={}",
        build_args.target, build_args.out_dir, mode
    );
    Ok(())
}
