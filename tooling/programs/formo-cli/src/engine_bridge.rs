use formo_ir::IrProgram;
use serde_json::{json, Value as JsonValue};
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
pub struct EngineBridgeReport {
    pub style_count: usize,
    pub canonical_style_count: usize,
    pub logic_status: String,
    pub logic_input: Option<String>,
    pub warning_count: usize,
    pub diagnostics: Vec<EngineBridgeDiagnostic>,
    pub manifest: JsonValue,
}

pub fn collect_engine_bridge_report(input_fm: &str, ir: &IrProgram) -> EngineBridgeReport {
    let style_count = ir.styles.len();
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

fn round2(input: f64) -> f64 {
    (input * 100.0).round() / 100.0
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}
