use formo_parser::{parse, AstComponent, AstProgram};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ResolvedProgram {
    pub ast: AstProgram,
    pub modules: Vec<String>,
    pub component_origins: HashMap<String, String>,
    pub style_modules: Vec<String>,
}

pub fn resolve(root_ast: AstProgram, input_path: &str) -> Result<ResolvedProgram, String> {
    let root_path = canonicalize_existing(Path::new(input_path))
        .map_err(|message| format_diag("E1200", input_path, 1, 1, &message))?;
    let root_dir = root_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut state = ResolveState {
        root_dir,
        ..ResolveState::default()
    };
    state.visit(root_path, Some(root_ast))?;

    Ok(ResolvedProgram {
        ast: AstProgram {
            raw: state.raw_chunks.join("\n"),
            tokens: Vec::new(),
            imports: Vec::new(),
            components: state.components,
        },
        modules: state.module_order,
        component_origins: state.component_origins,
        style_modules: state.style_modules,
    })
}

#[derive(Default)]
struct ResolveState {
    root_dir: PathBuf,
    visiting: Vec<String>,
    visited: HashSet<String>,
    module_order: Vec<String>,
    component_origins: HashMap<String, String>,
    components: Vec<AstComponent>,
    raw_chunks: Vec<String>,
    style_modules: Vec<String>,
    seen_style_modules: HashSet<String>,
}

impl ResolveState {
    fn visit(
        &mut self,
        module_path: PathBuf,
        parsed_override: Option<AstProgram>,
    ) -> Result<(), String> {
        let module_key = path_to_string(&module_path);
        let module_label = self.display_module(&module_key);

        if let Some(index) = self.visiting.iter().position(|p| p == &module_key) {
            let mut cycle = self.visiting[index..].to_vec();
            cycle.push(module_key.clone());
            let rendered = cycle
                .iter()
                .map(|path| self.display_module(path))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(format_diag(
                "E1201",
                &module_label,
                1,
                1,
                &format!("cyclic import detected: {rendered}"),
            ));
        }

        if self.visited.contains(&module_key) {
            return Ok(());
        }

        self.visiting.push(module_key.clone());
        let ast = match parsed_override {
            Some(ast) => ast,
            None => {
                let source = fs::read_to_string(&module_path).map_err(|e| {
                    format_diag(
                        "E1206",
                        &module_label,
                        1,
                        1,
                        &format!("cannot read module: {e}"),
                    )
                })?;
                parse(&source).map_err(|e| format_parser_diag("E1100", &module_label, &e))?
            }
        };

        let mut seen_aliases = HashSet::new();
        for import in &ast.imports {
            if let Some(alias) = &import.alias {
                if !seen_aliases.insert(alias.clone()) {
                    return Err(format_diag(
                        "E1205",
                        &module_label,
                        import.line,
                        import.col,
                        &format!("duplicate import alias `{alias}`"),
                    ));
                }
            }

            let import_path = resolve_import_path(&module_path, &import.path)
                .map_err(|e| format_diag("E1202", &module_label, import.line, import.col, &e))?;
            let import_key = path_to_string(&import_path);
            if is_style_module(&import_key) {
                if self.seen_style_modules.insert(import_key.clone()) {
                    self.style_modules.push(import_key);
                }
            } else if is_markup_module(&import_key) {
                self.visit(import_path, None)?;
            } else {
                return Err(format_diag(
                    "E1203",
                    &module_label,
                    import.line,
                    import.col,
                    &format!("unsupported import extension in `{}`", import.path),
                ));
            }
        }

        self.register_components(&module_key, &module_label, &ast.components)?;
        self.raw_chunks.push(ast.raw);

        self.visiting.pop();
        self.visited.insert(module_key.clone());
        self.module_order.push(module_key);

        Ok(())
    }

    fn display_module(&self, absolute_or_norm_path: &str) -> String {
        compact_path(absolute_or_norm_path, &self.root_dir)
    }

    fn register_components(
        &mut self,
        module_key: &str,
        module_label: &str,
        components: &[AstComponent],
    ) -> Result<(), String> {
        for component in components {
            if let Some(existing_origin) = self.component_origins.get(&component.name) {
                return Err(format_diag(
                    "E1204",
                    module_label,
                    component.line,
                    component.col,
                    &format!(
                        "duplicate component `{}` in {} and {}",
                        component.name,
                        self.display_module(existing_origin),
                        self.display_module(module_key)
                    ),
                ));
            }

            self.component_origins
                .insert(component.name.clone(), module_key.to_string());
            self.components.push(component.clone());
        }

        Ok(())
    }
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("cannot resolve path {}: {e}", path.display()))?;
    Ok(canonical)
}

