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
    parse_logic_args, parse_lsp_args, print_help,
};
use error::CliError;
use json_output::{
    attach_schema_if_enabled, build_error_meta, classify_error_stage, emit_json,
    CHECK_JSON_SCHEMA_ID,
};
use pipeline::pipeline;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        "logic" => run_logic(&args[2..]),
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

fn run_logic(raw_args: &[String]) -> Result<(), CliError> {
    let logic_args = parse_logic_args(raw_args)?;
    let source = fs::read_to_string(&logic_args.input).map_err(|err| {
        CliError::new(format!(
            "cannot read logic input `{}`: {err}",
            logic_args.input
        ))
    })?;

    match formo_logic::parse(&source) {
        Ok(program) => {
            let contract = formo_logic::runtime_contract(&program);
            let parity_ready_units = contract
                .units
                .iter()
                .filter(|unit| unit.parity_ready)
                .count();
            let parity_score = if contract.units.is_empty() {
                100.0
            } else {
                (parity_ready_units as f64 / contract.units.len() as f64) * 100.0
            };
            let mut event_count = 0usize;
            let mut symmetric_event_count = 0usize;
            let mut global_core_event_count = 0usize;

            let units = contract
                .units
                .iter()
                .map(|unit| {
                    let events = unit
                        .events
                        .iter()
                        .map(|event| {
                            event_count += 1;
                            let has_global_core = !event.global_calls.is_empty()
                                || event.set_count > 0
                                || event.emit_count > 0;
                            if has_global_core {
                                global_core_event_count += 1;
                            }
                            let symmetric_platform_calls = (event.web_calls.is_empty()
                                && event.desktop_calls.is_empty())
                                || (!event.web_calls.is_empty()
                                    && !event.desktop_calls.is_empty()
                                    && event.web_calls.len() == event.desktop_calls.len());
                            if symmetric_platform_calls {
                                symmetric_event_count += 1;
                            }
                            json!({
                                "name": event.name,
                                "totalActions": event.total_actions,
                                "setCount": event.set_count,
                                "emitCount": event.emit_count,
                                "throwCount": event.throw_count,
                                "breakCount": event.break_count,
                                "continueCount": event.continue_count,
                                "returnCount": event.return_count,
                                "ifCount": event.if_count,
                                "forCount": event.for_count,
                                "whileCount": event.while_count,
                                "matchCount": event.match_count,
                                "tryCount": event.try_count,
                                "catchCount": event.catch_count,
                                "globalCalls": event.global_calls,
                                "webCalls": event.web_calls,
                                "desktopCalls": event.desktop_calls,
                            })
                        })
                        .collect::<Vec<_>>();
                    json!({
                        "kind": unit.kind,
                        "name": unit.name,
                        "parityReady": unit.parity_ready,
                        "stateFieldCount": unit.state_field_count,
                        "typedStateFieldCount": unit.typed_state_field_count,
                        "functionCount": unit.function_count,
                        "typedFunctionCount": unit.typed_function_count,
                        "returningFunctionCount": unit.returning_function_count,
                        "enumCount": unit.enum_count,
                        "enumVariantCount": unit.enum_variant_count,
                        "structCount": unit.struct_count,
                        "typedStructCount": unit.typed_struct_count,
                        "structFieldCount": unit.struct_field_count,
                        "typeAliasCount": unit.type_alias_count,
                        "qualifiedTypeAliasCount": unit.qualified_type_alias_count,
                        "eventCount": unit.events.len(),
                        "events": events,
                    })
                })
                .collect::<Vec<_>>();
            let event_parity_score = if event_count == 0 {
                100.0
            } else {
                (symmetric_event_count as f64 / event_count as f64) * 100.0
            };

            let payload = json!({
                "ok": true,
                "input": logic_args.input,
                "module": contract.module,
                "useCount": program.uses.len(),
                "unitCount": contract.units.len(),
                "quality": {
                    "parityReadyUnits": parity_ready_units,
                    "parityScore": parity_score,
                    "eventCount": event_count,
                    "symmetricEventCount": symmetric_event_count,
                    "globalCoreEventCount": global_core_event_count,
                    "eventParityScore": event_parity_score,
                },
                "runtimeContract": {
                    "units": units,
                }
            });

            if let Some(manifest_out) = logic_args.rt_manifest_out.as_ref() {
                write_json_manifest(manifest_out, &payload)?;
            }

            if logic_args.json {
                emit_json(&payload, logic_args.json_pretty)?;
            } else {
                println!(
                    "logic check passed: {} (module={}, units={}, parityReady={}/{}, parityScore={:.2}%)",
                    logic_args.input,
                    contract.module,
                    contract.units.len(),
                    parity_ready_units,
                    contract.units.len(),
                    parity_score
                );
                if let Some(manifest_out) = logic_args.rt_manifest_out.as_ref() {
                    println!("runtime manifest written: {manifest_out}");
                }
            }
            Ok(())
        }
        Err(err) => {
            if logic_args.json {
                let payload = json!({
                    "ok": false,
                    "input": logic_args.input,
                    "stage": "logic-parser",
                    "error": err,
                });
                emit_json(&payload, logic_args.json_pretty)?;
                Err(CliError::printed("logic parse failed"))
            } else {
                Err(CliError::new(err))
            }
        }
    }
}

