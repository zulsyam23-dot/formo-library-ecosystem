mod args;
mod benchmark;
mod diagnose;
mod doctor;
mod engine_bridge;
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
use engine_bridge::{
    collect_engine_bridge_report, EngineBridgeReport, LogicActionStepBridge, LogicEventBridge,
    LogicEventScriptBridge, LogicSetExprTokenBridge, LogicSetOperandBridge,
};
use error::CliError;
use json_output::{
    attach_schema_if_enabled, build_error_meta, classify_error_stage, emit_json,
    CHECK_JSON_SCHEMA_ID,
};
use pipeline::pipeline;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
            ensure_input_extension("diagnose", &check_args.input, "fm")?;
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
            ensure_input_extension("fmt", &fmt_args.input, "fm")?;
            fmt_cmd::run_fmt(&fmt_args)
        }
        "lsp" => {
            let lsp_args = parse_lsp_args(&args[2..])?;
            ensure_input_extension("lsp", &lsp_args.input, "fm")?;
            lsp_bridge::run_lsp(&lsp_args)
        }
        "doctor" => {
            let doctor_args = parse_doctor_args(&args[2..])?;
            ensure_input_extension("doctor", &doctor_args.input, "fm")?;
            doctor::run_doctor(&doctor_args)
        }
        "bench" => {
            let bench_args = parse_benchmark_args(&args[2..])?;
            ensure_input_extension("bench", &bench_args.input, "fm")?;
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

fn ensure_input_extension(command: &str, input: &str, expected_ext: &str) -> Result<(), CliError> {
    let has_expected_ext = Path::new(input)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(expected_ext));

    if has_expected_ext {
        Ok(())
    } else {
        Err(CliError::new(format!(
            "`{command}` expects input file with `.{expected_ext}` extension, got `{input}`"
        )))
    }
}

