use crate::error::CliError;
use crate::term::{print_error, print_ok, print_warn, print_watch};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

type Snapshot = BTreeMap<String, (u128, u64)>;

pub fn run_watch_loop(
    command_label: &str,
    input: &str,
    mut run_once: impl FnMut() -> Result<(), CliError>,
) -> Result<(), CliError> {
    print_watch(&format!(
        "watch mode active: {} --input {} (Ctrl+C to stop)",
        command_label, input
    ));

    let mut snapshot = snapshot_sources()?;
    loop {
        match run_once() {
            Ok(()) => print_ok(&format!("{command_label}: ok")),
            Err(err) => {
                if !err.already_printed {
                    print_error(&err.message);
                }
            }
        }

        wait_for_change(&mut snapshot)?;
        print_warn("change detected, rerunning...");
    }
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
            let len = meta.len();
            snapshot.insert(file.to_string_lossy().to_string(), (modified, len));
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
