use super::node::lower_node_expanded;
use super::{lowering_error, LoweringResources, LoweringState, SlotBinding};
use formo_ir::{IrNode, SourceLoc};
use formo_parser::AstNode;
use std::collections::BTreeMap;

pub(super) fn lower_slot_node(
    slot_node: &AstNode,
    source_file: &str,
    slot_binding: Option<&SlotBinding>,
    resources: &LoweringResources<'_>,
    state: &mut LoweringState,
) -> Result<String, String> {
    let Some(binding) = slot_binding else {
        return Err(lowering_error(
            "E1408",
            source_file,
            slot_node.line,
            slot_node.col,
            "`<Slot/>` used outside custom component expansion",
        ));
    };

    let fragment_id = state.next_node_id();
    let mut child_ids = Vec::new();

    for child in &binding.children {
        let child_id = lower_node_expanded(
            child,
            &binding.source_file,
            &binding.env,
            None,
            resources,
            state,
        )?;
        child_ids.push(child_id);
    }

    state.out_nodes.push(IrNode {
        id: fragment_id.clone(),
        kind: "element".to_string(),
        name: "Fragment".to_string(),
        props: BTreeMap::new(),
        style_refs: Vec::new(),
        children: child_ids,
        source: SourceLoc {
            file: source_file.to_string(),
            line: slot_node.line,
            col: slot_node.col,
        },
    });

    Ok(fragment_id)
}