fn run_logic(raw_args: &[String]) -> Result<(), CliError> {
    let logic_args = parse_logic_args(raw_args)?;
    ensure_input_extension("logic", &logic_args.input, "fl")?;
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

fn write_engine_bridge_manifest(
    target: &str,
    out_dir: &str,
    payload: &serde_json::Value,
) -> Result<Vec<String>, CliError> {
    let mut paths = Vec::new();
    let root_path = format!("{out_dir}/engine.bridge.json");
    write_json_manifest(&root_path, payload)?;
    paths.push(root_path);

    if target == "multi" {
        for suffix in ["web/engine.bridge.json", "desktop/engine.bridge.json"] {
            let path = format!("{out_dir}/{suffix}");
            write_json_manifest(&path, payload)?;
            paths.push(path);
        }
    }

    Ok(paths)
}

fn sync_desktop_actions_with_logic(
    target: &str,
    out_dir: &str,
    bridge_report: &EngineBridgeReport,
) -> Result<(), CliError> {
    if bridge_report.logic_events.is_empty() {
        return Ok(());
    }

    let action_paths = match target {
        "desktop" => vec![PathBuf::from(out_dir)
            .join("native-app")
            .join("src")
            .join("actions.rs")],
        "multi" => vec![PathBuf::from(out_dir)
            .join("desktop")
            .join("native-app")
            .join("src")
            .join("actions.rs")],
        _ => Vec::new(),
    };

    for path in action_paths {
        if !path.exists() {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(|err| {
            CliError::new(format!(
                "cannot read generated desktop actions file {}: {err}",
                path.display()
            ))
        })?;

        let next = augment_actions_registry_with_logic(
            &raw,
            &bridge_report.logic_events,
            &bridge_report.logic_event_scripts,
        );
        if next == raw {
            continue;
        }

        fs::write(&path, next).map_err(|err| {
            CliError::new(format!(
                "cannot write generated desktop actions file {}: {err}",
                path.display()
            ))
        })?;
    }

    Ok(())
}

fn sync_web_runtime_with_logic(
    target: &str,
    out_dir: &str,
    bridge_report: &EngineBridgeReport,
) -> Result<(), CliError> {
    if bridge_report.logic_events.is_empty() {
        return Ok(());
    }

    let runtime_paths = match target {
        "web" => vec![
            PathBuf::from(out_dir).join("app.js"),
            PathBuf::from(out_dir)
                .join("runtime")
                .join("app")
                .join("50_actions_state.js"),
        ],
        "multi" => vec![
            PathBuf::from(out_dir).join("web").join("app.js"),
            PathBuf::from(out_dir)
                .join("web")
                .join("runtime")
                .join("app")
                .join("50_actions_state.js"),
        ],
        _ => Vec::new(),
    };

    for path in runtime_paths {
        if !path.exists() {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(|err| {
            CliError::new(format!(
                "cannot read generated web runtime file {}: {err}",
                path.display()
            ))
        })?;

        let next = augment_web_runtime_with_logic(
            &raw,
            &bridge_report.logic_events,
            &bridge_report.logic_event_scripts,
        );
        if next == raw {
            continue;
        }

        fs::write(&path, next).map_err(|err| {
            CliError::new(format!(
                "cannot write generated web runtime file {}: {err}",
                path.display()
            ))
        })?;
    }

    Ok(())
}

fn augment_web_runtime_with_logic(
    raw: &str,
    logic_events: &[LogicEventBridge],
    logic_event_scripts: &[LogicEventScriptBridge],
) -> String {
    const DISPATCH_SIGNATURE: &str = "function dispatchAction(name, payload, node, scope) {";
    const ACTION_LOOKUP_LINE: &str = "const action = actionHandlers[actionName];";

    if logic_event_scripts.is_empty() || !raw.contains(DISPATCH_SIGNATURE) {
        return raw.to_string();
    }

    let mut out = raw.to_string();
    if !out.contains("const formoGeneratedActions = {") {
        let runtime_block = render_web_action_runtime_block(logic_events, logic_event_scripts);
        let dispatch_with_block = format!("{runtime_block}\n\n  {DISPATCH_SIGNATURE}");
        out = out.replacen(DISPATCH_SIGNATURE, &dispatch_with_block, 1);
    }

    if out.contains(ACTION_LOOKUP_LINE) {
        out = out.replacen(
            ACTION_LOOKUP_LINE,
            "const action = resolveActionHandler(actionName);",
            1,
        );
    }

    out
}

fn render_web_action_runtime_block(
    logic_events: &[LogicEventBridge],
    logic_event_scripts: &[LogicEventScriptBridge],
) -> String {
    let mut lines = Vec::new();
    lines.extend(render_web_generated_action_handlers(
        logic_events,
        logic_event_scripts,
    ));
    lines.push(String::new());
    lines.push("  function resolveActionHandler(actionName) {".to_string());
    lines.push(
        "    if (Object.prototype.hasOwnProperty.call(formoGeneratedActions, actionName)) {"
            .to_string(),
    );
    lines.push("      return formoGeneratedActions[actionName];".to_string());
    lines.push("    }".to_string());
    lines.push("    return actionHandlers[actionName];".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function evalSetExpressionRpn(tokens) {".to_string());
    lines.push("    if (!Array.isArray(tokens) || tokens.length === 0) {".to_string());
    lines.push("      return undefined;".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push("    const stack = [];".to_string());
    lines.push("    for (const token of tokens) {".to_string());
    lines.push("      if (!Array.isArray(token) || token.length < 2) {".to_string());
    lines.push("        return undefined;".to_string());
    lines.push("      }".to_string());
    lines.push(String::new());
    lines.push("      const kind = token[0];".to_string());
    lines.push("      const raw = token[1];".to_string());
    lines.push("      if (kind === \"operator\") {".to_string());
    lines.push("        if (stack.length < 2) {".to_string());
    lines.push("          return undefined;".to_string());
    lines.push("        }".to_string());
    lines.push("        const rhs = stack.pop();".to_string());
    lines.push("        const lhs = stack.pop();".to_string());
    lines.push("        const next = applySetBinaryOperator(lhs, raw, rhs);".to_string());
    lines.push("        if (typeof next === \"undefined\") {".to_string());
    lines.push("          return undefined;".to_string());
    lines.push("        }".to_string());
    lines.push("        stack.push(next);".to_string());
    lines.push("        continue;".to_string());
    lines.push("      }".to_string());
    lines.push(String::new());
    lines.push("      const value = resolveSetOperandValue(kind, raw);".to_string());
    lines.push("      if (typeof value === \"undefined\") {".to_string());
    lines.push("        return undefined;".to_string());
    lines.push("      }".to_string());
    lines.push("      stack.push(value);".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push("    return stack.length === 1 ? stack[0] : undefined;".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function resolveSetOperandValue(kind, raw) {".to_string());
    lines.push("    if (kind === \"stateRef\") {".to_string());
    lines.push(
        "      return Object.prototype.hasOwnProperty.call(stateStore, raw) ? stateStore[raw] : undefined;"
            .to_string(),
    );
    lines.push("    }".to_string());
    lines.push("    if (kind === \"boolLiteral\") {".to_string());
    lines.push("      return String(raw).toLowerCase() === \"true\";".to_string());
    lines.push("    }".to_string());
    lines.push("    if (kind === \"stringLiteral\") {".to_string());
    lines.push("      return String(raw);".to_string());
    lines.push("    }".to_string());
    lines.push("    if (kind === \"intLiteral\") {".to_string());
    lines.push("      const parsed = Number.parseInt(String(raw), 10);".to_string());
    lines.push("      return Number.isNaN(parsed) ? undefined : parsed;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (kind === \"floatLiteral\") {".to_string());
    lines.push("      const parsed = Number.parseFloat(String(raw));".to_string());
    lines.push("      return Number.isNaN(parsed) ? undefined : parsed;".to_string());
    lines.push("    }".to_string());
    lines.push("    return undefined;".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function applySetBinaryOperator(lhs, op, rhs) {".to_string());
    lines.push("    if (op === \"add\") {".to_string());
    lines.push("      if (typeof lhs === \"string\" || typeof rhs === \"string\") {".to_string());
    lines.push("        return `${setStringify(lhs)}${setStringify(rhs)}`;".to_string());
    lines.push("      }".to_string());
    lines.push("      const left = setToNumber(lhs);".to_string());
    lines.push("      const right = setToNumber(rhs);".to_string());
    lines.push(
        "      return typeof left === \"undefined\" || typeof right === \"undefined\"".to_string(),
    );
    lines.push("        ? undefined".to_string());
    lines.push("        : left + right;".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push(
        "    if (op === \"sub\" || op === \"mul\" || op === \"div\" || op === \"mod\") {"
            .to_string(),
    );
    lines.push("      const left = setToNumber(lhs);".to_string());
    lines.push("      const right = setToNumber(rhs);".to_string());
    lines.push(
        "      if (typeof left === \"undefined\" || typeof right === \"undefined\") {".to_string(),
    );
    lines.push("        return undefined;".to_string());
    lines.push("      }".to_string());
    lines.push(
        "      if ((op === \"div\" || op === \"mod\") && Math.abs(right) < Number.EPSILON) {"
            .to_string(),
    );
    lines.push("        return undefined;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"sub\") {".to_string());
    lines.push("        return left - right;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"mul\") {".to_string());
    lines.push("        return left * right;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"div\") {".to_string());
    lines.push("        return left / right;".to_string());
    lines.push("      }".to_string());
    lines.push("      return left % right;".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push("    if (op === \"eq\") {".to_string());
    lines.push("      return setLooseEq(lhs, rhs);".to_string());
    lines.push("    }".to_string());
    lines.push("    if (op === \"notEq\") {".to_string());
    lines.push("      return !setLooseEq(lhs, rhs);".to_string());
    lines.push("    }".to_string());
    lines.push(
        "    if (op === \"lt\" || op === \"ltEq\" || op === \"gt\" || op === \"gtEq\") {"
            .to_string(),
    );
    lines.push("      const left = setToNumber(lhs);".to_string());
    lines.push("      const right = setToNumber(rhs);".to_string());
    lines.push(
        "      if (typeof left === \"undefined\" || typeof right === \"undefined\") {".to_string(),
    );
    lines.push("        return undefined;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"lt\") {".to_string());
    lines.push("        return left < right;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"ltEq\") {".to_string());
    lines.push("        return left <= right;".to_string());
    lines.push("      }".to_string());
    lines.push("      if (op === \"gt\") {".to_string());
    lines.push("        return left > right;".to_string());
    lines.push("      }".to_string());
    lines.push("      return left >= right;".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push("    if (op === \"and\") {".to_string());
    lines.push("      return setTruthy(lhs) && setTruthy(rhs);".to_string());
    lines.push("    }".to_string());
    lines.push("    if (op === \"or\") {".to_string());
    lines.push("      return setTruthy(lhs) || setTruthy(rhs);".to_string());
    lines.push("    }".to_string());
    lines.push(String::new());
    lines.push("    return undefined;".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function setToNumber(value) {".to_string());
    lines.push("    if (typeof value === \"number\") {".to_string());
    lines.push("      return Number.isNaN(value) ? undefined : value;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"boolean\") {".to_string());
    lines.push("      return value ? 1 : 0;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"string\") {".to_string());
    lines.push("      const parsed = Number.parseFloat(value.trim());".to_string());
    lines.push("      return Number.isNaN(parsed) ? undefined : parsed;".to_string());
    lines.push("    }".to_string());
    lines.push("    return undefined;".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function setTruthy(value) {".to_string());
    lines.push("    if (typeof value === \"boolean\") {".to_string());
    lines.push("      return value;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"number\") {".to_string());
    lines.push("      return Math.abs(value) > Number.EPSILON;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"string\") {".to_string());
    lines.push("      const lowered = value.trim().toLowerCase();".to_string());
    lines.push(
        "      return lowered.length > 0 && lowered !== \"false\" && lowered !== \"0\";"
            .to_string(),
    );
    lines.push("    }".to_string());
    lines.push("    if (Array.isArray(value)) {".to_string());
    lines.push("      return value.length > 0;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (value && typeof value === \"object\") {".to_string());
    lines.push("      return Object.keys(value).length > 0;".to_string());
    lines.push("    }".to_string());
    lines.push("    return false;".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function setLooseEq(lhs, rhs) {".to_string());
    lines.push("    const left = setToNumber(lhs);".to_string());
    lines.push("    const right = setToNumber(rhs);".to_string());
    lines.push(
        "    if (typeof left !== \"undefined\" && typeof right !== \"undefined\") {".to_string(),
    );
    lines.push("      return Math.abs(left - right) < Number.EPSILON;".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof lhs === \"boolean\" && typeof rhs === \"boolean\") {".to_string());
    lines.push("      return lhs === rhs;".to_string());
    lines.push("    }".to_string());
    lines.push("    return setStringify(lhs) === setStringify(rhs);".to_string());
    lines.push("  }".to_string());
    lines.push(String::new());
    lines.push("  function setStringify(value) {".to_string());
    lines.push("    if (value === null) {".to_string());
    lines.push("      return \"null\";".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"boolean\") {".to_string());
    lines.push("      return value ? \"true\" : \"false\";".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"number\") {".to_string());
    lines.push("      return String(value);".to_string());
    lines.push("    }".to_string());
    lines.push("    if (typeof value === \"string\") {".to_string());
    lines.push("      return value;".to_string());
    lines.push("    }".to_string());
    lines.push("    try {".to_string());
    lines.push("      return JSON.stringify(value);".to_string());
    lines.push("    } catch (_err) {".to_string());
    lines.push("      return \"<json>\";".to_string());
    lines.push("    }".to_string());
    lines.push("  }".to_string());

    lines.join("\n")
}

fn render_web_generated_action_handlers(
    logic_events: &[LogicEventBridge],
    logic_event_scripts: &[LogicEventScriptBridge],
) -> Vec<String> {
    let mut summary_by_name = BTreeMap::new();
    for event in logic_events {
        summary_by_name.insert(event.name.clone(), event);
    }

    let mut script_by_name = BTreeMap::new();
    for script in logic_event_scripts {
        script_by_name.entry(script.name.clone()).or_insert(script);
    }

    let mut lines = vec!["  const formoGeneratedActions = {".to_string()];
    for (event_name, script) in script_by_name {
        lines.push(format!(
            "    {}: function(event) {{",
            js_string_literal(&event_name)
        ));
        lines.push("      const _event = event || {};".to_string());
        if let Some(summary) = summary_by_name.get(&event_name).copied() {
            lines.push(format!(
                "      // FL contract: totalActions={} set={} emit={} globalCalls={} webCalls={} desktopCalls={}",
                summary.total_actions,
                summary.set_count,
                summary.emit_count,
                join_calls_for_comment(&summary.global_calls),
                join_calls_for_comment(&summary.web_calls),
                join_calls_for_comment(&summary.desktop_calls),
            ));
        }

        let mut generated_step_count = 0usize;
        for step in &script.steps {
            let step_lines = render_web_logic_step(&event_name, step);
            if step_lines.is_empty() {
                continue;
            }
            generated_step_count += 1;
            lines.extend(step_lines);
        }
        if generated_step_count == 0 {
            lines.push("      // No executable global FL actions mapped yet.".to_string());
        }
        lines.push("    },".to_string());
    }
    lines.push("  };".to_string());

    lines
}

fn render_web_logic_step(event_name: &str, step: &LogicActionStepBridge) -> Vec<String> {
    let mut out = Vec::new();
    let scope = step.scope.as_str();
    let kind = step.kind.as_str();

    if scope != "global" {
        let msg = format!(
            "[formo] skip {} step in scope `{}` for FL event `{}`",
            kind, scope, event_name
        );
        out.push(format!("      console.info({});", js_string_literal(&msg)));
        return out;
    }

    match kind {
        "set" => {
            if let Some(target) = step.target.as_ref() {
                let precise_set = render_web_precise_set_assignment(step, target);
                if precise_set.is_empty() {
                    out.push(format!(
                        "      // FL step: set state `{}` from event payload when payload is non-null.",
                        target
                    ));
                    out.push(
                        "      if (!(_event.payload === null || typeof _event.payload === \"undefined\")) {"
                            .to_string(),
                    );
                    out.push(format!(
                        "        writeState({}, _event.payload);",
                        js_string_literal(target),
                    ));
                    out.push("      }".to_string());
                } else {
                    out.extend(precise_set);
                }
            } else {
                let msg = format!("[formo] skip malformed `set` step in FL event `{event_name}`");
                out.push(format!("      console.info({});", js_string_literal(&msg)));
            }
        }
        "call" => {
            let target = step.target.as_deref().unwrap_or("<unknown>");
            let msg = format!(
                "[formo] global call `{}` from FL event `{}`",
                target, event_name
            );
            out.push(format!("      console.info({});", js_string_literal(&msg)));
        }
        "emit" => {
            let msg = format!("[formo] emit from FL event `{}`", event_name);
            out.push(format!("      console.info({});", js_string_literal(&msg)));
        }
        "throw" | "break" | "continue" | "return" => {
            let msg = format!(
                "[formo] control-flow action `{}` from FL event `{}`",
                kind, event_name
            );
            out.push(format!("      console.info({});", js_string_literal(&msg)));
        }
        _ => {
            let msg = format!(
                "[formo] unsupported FL action `{}` for event `{}`",
                kind, event_name
            );
            out.push(format!("      console.info({});", js_string_literal(&msg)));
        }
    }

    out
}

fn render_web_precise_set_assignment(step: &LogicActionStepBridge, target: &str) -> Vec<String> {
    let set_value_hint = step.set_value_hint.as_deref().unwrap_or("expression");

    if step.set_operands.is_empty() {
        return Vec::new();
    }
    if !step.set_operators.is_empty() || step.set_operands.len() > 1 {
        return render_web_expression_set_assignment(step, target, set_value_hint);
    }

    let operand = &step.set_operands[0];
    match operand.kind.as_str() {
        "stateRef" => {
            let Some(source_key) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!("      // FL step: set from state ref `{}`.", source_key),
                format!(
                    "      if (Object.prototype.hasOwnProperty.call(stateStore, {})) {{",
                    js_string_literal(source_key),
                ),
                format!(
                    "        writeState({}, stateStore[{}]);",
                    js_string_literal(target),
                    js_string_literal(source_key),
                ),
                "      }".to_string(),
            ]
        }
        "boolLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![format!(
                "      writeState({}, {});",
                js_string_literal(target),
                if value.eq_ignore_ascii_case("true") {
                    "true"
                } else {
                    "false"
                }
            )]
        }
        "stringLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![format!(
                "      writeState({}, {});",
                js_string_literal(target),
                js_string_literal(value)
            )]
        }
        "intLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!(
                    "      const parsed = Number.parseInt({}, 10);",
                    js_string_literal(value),
                ),
                "      if (!Number.isNaN(parsed)) {".to_string(),
                format!("        writeState({}, parsed);", js_string_literal(target)),
                "      }".to_string(),
            ]
        }
        "floatLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!(
                    "      const parsed = Number.parseFloat({});",
                    js_string_literal(value),
                ),
                "      if (!Number.isNaN(parsed)) {".to_string(),
                format!("        writeState({}, parsed);", js_string_literal(target)),
                "      }".to_string(),
            ]
        }
        _ => Vec::new(),
    }
}

fn render_web_expression_set_assignment(
    step: &LogicActionStepBridge,
    target: &str,
    set_value_hint: &str,
) -> Vec<String> {
    let expression_rpn_lit = render_web_expression_rpn_literal(&step.set_expression_rpn);
    if expression_rpn_lit.is_empty() {
        return Vec::new();
    }

    vec![
        format!(
            "      // FL step: evaluate expression set (hint={}).",
            set_value_hint
        ),
        format!(
            "      const nextValue = evalSetExpressionRpn({});",
            expression_rpn_lit
        ),
        "      if (typeof nextValue !== \"undefined\") {".to_string(),
        format!(
            "        writeState({}, nextValue);",
            js_string_literal(target)
        ),
        "      }".to_string(),
    ]
}

fn render_web_expression_rpn_literal(tokens: &[LogicSetExprTokenBridge]) -> String {
    if tokens.is_empty() {
        return "[]".to_string();
    }

    let tuples = tokens
        .iter()
        .map(|token| {
            let kind = js_string_literal(&token.kind);
            let value = js_string_literal(token.value.as_deref().unwrap_or(""));
            format!("[{}, {}]", kind, value)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{tuples}]")
}

fn augment_actions_registry_with_logic(
    raw: &str,
    logic_events: &[LogicEventBridge],
    logic_event_scripts: &[LogicEventScriptBridge],
) -> String {
    const DEFAULT_MATCH_ARM: &str = "        _ => return Ok(false),";

    if logic_event_scripts.is_empty() || !raw.contains(DEFAULT_MATCH_ARM) {
        return raw.to_string();
    }

    let mut summary_by_name = BTreeMap::new();
    for event in logic_events {
        summary_by_name.insert(event.name.clone(), event);
    }

    let mut script_by_name = BTreeMap::new();
    for script in logic_event_scripts {
        script_by_name.entry(script.name.clone()).or_insert(script);
    }

    let mut mappings = collect_registered_action_mappings(raw);
    let mut existing_events = mappings
        .iter()
        .map(|(event, _)| event.clone())
        .collect::<BTreeSet<_>>();
    let mut used_handlers = collect_registered_handler_names(raw);
    let mut extra_arms = String::new();

    for event_name in script_by_name.keys() {
        if existing_events.contains(event_name) {
            continue;
        }

        let handler = unique_handler_name(event_name, &mut used_handlers);
        extra_arms.push_str(&format!("        {:?} => {},\n", event_name, handler));
        mappings.push((event_name.clone(), handler));
        existing_events.insert(event_name.clone());
    }

    let mut out = if extra_arms.is_empty() {
        raw.to_string()
    } else {
        raw.replacen(
            DEFAULT_MATCH_ARM,
            &format!("{extra_arms}{DEFAULT_MATCH_ARM}"),
            1,
        )
    };

    for (event_name, handler_name) in mappings {
        let Some(script) = script_by_name.get(&event_name) else {
            continue;
        };
        let rendered = render_logic_handler(
            &handler_name,
            &event_name,
            script,
            summary_by_name.get(&event_name).copied(),
        );
        out = replace_or_append_handler_block(out, &handler_name, &rendered);
    }

    out
}

fn collect_registered_action_mappings(raw: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let Some((lhs, rhs)) = line.split_once("=>") else {
            continue;
        };
        let action_name = lhs.trim().trim_matches('"');
        let handler_name = rhs.trim().trim_end_matches(',');
        if action_name.is_empty() || handler_name.is_empty() {
            continue;
        }
        if !handler_name.starts_with("handle_") {
            continue;
        }
        out.push((action_name.to_string(), handler_name.to_string()));
    }
    out
}

fn collect_registered_handler_names(raw: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in raw.lines() {
        if let Some((_, rhs)) = line.split_once("=>") {
            let candidate = rhs.trim().trim_end_matches(',');
            if candidate.starts_with("handle_") {
                out.insert(candidate.to_string());
            }
        }

        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            let name = rest.split('(').next().unwrap_or_default().trim();
            if !name.is_empty() {
                out.insert(name.to_string());
            }
        }
    }
    out
}

fn unique_handler_name(event_name: &str, used_handlers: &mut BTreeSet<String>) -> String {
    let mut base = to_snake_case(event_name);
    if base.is_empty() {
        base = "action".to_string();
    }
    if base
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        base = format!("action_{base}");
    }

    let mut candidate = format!("handle_{base}");
    let mut suffix = 2usize;
    while !used_handlers.insert(candidate.clone()) {
        candidate = format!("handle_{base}_{suffix}");
        suffix += 1;
    }
    candidate
}

fn replace_or_append_handler_block(raw: String, handler_name: &str, replacement: &str) -> String {
    if let Some((start, end)) = find_handler_block(&raw, handler_name) {
        let mut out = raw;
        out.replace_range(start..end, replacement);
        return out;
    }

    let mut out = raw.trim_end().to_string();
    out.push_str("\n\n");
    out.push_str(replacement.trim_end());
    out.push('\n');
    out
}

fn find_handler_block(raw: &str, handler_name: &str) -> Option<(usize, usize)> {
    let signature = format!(
        "fn {}(event: ActionEvent, state_store: Signal<NativeState>) {{",
        handler_name
    );
    let start = raw.find(&signature)?;
    let mut index = start + signature.len();
    let bytes = raw.as_bytes();
    let mut depth = 1usize;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((start, index + 1));
                }
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn render_logic_handler(
    handler_name: &str,
    event_name: &str,
    script: &LogicEventScriptBridge,
    summary: Option<&LogicEventBridge>,
) -> String {
    let mut lines = vec![
        format!(
            "fn {}(event: ActionEvent, state_store: Signal<NativeState>) {{",
            handler_name
        ),
        "    let _ = (&event, &state_store);".to_string(),
    ];

    if let Some(summary) = summary {
        lines.push(format!(
            "    // FL contract: totalActions={} set={} emit={} globalCalls={} webCalls={} desktopCalls={}",
            summary.total_actions,
            summary.set_count,
            summary.emit_count,
            join_calls_for_comment(&summary.global_calls),
            join_calls_for_comment(&summary.web_calls),
            join_calls_for_comment(&summary.desktop_calls),
        ));
    }

    let mut generated_step_count = 0usize;
    for step in &script.steps {
        let step_lines = render_logic_step(event_name, step);
        if step_lines.is_empty() {
            continue;
        }
        generated_step_count += 1;
        lines.extend(step_lines);
    }

    if generated_step_count == 0 {
        lines.push("    // No executable global FL actions mapped yet.".to_string());
    }

    lines.push("}".to_string());
    lines.join("\n")
}

fn render_logic_step(event_name: &str, step: &LogicActionStepBridge) -> Vec<String> {
    let mut out = Vec::new();
    let scope = step.scope.as_str();
    let kind = step.kind.as_str();

    if scope != "global" {
        let msg = format!(
            "[formo] skip {} step in scope `{}` for FL event `{}`",
            kind, scope, event_name
        );
        out.push(format!("    println!({});", rust_string_literal(&msg)));
        return out;
    }

    match kind {
        "set" => {
            if let Some(target) = step.target.as_ref() {
                let target_lit = rust_string_literal(target);
                let precise_set = render_precise_set_assignment(step, target_lit.as_str());
                if precise_set.is_empty() {
                    out.push(format!(
                        "    // FL step: set state `{}` from event payload when payload is non-null.",
                        target
                    ));
                    out.push("    if !event.payload.is_null() {".to_string());
                    out.push(format!(
                        "        set_state(state_store.clone(), {}, event.payload.clone());",
                        target_lit,
                    ));
                    out.push("    }".to_string());
                } else {
                    out.extend(precise_set);
                }
            } else {
                let msg = format!("[formo] skip malformed `set` step in FL event `{event_name}`");
                out.push(format!("    println!({});", rust_string_literal(&msg)));
            }
        }
        "call" => {
            let target = step.target.as_deref().unwrap_or("<unknown>");
            let msg = format!(
                "[formo] global call `{}` from FL event `{}`",
                target, event_name
            );
            out.push(format!("    println!({});", rust_string_literal(&msg)));
        }
        "emit" => {
            let msg = format!("[formo] emit from FL event `{}`", event_name);
            out.push(format!("    println!({});", rust_string_literal(&msg)));
        }
        "throw" | "break" | "continue" | "return" => {
            let msg = format!(
                "[formo] control-flow action `{}` from FL event `{}`",
                kind, event_name
            );
            out.push(format!("    println!({});", rust_string_literal(&msg)));
        }
        _ => {
            let msg = format!(
                "[formo] unsupported FL action `{}` for event `{}`",
                kind, event_name
            );
            out.push(format!("    println!({});", rust_string_literal(&msg)));
        }
    }

    out
}

fn render_precise_set_assignment(step: &LogicActionStepBridge, target_lit: &str) -> Vec<String> {
    let set_value_hint = step.set_value_hint.as_deref().unwrap_or("expression");

    if step.set_operands.is_empty() {
        return Vec::new();
    }
    if !step.set_operators.is_empty() || step.set_operands.len() > 1 {
        return render_expression_set_assignment(step, target_lit, set_value_hint);
    }

    let operand = &step.set_operands[0];
    match operand.kind.as_str() {
        "stateRef" => {
            let Some(source_key) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!("    // FL step: set from state ref `{}`.", source_key),
                format!(
                    "    if let Some(next_value) = state_store.read().get({}).cloned() {{",
                    rust_string_literal(source_key),
                ),
                format!(
                    "        set_state(state_store.clone(), {}, next_value);",
                    target_lit
                ),
                "    }".to_string(),
            ]
        }
        "boolLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![format!(
                "    set_state(state_store.clone(), {}, JsonValue::Bool({}));",
                target_lit,
                if value.eq_ignore_ascii_case("true") {
                    "true"
                } else {
                    "false"
                }
            )]
        }
        "stringLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![format!(
                "    set_state(state_store.clone(), {}, JsonValue::String({}.to_string()));",
                target_lit,
                rust_string_literal(value)
            )]
        }
        "intLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!(
                    "    if let Ok(parsed) = {}.parse::<i64>() {{",
                    rust_string_literal(value),
                ),
                format!(
                    "        set_state(state_store.clone(), {}, JsonValue::from(parsed));",
                    target_lit
                ),
                "    }".to_string(),
            ]
        }
        "floatLiteral" => {
            let Some(value) = operand.value.as_ref() else {
                return Vec::new();
            };
            vec![
                format!(
                    "    if let Ok(parsed) = {}.parse::<f64>() {{",
                    rust_string_literal(value),
                ),
                format!(
                    "        set_state(state_store.clone(), {}, JsonValue::from(parsed));",
                    target_lit
                ),
                "    }".to_string(),
            ]
        }
        _ => Vec::new(),
    }
}

