mod attrs;
mod component;
mod node;
mod slot;
mod style;
mod values;

use formo_ir::{Diagnostic, IrComponent, IrNode, IrProgram, SourceLoc, Target, IR_VERSION};
use formo_parser::{AstComponent, AstNode, AstValue};
use formo_style::StyledProgram;
use node::lower_node_expanded;
use std::collections::{HashMap, HashSet};
use style::build_auto_style_map;

pub fn lower_to_ir(styled: &StyledProgram, input: &str) -> Result<IrProgram, String> {
    let ast = &styled.typed.resolved.ast;
    if ast.components.is_empty() {
        return Err(lowering_error(
            "E1400",
            input,
            1,
            1,
            "no component found for IR lowering",
        ));
    }

    let entry = ast
        .components
        .iter()
        .find(|component| component.name == "App")
        .map(|component| component.name.clone())
        .unwrap_or_else(|| ast.components[0].name.clone());

    let component_map = ast
        .components
        .iter()
        .map(|component| (component.name.clone(), component))
        .collect::<HashMap<_, _>>();
    let auto_style_map = build_auto_style_map(&styled.styles);
    let known_style_ids = styled
        .styles
        .iter()
        .map(|style| style.id.clone())
        .collect::<HashSet<_>>();

    let Some(entry_component) = component_map.get(&entry) else {
        return Err(lowering_error(
            "E1401",
            input,
            1,
            1,
            &format!("entry component `{entry}` not found"),
        ));
    };

    let entry_source_file = styled
        .typed
        .resolved
        .component_origins
        .get(&entry)
        .map_or(input, String::as_str);

    if entry_component.nodes.len() != 1 {
        return Err(lowering_error(
            "E1402",
            entry_source_file,
            entry_component.line,
            entry_component.col,
            &format!(
                "entry component `{}` must have exactly one root node for IR lowering",
                entry
            ),
        ));
    }

    let resources = LoweringResources {
        auto_style_map: &auto_style_map,
        known_style_ids: &known_style_ids,
        component_map: &component_map,
        component_origins: &styled.typed.resolved.component_origins,
    };
    let mut state = LoweringState::new(entry.clone());
    let env = HashMap::new();

    let root_id = lower_node_expanded(
        &entry_component.nodes[0],
        entry_source_file,
        &env,
        None,
        &resources,
        &mut state,
    )?;

    let diagnostics = styled
        .typed
        .diagnostics
        .iter()
        .map(|diag| Diagnostic {
            code: diag.code.clone(),
            level: "warning".to_string(),
            message: diag.message.clone(),
            source: SourceLoc {
                file: diag.file.clone(),
                line: diag.line,
                col: diag.col,
            },
        })
        .collect::<Vec<_>>();

    Ok(IrProgram {
        ir_version: IR_VERSION.to_string(),
        entry: entry.clone(),
        target: Target::Multi,
        tokens: styled.tokens.clone(),
        components: vec![IrComponent {
            id: format!("c_{}", entry.to_lowercase()),
            name: entry,
            root_node_id: root_id,
            exports: true,
            source: SourceLoc {
                file: entry_source_file.to_string(),
                line: entry_component.line,
                col: entry_component.col,
            },
        }],
        nodes: state.out_nodes,
        styles: styled.styles.clone(),
        diagnostics,
    })
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "Window"
            | "Page"
            | "Row"
            | "Column"
            | "Stack"
            | "Card"
            | "Text"
            | "Image"
            | "Button"
            | "Input"
            | "Checkbox"
            | "Switch"
            | "Scroll"
            | "Spacer"
            | "Modal"
            | "If"
            | "For"
            | "Slot"
    )
}

fn infer_node_kind(name: &str) -> &'static str {
    match name {
        "If" => "if",
        "For" => "for",
        "Slot" => "slot",
        _ => "element",
    }
}

#[derive(Clone)]
struct SlotBinding {
    source_file: String,
    children: Vec<AstNode>,
    env: HashMap<String, AstValue>,
}

struct NodeIdGen {
    next: usize,
}

impl NodeIdGen {
    fn next_id(&mut self) -> String {
        let id = format!("n{}", self.next);
        self.next += 1;
        id
    }
}

pub(super) struct LoweringResources<'a> {
    pub(super) auto_style_map: &'a HashMap<String, Vec<String>>,
    pub(super) known_style_ids: &'a HashSet<String>,
    pub(super) component_map: &'a HashMap<String, &'a AstComponent>,
    pub(super) component_origins: &'a HashMap<String, String>,
}

pub(super) struct LoweringState {
    pub(super) expansion_stack: Vec<String>,
    id_gen: NodeIdGen,
    pub(super) out_nodes: Vec<IrNode>,
}

impl LoweringState {
    fn new(entry_component: String) -> Self {
        Self {
            expansion_stack: vec![entry_component],
            id_gen: NodeIdGen { next: 1 },
            out_nodes: Vec::new(),
        }
    }

    pub(super) fn next_node_id(&mut self) -> String {
        self.id_gen.next_id()
    }
}

pub(super) fn lowering_error(
    code: &str,
    file: &str,
    line: usize,
    col: usize,
    message: &str,
) -> String {
    format!("{code} {file}:{line}:{col} {message}")
}
