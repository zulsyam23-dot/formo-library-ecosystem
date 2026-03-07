use crate::args::LspArgs;
use crate::error::CliError;
use crate::json_output::{build_error_meta, classify_error_stage};
use crate::lsp_output::{build_lsp_failure_payload, build_lsp_success_payload};
use crate::pipeline::pipeline;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

type Snapshot = BTreeMap<String, (u128, u64)>;

pub fn run_lsp(args: &LspArgs) -> Result<(), CliError> {
    if args.watch {
        run_lsp_watch(&args.input)
    } else {
        emit_publish_diagnostics(&args.input)
    }
}

fn run_lsp_watch(input: &str) -> Result<(), CliError> {
    let mut snapshot = snapshot_sources()?;
    loop {
        emit_publish_diagnostics(input)?;
        wait_for_change(&mut snapshot)?;
    }
}

fn emit_publish_diagnostics(input: &str) -> Result<(), CliError> {
    let payload = match pipeline(input) {
        Ok(ir) => build_lsp_success_payload(input, &ir),
        Err(err) => {
            let stage = classify_error_stage(&err);
            let meta = build_error_meta(&err);
            build_lsp_failure_payload(input, stage, &err, &meta)
        }
    };

    let Some(documents) = payload.get("documents").and_then(|value| value.as_array()) else {
        return Err(CliError::new(
            "invalid lsp payload: missing documents array",
        ));
    };

    for doc in documents {
        let envelope = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": doc
        });
        println!("{envelope}");
    }

    Ok(())
}

fn wait_for_change(last: &mut Snapshot) -> Result<(), CliError> {
    loop {
        thread::sleep(Duration::from_millis(400));
        let current = snapshot_sources()?;
        if &current != last {
            *last = current;
            return Ok(());
        }
    }
}

fn snapshot_sources() -> Result<Snapshot, CliError> {
    let root = std::env::current_dir()
        .map_err(|e| CliError::new(format!("cannot read current directory: {e}")))?;
    let mut files = Vec::new();
    collect_sources(&root, &mut files)?;

    let mut snapshot = Snapshot::new();
    for file in files {
        if let Ok(meta) = fs::metadata(&file) {
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |dur| dur.as_nanos());
            snapshot.insert(file.to_string_lossy().to_string(), (modified, meta.len()));
        }
    }
    Ok(snapshot)
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), CliError> {
    let entries = fs::read_dir(dir)
        .map_err(|e| CliError::new(format!("cannot read {}: {e}", dir.display())))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| CliError::new(format!("cannot read directory entry: {e}")))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CliError::new(format!("cannot read file type {}: {e}", path.display())))?;

        if file_type.is_dir() {
            let Some(name) = path.file_name().and_then(|x| x.to_str()) else {
                continue;
            };
            if should_skip_dir(name) {
                continue;
            }
            collect_sources(&path, out)?;
            continue;
        }

        if file_type.is_file() && is_source_ext(&path) {
            out.push(path);
        }
    }

    Ok(())
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "dist" | "dist2" | "node_modules" | ".idea" | ".vscode"
    )
}

fn is_source_ext(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("fm" | "fs")
    )
}