fn render_expression_set_assignment(
    step: &LogicActionStepBridge,
    target_lit: &str,
    set_value_hint: &str,
) -> Vec<String> {
    let expression_rpn_lit = render_expression_rpn_literal(&step.set_expression_rpn);
    if !expression_rpn_lit.is_empty() {
        return vec![
            format!(
                "    // FL step: evaluate expression set (hint={}).",
                set_value_hint
            ),
            format!(
                "    if let Some(next_value) = eval_set_expression_rpn(state_store.clone(), {}) {{",
                expression_rpn_lit
            ),
            format!(
                "        set_state(state_store.clone(), {}, next_value);",
                target_lit
            ),
            "    }".to_string(),
        ];
    }

    let operands_lit = render_operand_pairs_literal(&step.set_operands);
    if operands_lit.is_empty() {
        return Vec::new();
    }
    let operators_lit = render_operator_list_literal(&step.set_operators);

    vec![
        format!(
            "    // FL step: evaluate expression set (hint={}).",
            set_value_hint
        ),
        format!(
            "    if let Some(next_value) = eval_set_expression(state_store.clone(), {}, {}) {{",
            operands_lit, operators_lit
        ),
        format!(
            "        set_state(state_store.clone(), {}, next_value);",
            target_lit
        ),
        "    }".to_string(),
    ]
}