fn resolve_import_path(module_file: &Path, import_path: &str) -> Result<PathBuf, String> {
    let base_dir = module_file.parent().ok_or_else(|| {
        format!(
            "cannot resolve parent directory for module {}",
            module_file.display()
        )
    })?;

    let candidate = base_dir.join(import_path);
    let from_display = module_file
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path_to_string(module_file));
    canonicalize_existing(&candidate).map_err(|_| {
        format!(
            "import target not found: {} (from {})",
            import_path, from_display
        )
    })
}

fn path_to_string(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();
    let without_extended_prefix = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    without_extended_prefix.replace('\\', "/")
}

fn is_style_module(path: &str) -> bool {
    path.ends_with(".fs")
}

fn is_markup_module(path: &str) -> bool {
    path.ends_with(".fm")
}

fn format_diag(code: &str, file: &str, line: usize, col: usize, message: &str) -> String {
    format!("{code} {file}:{line}:{col} {message}")
}

fn format_parser_diag(code: &str, file: &str, raw: &str) -> String {
    if let Some((message, line, col)) = parse_parser_line_col(raw) {
        return format_diag(code, file, line, col, message);
    }
    format_diag(code, file, 1, 1, raw)
}

fn parse_parser_line_col(raw: &str) -> Option<(&str, usize, usize)> {
    let marker = " at ";
    let index = raw.rfind(marker)?;
    let message = raw[..index].trim();
    let location = raw[index + marker.len()..].trim();
    let (line_raw, col_raw) = location.split_once(':')?;
    let line = line_raw.parse::<usize>().ok()?;
    let col = col_raw.parse::<usize>().ok()?;
    Some((message, line, col))
}

fn compact_path(path: &str, root_dir: &Path) -> String {
    let candidate = PathBuf::from(path);
    if let Ok(relative) = candidate.strip_prefix(root_dir) {
        let rendered = path_to_string(relative);
        if !rendered.is_empty() {
            return rendered;
        }
    }

    candidate
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempWorkspace {
        root: PathBuf,
    }

    impl TempWorkspace {
        fn new(prefix: &str) -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos();
            let root =
                std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), stamp));
            fs::create_dir_all(&root).expect("should create temp workspace");
            Self { root }
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_file(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("should create parent directory");
        }
        fs::write(path, content).expect("should write file");
    }

    #[test]
    fn cycle_error_uses_compact_path_chain() {
        let workspace = TempWorkspace::new("formo_resolver_cycle");
        write_file(
            &workspace.root,
            "main.fm",
            r#"import "a.fm" as A;
component App() { <Page/> }
"#,
        );
        write_file(
            &workspace.root,
            "a.fm",
            r#"import "b.fm" as B;
component AComp() { <Page/> }
"#,
        );
        write_file(
            &workspace.root,
            "b.fm",
            r#"import "a.fm" as A;
component BComp() { <Page/> }
"#,
        );

        let main_path = workspace.root.join("main.fm");
        let source = fs::read_to_string(&main_path).expect("main.fm exists");
        let ast = parse(&source).expect("parse root ok");
        let err = resolve(ast, main_path.to_str().expect("path should be utf8"))
            .expect_err("should fail on cycle");
        assert!(
            err.contains("cyclic import detected: a.fm -> b.fm -> a.fm"),
            "unexpected error text: {err}"
        );
    }

    #[test]
    fn missing_import_error_uses_compact_from_path() {
        let workspace = TempWorkspace::new("formo_resolver_missing");
        write_file(
            &workspace.root,
            "main.fm",
            r#"import "missing.fm" as Missing;
component App() { <Page/> }
"#,
        );

        let main_path = workspace.root.join("main.fm");
        let source = fs::read_to_string(&main_path).expect("main.fm exists");
        let ast = parse(&source).expect("parse root ok");
        let err = resolve(ast, main_path.to_str().expect("path should be utf8"))
            .expect_err("should fail on missing import");
        assert!(
            err.contains("import target not found: missing.fm (from main.fm)"),
            "unexpected error text: {err}"
        );
    }

    #[test]
    fn reject_duplicate_import_alias_in_same_module() {
        let workspace = TempWorkspace::new("formo_resolver_alias");
        write_file(
            &workspace.root,
            "main.fm",
            r#"import "views/a.fm" as Views;
import "views/b.fm" as Views;
component App() { <Page/> }
"#,
        );
        write_file(
            &workspace.root,
            "views/a.fm",
            r#"component A() { <Page/> }
"#,
        );
        write_file(
            &workspace.root,
            "views/b.fm",
            r#"component B() { <Page/> }
"#,
        );

        let main_path = workspace.root.join("main.fm");
        let source = fs::read_to_string(&main_path).expect("main.fm exists");
        let ast = parse(&source).expect("parse root ok");
        let err = resolve(ast, main_path.to_str().expect("path should be utf8"))
            .expect_err("should fail on duplicate alias");
        assert!(
            err.contains("duplicate import alias `Views`"),
            "unexpected error text: {err}"
        );
        assert!(
            err.contains("main.fm:2:1"),
            "line/col should be included: {err}"
        );
    }
}
