use formo_ir::IrProgram;
use std::fs;
use std::path::Path;

#[cfg(feature = "backend-desktop")]
use formo_backend_desktop::DesktopBackend;
#[cfg(feature = "backend-web")]
use formo_backend_web::WebBackend;
#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
use formo_ir::Backend;
#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
use formo_ir::BackendOutput;

#[derive(Debug, Clone, Default)]
pub struct EmitReport {
    pub desktop_parity_warning_count: usize,
    pub desktop_style_warning_count: usize,
    pub desktop_widget_warning_count: usize,
    pub desktop_parity_diagnostics_path: Option<String>,
}

impl EmitReport {
    fn merge(&mut self, other: Self) {
        self.desktop_parity_warning_count += other.desktop_parity_warning_count;
        self.desktop_style_warning_count += other.desktop_style_warning_count;
        self.desktop_widget_warning_count += other.desktop_widget_warning_count;
        if self.desktop_parity_diagnostics_path.is_none() {
            self.desktop_parity_diagnostics_path = other.desktop_parity_diagnostics_path;
        }
    }
}

pub fn emit_target(
    ir: &IrProgram,
    target: &str,
    out_dir: &str,
    production: bool,
    strict_parity: bool,
) -> Result<EmitReport, String> {
    if !Path::new(out_dir).exists() {
        fs::create_dir_all(out_dir).map_err(|e| format!("cannot create {out_dir}: {e}"))?;
    }

    match target {
        "web" => emit_web(ir, out_dir, production, strict_parity),
        "desktop" => emit_desktop(ir, out_dir),
        "multi" => {
            let mut report = emit_web(ir, &format!("{out_dir}/web"), production, false)?;
            report.merge(emit_desktop(ir, &format!("{out_dir}/desktop"))?);
            Ok(report)
        }
        other => Err(format!("unsupported target: {other}")),
    }
}

fn emit_web(
    ir: &IrProgram,
    out_dir: &str,
    production: bool,
    strict_parity: bool,
) -> Result<EmitReport, String> {
    #[cfg(feature = "backend-web")]
    {
        write_output(WebBackend.emit(ir)?, out_dir, production)?;
        if strict_parity {
            return run_web_desktop_parity_audit(ir, out_dir);
        }
        return Ok(EmitReport::default());
    }
    #[cfg(not(feature = "backend-web"))]
    {
        let _ = (ir, out_dir, production, strict_parity);
        Err("target `web` unavailable: rebuild formo-cli with feature `backend-web`".to_string())
    }
}

fn emit_desktop(ir: &IrProgram, out_dir: &str) -> Result<EmitReport, String> {
    #[cfg(feature = "backend-desktop")]
    {
        let output = DesktopBackend.emit(ir)?;
        let mut report = summarize_desktop_output(&output);
        if report.desktop_parity_warning_count > 0 {
            report.desktop_parity_diagnostics_path = Some(format!("{out_dir}/app.native.json"));
        }
        write_output(output, out_dir, false)?;
        return Ok(report);
    }
    #[cfg(not(feature = "backend-desktop"))]
    {
        let _ = (ir, out_dir);
        Err(
            "target `desktop` unavailable: rebuild formo-cli with feature `backend-desktop`"
                .to_string(),
        )
    }
}

#[cfg(feature = "backend-desktop")]
fn summarize_desktop_output(output: &BackendOutput) -> EmitReport {
    let parity_diagnostics = collect_desktop_parity_diagnostics(output);
    summarize_desktop_parity_diagnostics(&parity_diagnostics)
}

#[cfg(feature = "backend-desktop")]
fn summarize_desktop_parity_diagnostics(parity_diagnostics: &[serde_json::Value]) -> EmitReport {
    let mut report = EmitReport::default();
    for diagnostic in parity_diagnostics {
        let Some(code) = diagnostic.get("code").and_then(|value| value.as_str()) else {
            continue;
        };
        report.desktop_parity_warning_count += 1;
        match code {
            "W7601" => report.desktop_style_warning_count += 1,
            "W7602" => report.desktop_widget_warning_count += 1,
            _ => {}
        }
    }
    report
}

#[cfg(feature = "backend-desktop")]
fn collect_desktop_parity_diagnostics(output: &BackendOutput) -> Vec<serde_json::Value> {
    let Some(bundle) = output
        .files
        .iter()
        .find(|file| file.path == "app.native.json")
    else {
        return Vec::new();
    };

    let parsed: serde_json::Value = match serde_json::from_str(&bundle.content) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let Some(diagnostics) = parsed.get("diagnostics").and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .get("code")
                .and_then(|value| value.as_str())
                .is_some_and(|code| code.starts_with("W76"))
        })
        .cloned()
        .collect()
}

#[cfg(all(feature = "backend-web", feature = "backend-desktop"))]
fn run_web_desktop_parity_audit(ir: &IrProgram, out_dir: &str) -> Result<EmitReport, String> {
    let output = DesktopBackend.emit(ir)?;
    let parity_diagnostics = collect_desktop_parity_diagnostics(&output);
    let mut report = summarize_desktop_parity_diagnostics(&parity_diagnostics);

    if report.desktop_parity_warning_count > 0 {
        let parity_report_path = format!("{out_dir}/desktop.parity.json");
        write_parity_report(&parity_report_path, &parity_diagnostics)?;
        report.desktop_parity_diagnostics_path = Some(parity_report_path);
    }

    Ok(report)
}