fn render_operand_pairs_literal(operands: &[LogicSetOperandBridge]) -> String {
    if operands.is_empty() {
        return String::new();
    }

    let tuples = operands
        .iter()
        .map(|operand| {
            let kind = rust_string_literal(&operand.kind);
            let value = rust_string_literal(operand.value.as_deref().unwrap_or(""));
            format!("({}, {})", kind, value)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{tuples}]")
}

fn render_operator_list_literal(operators: &[String]) -> String {
    if operators.is_empty() {
        return "&[]".to_string();
    }

    let items = operators
        .iter()
        .map(|operator| rust_string_literal(operator))
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{items}]")
}

fn render_expression_rpn_literal(tokens: &[LogicSetExprTokenBridge]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let tuples = tokens
        .iter()
        .map(|token| {
            let kind = rust_string_literal(&token.kind);
            let value = rust_string_literal(token.value.as_deref().unwrap_or(""));
            format!("({}, {})", kind, value)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{tuples}]")
}

fn rust_string_literal(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "\"\"".to_string())
}

fn js_string_literal(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "\"\"".to_string())
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    let mut prev_is_sep = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !out.is_empty() && !prev_is_sep {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_is_sep = false;
        } else if !out.is_empty() && !prev_is_sep {
            out.push('_');
            prev_is_sep = true;
        }
    }

    out.trim_matches('_').to_string()
}

