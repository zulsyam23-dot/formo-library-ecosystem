use super::lowering_error;
use super::style::parse_style_refs;
use super::values::{lower_value, resolve_ast_value};
use formo_ir::Value;
use formo_parser::{AstAttr, AstValue};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(super) fn lower_attributes_with_env(
    attrs: &[AstAttr],
    source_file: &str,
    env: &HashMap<String, AstValue>,
    known_style_ids: &HashSet<String>,
) -> Result<(BTreeMap<String, Value>, Vec<String>), String> {
    let mut props = BTreeMap::new();
    let mut style_refs = Vec::new();
    for attr in attrs {
        let resolved = resolve_ast_value(&attr.value, env, 0)
            .map_err(|msg| lowering_error("E1411", source_file, attr.line, attr.col, &msg))?;
        if attr.name == "style" {
            let refs = parse_style_refs(&resolved)
                .map_err(|msg| lowering_error("E1409", source_file, attr.line, attr.col, &msg))?;
            for style_ref in refs {
                if !known_style_ids.contains(&style_ref) {
                    return Err(lowering_error(
                        "E1410",
                        source_file,
                        attr.line,
                        attr.col,
                        &format!("unknown style `{style_ref}`"),
                    ));
                }
                if !style_refs.iter().any(|existing| existing == &style_ref) {
                    style_refs.push(style_ref);
                }
            }
            continue;
        }

        props.insert(
            attr.name.clone(),
            lower_value(&resolved)
                .map_err(|msg| lowering_error("E1412", source_file, attr.line, attr.col, &msg))?,
        );
    }
    Ok((props, style_refs))
}
