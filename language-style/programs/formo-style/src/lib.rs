mod allowlist;
mod diagnostics;
mod parser;
mod value;

#[cfg(test)]
mod tests;

use diagnostics::format_style_diag;
use formo_ir::{IrStyle, Value};
use formo_typer::TypedProgram;
use parser::parse_style_module;
use std::collections::{BTreeMap, HashMap};
use std::fs;

#[derive(Debug, Clone)]
pub struct StyledProgram {
    pub typed: TypedProgram,
    pub styles: Vec<IrStyle>,
    pub tokens: BTreeMap<String, Value>,
}

pub fn compile_styles(typed: TypedProgram) -> Result<StyledProgram, String> {
    let mut styles = Vec::new();
    let mut tokens = BTreeMap::new();
    let mut style_origins: HashMap<String, String> = HashMap::new();

    for style_path in &typed.resolved.style_modules {
        let source = fs::read_to_string(style_path).map_err(|e| {
            format_style_diag(
                "E1300",
                style_path,
                1,
                1,
                &format!("cannot read style module: {e}"),
            )
        })?;
        let module = parse_style_module(&source, style_path, &tokens)?;

        for (key, value) in module.tokens {
            if tokens.contains_key(&key) {
                return Err(format_style_diag(
                    "E1302",
                    style_path,
                    1,
                    1,
                    &format!("duplicate token `{key}`"),
                ));
            }
            tokens.insert(key, value);
        }

        for style in module.styles {
            if let Some(existing_path) = style_origins.get(&style.id) {
                return Err(format_style_diag(
                    "E1303",
                    style_path,
                    1,
                    1,
                    &format!(
                        "duplicate style `{}` in {} and {}",
                        style.id, existing_path, style_path
                    ),
                ));
            }
            style_origins.insert(style.id.clone(), style_path.clone());
            styles.push(style);
        }
    }

    Ok(StyledProgram {
        typed,
        styles,
        tokens,
    })
}