fn join_calls_for_comment(calls: &[String]) -> String {
    if calls.is_empty() {
        "<none>".to_string()
    } else {
        calls.join("|")
    }
}

fn run_check(raw_args: &[String]) -> Result<(), CliError> {
    let check_args = parse_check_args(raw_args)?;
    ensure_input_extension("check", &check_args.input, "fm")?;
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
    ensure_input_extension("build", &build_args.input, "fm")?;
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
    let bridge_report = collect_engine_bridge_report(&build_args.input, &ir);
    sync_desktop_actions_with_logic(&build_args.target, &build_args.out_dir, &bridge_report)?;
    sync_web_runtime_with_logic(&build_args.target, &build_args.out_dir, &bridge_report)?;
    let bridge_paths = write_engine_bridge_manifest(
        &build_args.target,
        &build_args.out_dir,
        &bridge_report.manifest,
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
    println!(
        "engine bridge: profile=fm-fs-fl canonical-style={}/{} logic-status={} warnings={}",
        bridge_report.canonical_style_count,
        bridge_report.style_count,
        bridge_report.logic_status,
        bridge_report.warning_count
    );
    if let Some(ref logic_input) = bridge_report.logic_input {
        println!("engine logic input: {logic_input}");
    }
    if let Some(path) = bridge_paths.first() {
        println!("engine bridge manifest: {path}");
    }
    for diag in &bridge_report.diagnostics {
        if let Some(file) = &diag.file {
            println!("engine warning {}: {} ({})", diag.code, diag.message, file);
        } else {
            println!("engine warning {}: {}", diag.code, diag.message);
        }
    }
    if !bridge_report.missing_logic_event_bindings.is_empty() {
        println!(
            "engine action binding gaps: {}",
            bridge_report.missing_logic_event_bindings.join(", ")
        );
    }

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

    if build_args.strict_engine && bridge_report.warning_count > 0 {
        return Err(CliError::new(format!(
            "E7700 strict engine failed: {} warning(s) found in FM/FS/FL bridge profile",
            bridge_report.warning_count
        )));
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
