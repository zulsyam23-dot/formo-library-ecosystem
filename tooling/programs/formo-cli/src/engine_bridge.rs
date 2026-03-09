use formo_ir::IrProgram;
use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const ENGINE_PROFILE_ID: &str = "fm-fs-fl-bridge";
const ENGINE_PROFILE_VERSION: &str = "2026.03";

#[derive(Debug, Clone)]
pub struct EngineBridgeDiagnostic {
    pub code: String,
    pub message: String,
    pub file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogicEventBridge {
    pub name: String,
    pub total_actions: usize,
    pub set_count: usize,
    pub emit_count: usize,
    pub global_calls: Vec<String>,
    pub web_calls: Vec<String>,
    pub desktop_calls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LogicActionStepBridge {
    pub kind: String,
    pub scope: String,
    pub target: Option<String>,
    pub set_value_hint: Option<String>,
    pub set_operands: Vec<LogicSetOperandBridge>,
    pub set_operators: Vec<String>,
    pub set_expression_rpn: Vec<LogicSetExprTokenBridge>,
}

#[derive(Debug, Clone)]
pub struct LogicSetOperandBridge {
    pub kind: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogicSetExprTokenBridge {
    pub kind: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogicEventScriptBridge {
    pub name: String,
    pub steps: Vec<LogicActionStepBridge>,
}

#[derive(Debug, Clone)]
pub struct EngineBridgeReport {
    pub style_count: usize,
    pub canonical_style_count: usize,
    pub logic_status: String,
    pub logic_input: Option<String>,
    pub logic_events: Vec<LogicEventBridge>,
    pub logic_event_scripts: Vec<LogicEventScriptBridge>,
    pub missing_logic_event_bindings: Vec<String>,
    pub warning_count: usize,
    pub diagnostics: Vec<EngineBridgeDiagnostic>,
    pub manifest: JsonValue,
}

pub fn collect_engine_bridge_report(input_fm: &str, ir: &IrProgram) -> EngineBridgeReport {
    let style_count = ir.styles.len();
    let ir_action_bindings = collect_ir_action_bindings(ir);
    let canonical_style_count = ir
        .styles
        .iter()
        .filter(|style| !style.canonical_decls.is_empty())
        .count();
    let canonical_style_coverage_pct = if style_count == 0 {
        100.0
    } else {
        (canonical_style_count as f64 / style_count as f64) * 100.0
    };

    let mut diagnostics = Vec::new();
    if style_count > 0 && canonical_style_count < style_count {
        diagnostics.push(EngineBridgeDiagnostic {
            code: "W7704".to_string(),
            message: format!(
                "style canonical coverage incomplete: {}/{} style block(s) contain canonicalDecls",
                canonical_style_count, style_count
            ),
            file: Some(normalize_path(input_fm)),
        });
    }

    let mut logic_status = "missing".to_string();
    let mut logic_input: Option<String> = None;
    let mut logic_events: Vec<LogicEventBridge> = Vec::new();
    let mut logic_event_scripts: Vec<LogicEventScriptBridge> = Vec::new();
    let mut missing_logic_event_bindings: Vec<String> = Vec::new();
    let mut logic_summary = json!({
        "status": "missing",
        "input": null,
        "module": null,
        "unitCount": 0,
        "parityReadyUnits": 0,
        "parityScore": 0.0,
        "eventCount": 0,
        "symmetricEventCount": 0,
        "eventParityScore": 0.0,
        "nonParityUnits": [],
        "eventNames": [],
        "fmActionBindingCount": ir_action_bindings.len(),
        "missingEventBindings": [],
    });

    match discover_logic_file(input_fm) {
        None => {
            diagnostics.push(EngineBridgeDiagnostic {
                code: "W7701".to_string(),
                message: "logic bridge file `.fl` not found (expected under `logic/` or alongside FM entry)".to_string(),
                file: Some(normalize_path(input_fm)),
            });
        }
        Some(path) => {
            let path_display = normalize_path(path.as_path());
            logic_input = Some(path_display.clone());
            match fs::read_to_string(&path) {
                Ok(source) => match formo_logic::parse(&source) {
                    Ok(program) => {
                        let contract = formo_logic::runtime_contract(&program);
                        logic_events = collect_logic_event_bridges(&contract);
                        logic_event_scripts = collect_logic_event_scripts(&program);
                        let logic_event_names = logic_events
                            .iter()
                            .map(|event| event.name.clone())
                            .collect::<BTreeSet<_>>();
                        missing_logic_event_bindings = ir_action_bindings
                            .iter()
                            .filter(|name| !logic_event_names.contains(*name))
                            .cloned()
                            .collect::<Vec<_>>();

                        let unit_count = contract.units.len();
                        let parity_ready_units = contract
                            .units
                            .iter()
                            .filter(|unit| unit.parity_ready)
                            .count();
                        let parity_score = if unit_count == 0 {
                            100.0
                        } else {
                            (parity_ready_units as f64 / unit_count as f64) * 100.0
                        };

                        let mut event_count = 0usize;
                        let mut symmetric_event_count = 0usize;
                        for unit in &contract.units {
                            for event in &unit.events {
                                event_count += 1;
                                if is_symmetric_event(event) {
                                    symmetric_event_count += 1;
                                }
                            }
                        }
                        let event_parity_score = if event_count == 0 {
                            100.0
                        } else {
                            (symmetric_event_count as f64 / event_count as f64) * 100.0
                        };

                        let non_parity_units = contract
                            .units
                            .iter()
                            .filter(|unit| !unit.parity_ready)
                            .map(|unit| unit.name.clone())
                            .collect::<Vec<_>>();
                        if !non_parity_units.is_empty() || event_parity_score < 100.0 {
                            diagnostics.push(EngineBridgeDiagnostic {
                                code: "W7703".to_string(),
                                message: format!(
                                    "logic bridge parity is not full: units parityReady={}/{} eventParityScore={:.2}%",
                                    parity_ready_units, unit_count, event_parity_score
                                ),
                                file: Some(path_display.clone()),
                            });
                        }
                        if !missing_logic_event_bindings.is_empty() {
                            diagnostics.push(EngineBridgeDiagnostic {
                                code: "W7705".to_string(),
                                message: format!(
                                    "FM action bindings without matching FL event: {}",
                                    missing_logic_event_bindings.join(", ")
                                ),
                                file: Some(path_display.clone()),
                            });
                        }

                        logic_status = "ok".to_string();
                        logic_summary = json!({
                            "status": "ok",
                            "input": path_display,
                            "module": contract.module,
                            "unitCount": unit_count,
                            "parityReadyUnits": parity_ready_units,
                            "parityScore": round2(parity_score),
                            "eventCount": event_count,
                            "symmetricEventCount": symmetric_event_count,
                            "eventParityScore": round2(event_parity_score),
                            "nonParityUnits": non_parity_units,
                            "eventNames": logic_events
                                .iter()
                                .map(|event| event.name.clone())
                                .collect::<Vec<_>>(),
                            "fmActionBindingCount": ir_action_bindings.len(),
                            "missingEventBindings": missing_logic_event_bindings.clone(),
                        });
                    }
                    Err(err) => {
                        diagnostics.push(EngineBridgeDiagnostic {
                            code: "W7702".to_string(),
                            message: format!("cannot parse logic bridge `.fl`: {err}"),
                            file: Some(path_display.clone()),
                        });
                        logic_status = "error".to_string();
                        logic_summary = json!({
                            "status": "error",
                            "input": path_display,
                            "module": null,
                            "error": err,
                            "unitCount": 0,
                            "parityReadyUnits": 0,
                            "parityScore": 0.0,
                            "eventCount": 0,
                            "symmetricEventCount": 0,
                            "eventParityScore": 0.0,
                            "nonParityUnits": [],
                            "eventNames": [],
                            "fmActionBindingCount": ir_action_bindings.len(),
                            "missingEventBindings": [],
                        });
                    }
                },
                Err(err) => {
                    diagnostics.push(EngineBridgeDiagnostic {
                        code: "W7702".to_string(),
                        message: format!("cannot read logic bridge `.fl`: {err}"),
                        file: Some(path_display.clone()),
                    });
                    logic_status = "error".to_string();
                    logic_summary = json!({
                        "status": "error",
                        "input": path_display,
                        "module": null,
                        "error": err.to_string(),
                        "unitCount": 0,
                        "parityReadyUnits": 0,
                        "parityScore": 0.0,
                        "eventCount": 0,
                        "symmetricEventCount": 0,
                        "eventParityScore": 0.0,
                        "nonParityUnits": [],
                        "eventNames": [],
                        "fmActionBindingCount": ir_action_bindings.len(),
                        "missingEventBindings": [],
                    });
                }
            }
        }
    }

    let warning_count = diagnostics.len();
    let diagnostics_json = diagnostics
        .iter()
        .map(|diag| {
            json!({
                "code": diag.code,
                "level": "warning",
                "message": diag.message,
                "source": diag.file,
            })
        })
        .collect::<Vec<_>>();

    let manifest = json!({
        "formatVersion": "1.0.0",
        "engineProfile": {
            "id": ENGINE_PROFILE_ID,
            "version": ENGINE_PROFILE_VERSION,
        },
        "input": {
            "fm": normalize_path(input_fm),
            "fl": logic_input,
        },
        "standard": {
            "fm": {
                "entry": ir.entry,
                "componentCount": ir.components.len(),
                "nodeCount": ir.nodes.len(),
                "actionBindingCount": ir_action_bindings.len(),
                "actionBindings": ir_action_bindings,
            },
            "fs": {
                "styleCount": style_count,
                "canonicalStyleCount": canonical_style_count,
                "canonicalCoveragePct": round2(canonical_style_coverage_pct),
            },
            "fl": logic_summary,
        },
        "warningCount": warning_count,
        "diagnostics": diagnostics_json,
    });

    EngineBridgeReport {
        style_count,
        canonical_style_count,
        logic_status,
        logic_input,
        logic_events,
        logic_event_scripts,
        missing_logic_event_bindings,
        warning_count,
        diagnostics,
        manifest,
    }
}

fn discover_logic_file(input_fm: &str) -> Option<PathBuf> {
    let input_path = Path::new(input_fm);
    let base_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    let explicit_candidates = [
        base_dir.join("logic/controllers/app_controller.fl"),
        base_dir.join("logic/controllers/app-controller.fl"),
        base_dir.join("logic/app_controller.fl"),
        base_dir.join("logic/app-controller.fl"),
        base_dir.join("logic/main.fl"),
        base_dir.join("app_controller.fl"),
        base_dir.join("app-controller.fl"),
    ];
    if let Some(found) = explicit_candidates.into_iter().find(|path| path.exists()) {
        return Some(found);
    }

    let logic_dir = base_dir.join("logic");
    if !logic_dir.exists() {
        return None;
    }

    let mut found = Vec::new();
    collect_fl_files(&logic_dir, &mut found);
    found.sort();
    found.into_iter().next()
}

fn collect_fl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_fl_files(&path, out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "fl") {
            out.push(path);
        }
    }
}

fn is_symmetric_event(event: &formo_logic::RuntimeEventContract) -> bool {
    (event.web_calls.is_empty() && event.desktop_calls.is_empty())
        || (!event.web_calls.is_empty()
            && !event.desktop_calls.is_empty()
            && event.web_calls.len() == event.desktop_calls.len())
}

fn collect_ir_action_bindings(ir: &IrProgram) -> Vec<String> {
    const ACTION_PROP_KEYS: [&str; 5] = ["onPress", "onClick", "onChange", "onClose", "action"];

    let mut out = BTreeSet::new();
    for node in &ir.nodes {
        for key in ACTION_PROP_KEYS {
            let Some(value) = node.props.get(key).and_then(|value| value.v.as_str()) else {
                continue;
            };
            let name = value.trim();
            if name.is_empty() {
                continue;
            }
            out.insert(name.to_string());
        }
    }

    out.into_iter().collect()
}

fn collect_logic_event_bridges(contract: &formo_logic::RuntimeContract) -> Vec<LogicEventBridge> {
    let mut out = Vec::new();
    for unit in &contract.units {
        for event in &unit.events {
            out.push(LogicEventBridge {
                name: event.name.clone(),
                total_actions: event.total_actions,
                set_count: event.set_count,
                emit_count: event.emit_count,
                global_calls: event.global_calls.clone(),
                web_calls: event.web_calls.clone(),
                desktop_calls: event.desktop_calls.clone(),
            });
        }
    }
    out
}

fn collect_logic_event_scripts(program: &formo_logic::LogicProgram) -> Vec<LogicEventScriptBridge> {
    let mut by_name: BTreeMap<String, Vec<LogicActionStepBridge>> = BTreeMap::new();

    for unit in &program.units {
        for event in &unit.events {
            let entry = by_name.entry(event.name.clone()).or_default();
            for action in &event.actions {
                let kind = match action.kind {
                    formo_logic::LogicActionKind::Call => "call",
                    formo_logic::LogicActionKind::Set => "set",
                    formo_logic::LogicActionKind::Emit => "emit",
                    formo_logic::LogicActionKind::Throw => "throw",
                    formo_logic::LogicActionKind::Break => "break",
                    formo_logic::LogicActionKind::Continue => "continue",
                    formo_logic::LogicActionKind::Return => "return",
                };

                let scope = match action.scope {
                    formo_logic::LogicScope::Global => "global",
                    formo_logic::LogicScope::Web => "web",
                    formo_logic::LogicScope::Desktop => "desktop",
                };

                let set_value_hint = action.set_value_hint.as_ref().map(|hint| match hint {
                    formo_logic::LogicSetValueHint::BoolLiteral => "boolLiteral",
                    formo_logic::LogicSetValueHint::StringLiteral => "stringLiteral",
                    formo_logic::LogicSetValueHint::IntLiteral => "intLiteral",
                    formo_logic::LogicSetValueHint::FloatLiteral => "floatLiteral",
                    formo_logic::LogicSetValueHint::Expression => "expression",
                });

                let set_operands = action
                    .set_operands
                    .iter()
                    .map(|operand| match operand {
                        formo_logic::LogicSetOperand::StateRef(name) => LogicSetOperandBridge {
                            kind: "stateRef".to_string(),
                            value: Some(name.clone()),
                        },
                        formo_logic::LogicSetOperand::BoolLiteral(value) => LogicSetOperandBridge {
                            kind: "boolLiteral".to_string(),
                            value: Some(value.to_string()),
                        },
                        formo_logic::LogicSetOperand::StringLiteral(value) => {
                            LogicSetOperandBridge {
                                kind: "stringLiteral".to_string(),
                                value: Some(value.clone()),
                            }
                        }
                        formo_logic::LogicSetOperand::IntLiteral(value) => LogicSetOperandBridge {
                            kind: "intLiteral".to_string(),
                            value: Some(value.clone()),
                        },
                        formo_logic::LogicSetOperand::FloatLiteral(value) => {
                            LogicSetOperandBridge {
                                kind: "floatLiteral".to_string(),
                                value: Some(value.clone()),
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let set_operators = action
                    .set_operators
                    .iter()
                    .map(|operator| match operator {
                        formo_logic::LogicSetOperator::Add => "add",
                        formo_logic::LogicSetOperator::Sub => "sub",
                        formo_logic::LogicSetOperator::Mul => "mul",
                        formo_logic::LogicSetOperator::Div => "div",
                        formo_logic::LogicSetOperator::Mod => "mod",
                        formo_logic::LogicSetOperator::Eq => "eq",
                        formo_logic::LogicSetOperator::NotEq => "notEq",
                        formo_logic::LogicSetOperator::Lt => "lt",
                        formo_logic::LogicSetOperator::LtEq => "ltEq",
                        formo_logic::LogicSetOperator::Gt => "gt",
                        formo_logic::LogicSetOperator::GtEq => "gtEq",
                        formo_logic::LogicSetOperator::And => "and",
                        formo_logic::LogicSetOperator::Or => "or",
                    })
                    .map(str::to_string)
                    .collect::<Vec<_>>();

                let set_expression_rpn = action
                    .set_expression_rpn
                    .iter()
                    .map(|token| match token {
                        formo_logic::LogicSetExprToken::Operand(operand) => match operand {
                            formo_logic::LogicSetOperand::StateRef(name) => {
                                LogicSetExprTokenBridge {
                                    kind: "stateRef".to_string(),
                                    value: Some(name.clone()),
                                }
                            }
                            formo_logic::LogicSetOperand::BoolLiteral(value) => {
                                LogicSetExprTokenBridge {
                                    kind: "boolLiteral".to_string(),
                                    value: Some(value.to_string()),
                                }
                            }
                            formo_logic::LogicSetOperand::StringLiteral(value) => {
                                LogicSetExprTokenBridge {
                                    kind: "stringLiteral".to_string(),
                                    value: Some(value.clone()),
                                }
                            }
                            formo_logic::LogicSetOperand::IntLiteral(value) => {
                                LogicSetExprTokenBridge {
                                    kind: "intLiteral".to_string(),
                                    value: Some(value.clone()),
                                }
                            }
                            formo_logic::LogicSetOperand::FloatLiteral(value) => {
                                LogicSetExprTokenBridge {
                                    kind: "floatLiteral".to_string(),
                                    value: Some(value.clone()),
                                }
                            }
                        },
                        formo_logic::LogicSetExprToken::Operator(operator) => {
                            let op = match operator {
                                formo_logic::LogicSetOperator::Add => "add",
                                formo_logic::LogicSetOperator::Sub => "sub",
                                formo_logic::LogicSetOperator::Mul => "mul",
                                formo_logic::LogicSetOperator::Div => "div",
                                formo_logic::LogicSetOperator::Mod => "mod",
                                formo_logic::LogicSetOperator::Eq => "eq",
                                formo_logic::LogicSetOperator::NotEq => "notEq",
                                formo_logic::LogicSetOperator::Lt => "lt",
                                formo_logic::LogicSetOperator::LtEq => "ltEq",
                                formo_logic::LogicSetOperator::Gt => "gt",
                                formo_logic::LogicSetOperator::GtEq => "gtEq",
                                formo_logic::LogicSetOperator::And => "and",
                                formo_logic::LogicSetOperator::Or => "or",
                            };
                            LogicSetExprTokenBridge {
                                kind: "operator".to_string(),
                                value: Some(op.to_string()),
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                entry.push(LogicActionStepBridge {
                    kind: kind.to_string(),
                    scope: scope.to_string(),
                    target: action.target.clone(),
                    set_value_hint: set_value_hint.map(str::to_string),
                    set_operands,
                    set_operators,
                    set_expression_rpn,
                });
            }
        }
    }

    by_name
        .into_iter()
        .map(|(name, steps)| LogicEventScriptBridge { name, steps })
        .collect()
}

fn round2(input: f64) -> f64 {
    (input * 100.0).round() / 100.0
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}
