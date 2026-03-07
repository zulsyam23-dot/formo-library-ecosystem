use formo_resolver::ResolvedProgram;
use std::collections::HashMap;

mod checker;
mod error_codes;
mod rules;
mod semantics;

use checker::check_component;
pub use error_codes::{
    find as find_error_code, is_registered as is_registered_error_code, ErrorCodeEntry, REGISTRY,
};

#[derive(Debug, Clone)]
pub struct TypeDiagnostic {
    pub code: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub resolved: ResolvedProgram,
    pub diagnostics: Vec<TypeDiagnostic>,
}

pub fn type_check(resolved: ResolvedProgram) -> Result<TypedProgram, String> {
    let mut diagnostics = Vec::new();
    let fallback_file = resolved
        .modules
        .last()
        .cloned()
        .unwrap_or_else(|| "<unknown>".to_string());

    let component_map = resolved
        .ast
        .components
        .iter()
        .map(|component| (component.name.clone(), component))
        .collect::<HashMap<_, _>>();

    for component in &resolved.ast.components {
        let file = resolved
            .component_origins
            .get(&component.name)
            .cloned()
            .unwrap_or_else(|| fallback_file.clone());

        check_component(component, &file, &component_map, &mut diagnostics);
    }

    if !diagnostics.is_empty() {
        return Err(format_diagnostics(&diagnostics));
    }

    Ok(TypedProgram {
        resolved,
        diagnostics,
    })
}

fn format_diagnostics(diags: &[TypeDiagnostic]) -> String {
    diags
        .iter()
        .map(|d| format!("{} {}:{}:{} {}", d.code, d.file, d.line, d.col, d.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