#[cfg(all(feature = "backend-web", not(feature = "backend-desktop")))]
fn run_web_desktop_parity_audit(_ir: &IrProgram, _out_dir: &str) -> Result<EmitReport, String> {
    Err(
        "target `web` strict parity unavailable: rebuild formo-cli with feature `backend-desktop`"
            .to_string(),
    )
}

#[cfg(all(feature = "backend-web", feature = "backend-desktop"))]
fn write_parity_report(path: &str, diagnostics: &[serde_json::Value]) -> Result<(), String> {
    let payload = serde_json::json!({
        "target": "desktop",
        "warningCount": diagnostics.len(),
        "diagnostics": diagnostics,
    });

    let text =
        serde_json::to_string_pretty(&payload).map_err(|err| format!("cannot serialize parity report: {err}"))?;
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("cannot create {}: {err}", parent.display()))?;
        }
    }
    fs::write(path, text).map_err(|err| format!("cannot write {path}: {err}"))?;
    Ok(())
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn write_output(output: BackendOutput, out_dir: &str, production: bool) -> Result<(), String> {
    if !Path::new(out_dir).exists() {
        fs::create_dir_all(out_dir).map_err(|e| format!("cannot create {out_dir}: {e}"))?;
    }

    for mut file in output.files {
        if production {
            if file.path.ends_with(".js") {
                file.content = minify_js(&file.content);
            } else if file.path.ends_with(".css") {
                file.content = minify_css(&file.content);
            }
        }

        let full = format!("{out_dir}/{}", file.path);
        if let Some(parent) = Path::new(&full).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
        }
        fs::write(&full, file.content).map_err(|e| format!("cannot write {full}: {e}"))?;
    }

    Ok(())
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn minify_css(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut pending_space = false;

    while i < chars.len() {
        let ch = chars[i];
        if !in_single && !in_double && ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            if pending_space && needs_space_css(out.chars().last(), Some(ch)) {
                out.push(' ');
            }
            pending_space = false;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            if pending_space && needs_space_css(out.chars().last(), Some(ch)) {
                out.push(' ');
            }
            pending_space = false;
            out.push(ch);
            i += 1;
            continue;
        }

        if !in_single && !in_double && ch.is_whitespace() {
            pending_space = true;
            i += 1;
            continue;
        }

        if pending_space && needs_space_css(out.chars().last(), Some(ch)) {
            out.push(' ');
        }
        pending_space = false;
        out.push(ch);
        i += 1;
    }

    out.trim().to_string()
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn needs_space_css(prev: Option<char>, next: Option<char>) -> bool {
    let Some(a) = prev else { return false };
    let Some(b) = next else { return false };
    is_word_char(a) && is_word_char(b)
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn minify_js(input: &str) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }

    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut state = State::Normal;
    let mut pending_space = false;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        match state {
            State::Normal => {
                if ch == '/' && next == Some('/') {
                    state = State::LineComment;
                    i += 2;
                    continue;
                }
                if ch == '/' && next == Some('*') {
                    state = State::BlockComment;
                    i += 2;
                    continue;
                }
                if ch.is_whitespace() {
                    pending_space = true;
                    i += 1;
                    continue;
                }
                if ch == '\'' {
                    if pending_space && needs_space_js(out.chars().last(), Some(ch)) {
                        out.push(' ');
                    }
                    pending_space = false;
                    state = State::Single;
                    escaped = false;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '"' {
                    if pending_space && needs_space_js(out.chars().last(), Some(ch)) {
                        out.push(' ');
                    }
                    pending_space = false;
                    state = State::Double;
                    escaped = false;
                    out.push(ch);
                    i += 1;
                    continue;
                }
                if ch == '`' {
                    if pending_space && needs_space_js(out.chars().last(), Some(ch)) {
                        out.push(' ');
                    }
                    pending_space = false;
                    state = State::Template;
                    escaped = false;
                    out.push(ch);
                    i += 1;
                    continue;
                }

                if pending_space && needs_space_js(out.chars().last(), Some(ch)) {
                    out.push(' ');
                }
                pending_space = false;
                out.push(ch);
                i += 1;
            }
            State::LineComment => {
                if ch == '\n' || ch == '\r' {
                    state = State::Normal;
                    pending_space = true;
                }
                i += 1;
            }
            State::BlockComment => {
                if ch == '*' && next == Some('/') {
                    state = State::Normal;
                    pending_space = true;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            State::Single => {
                out.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '\'' {
                    state = State::Normal;
                }
                i += 1;
            }
            State::Double => {
                out.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    state = State::Normal;
                }
                i += 1;
            }
            State::Template => {
                out.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '`' {
                    state = State::Normal;
                }
                i += 1;
            }
        }
    }

    out.trim().to_string()
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn needs_space_js(prev: Option<char>, next: Option<char>) -> bool {
    let Some(a) = prev else { return false };
    let Some(b) = next else { return false };
    is_word_char(a) && is_word_char(b)
}

#[cfg(any(feature = "backend-web", feature = "backend-desktop"))]
fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