fn write_json_manifest(path: &str, payload: &serde_json::Value) -> Result<(), CliError> {
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| CliError::new(format!("cannot serialize runtime manifest: {err}")))?;
    let manifest_path = PathBuf::from(path);
    if let Some(parent) = manifest_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                CliError::new(format!(
                    "cannot create runtime manifest directory {}: {err}",
                    parent.display()
                ))
            })?;
        }
    }
    fs::write(&manifest_path, text).map_err(|err| {
        CliError::new(format!(
            "cannot write runtime manifest {}: {err}",
            manifest_path.display()
        ))
    })?;
    Ok(())
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
    if build_args.release_exe && build_args.target == "web" {
        return Err(CliError::new(
            "`--release-exe` only supports `desktop` or `multi` target",
        ));
    }
    let ir = pipeline(&build_args.input)?;
    let report = output::emit_target(
        &ir,
        &build_args.target,
        &build_args.out_dir,
        build_args.prod,
        build_args.strict_parity,
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

    if report.desktop_parity_warning_count > 0 {
        println!(
            "desktop parity warnings: total={} style={} widget={}",
            report.desktop_parity_warning_count,
            report.desktop_style_warning_count,
            report.desktop_widget_warning_count
        );
        if let Some(ref path) = report.desktop_parity_diagnostics_path {
            println!("desktop parity details: {path}");
        }
    }

    if build_args.strict_parity && report.desktop_parity_warning_count > 0 {
        let detail_path = report
            .desktop_parity_diagnostics_path
            .unwrap_or_else(|| "<unknown>".to_string());
        return Err(CliError::new(format!(
            "E7600 strict parity failed: {} desktop parity warning(s) detected (style={}, widget={}); details: {}",
            report.desktop_parity_warning_count,
            report.desktop_style_warning_count,
            report.desktop_widget_warning_count,
            detail_path
        )));
    }

    if build_args.release_exe {
        build_native_release_executable(&build_args.target, &build_args.out_dir)?;
    }

    Ok(())
}

fn build_native_release_executable(target: &str, out_dir: &str) -> Result<(), CliError> {
    match target {
        "desktop" => {
            let native_app_dir = PathBuf::from(out_dir).join("native-app");
            run_cargo_release_build(&native_app_dir)?;
            print_release_binary_hint(&native_app_dir);
            Ok(())
        }
        "multi" => {
            let native_app_dir = PathBuf::from(out_dir).join("desktop").join("native-app");
            run_cargo_release_build(&native_app_dir)?;
            print_release_binary_hint(&native_app_dir);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn run_cargo_release_build(native_app_dir: &PathBuf) -> Result<(), CliError> {
    if !native_app_dir.exists() {
        return Err(CliError::new(format!(
            "native app directory not found: {}",
            native_app_dir.display()
        )));
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(native_app_dir)
        .status()
        .map_err(|err| {
            CliError::new(format!(
                "failed to run `cargo build --release` in {}: {err}",
                native_app_dir.display()
            ))
        })?;

    if !status.success() {
        return Err(CliError::new(format!(
            "`cargo build --release` failed in {}",
            native_app_dir.display()
        )));
    }

    Ok(())
}

fn print_release_binary_hint(native_app_dir: &PathBuf) {
    let release_dir = native_app_dir.join("target").join("release");
    #[cfg(target_os = "windows")]
    {
        println!(
            "native release executable generated under: {} (*.exe)",
            release_dir.display()
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "native release executable generated under: {}",
            release_dir.display()
        );
    }
}
