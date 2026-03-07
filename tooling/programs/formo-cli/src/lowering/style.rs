use formo_ir::IrStyle;
use formo_parser::AstValue;
use std::collections::HashMap;

pub(super) fn parse_style_refs(value: &AstValue) -> Result<Vec<String>, String> {
    let raw = match value {
        AstValue::String(text) => text.clone(),
        AstValue::Identifier(text) => text.clone(),
        _ => return Err("style attribute must be string or identifier".to_string()),
    };

    let mut refs = Vec::new();
    for piece in raw.split(',') {
        let trimmed = piece.trim();
        if trimmed.is_empty() {
            continue;
        }
        refs.push(trimmed.to_string());
    }

    if refs.is_empty() {
        return Err("style attribute cannot be empty".to_string());
    }

    Ok(refs)
}

pub(super) fn build_auto_style_map(styles: &[IrStyle]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for style in styles {
        if style.selector.part != "root" {
            continue;
        }
        map.entry(style.selector.component.clone())
            .or_default()
            .push(style.id.clone());
    }
    map
}
