use crate::{Diagnostic, IrComponent, IrNode, IrProgram, IrStyle, Target, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct InternalProgram {
    pub ir_version: String,
    pub entry: String,
    pub target: Target,
    pub tokens: BTreeMap<String, Value>,
    pub components_by_id: BTreeMap<String, IrComponent>,
    pub nodes_by_id: BTreeMap<String, IrNode>,
    pub styles_by_id: BTreeMap<String, IrStyle>,
    pub diagnostics: Vec<Diagnostic>,
}

impl InternalProgram {
    pub fn try_from_public(ir: &IrProgram) -> Result<Self, String> {
        Ok(Self {
            ir_version: ir.ir_version.clone(),
            entry: ir.entry.clone(),
            target: ir.target.clone(),
            tokens: ir.tokens.clone(),
            components_by_id: collect_unique("component", ir.components.iter(), |item| &item.id)?,
            nodes_by_id: collect_unique("node", ir.nodes.iter(), |item| &item.id)?,
            styles_by_id: collect_unique("style", ir.styles.iter(), |item| &item.id)?,
            diagnostics: ir.diagnostics.clone(),
        })
    }

    pub fn into_public(self) -> IrProgram {
        IrProgram {
            ir_version: self.ir_version,
            entry: self.entry,
            target: self.target,
            tokens: self.tokens,
            components: self.components_by_id.into_values().collect(),
            nodes: self.nodes_by_id.into_values().collect(),
            styles: self.styles_by_id.into_values().collect(),
            diagnostics: self.diagnostics,
        }
    }
}

impl TryFrom<&IrProgram> for InternalProgram {
    type Error = String;

    fn try_from(value: &IrProgram) -> Result<Self, Self::Error> {
        Self::try_from_public(value)
    }
}

impl TryFrom<IrProgram> for InternalProgram {
    type Error = String;

    fn try_from(value: IrProgram) -> Result<Self, Self::Error> {
        Self::try_from_public(&value)
    }
}

fn collect_unique<'a, T, I, F>(
    kind: &str,
    items: I,
    key_fn: F,
) -> Result<BTreeMap<String, T>, String>
where
    T: Clone + 'a,
    I: Iterator<Item = &'a T>,
    F: Fn(&T) -> &str,
{
    let mut map = BTreeMap::new();
    for item in items {
        let key = key_fn(item);
        if map.contains_key(key) {
            return Err(format!("duplicate {kind} id `{key}` in IR"));
        }
        map.insert(key.to_string(), item.clone());
    }
    Ok(map)
}
