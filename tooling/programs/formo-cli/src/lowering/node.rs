use super::attrs::lower_attributes_with_env;
use super::component::expand_component_call;
use super::slot::lower_slot_node;
use super::{infer_node_kind, is_builtin, LoweringResources, LoweringState, SlotBinding};
use formo_ir::{IrNode, SourceLoc};
use formo_parser::{AstNode, AstValue};
use std::collections::HashMap;

pub(super) fn lower_node_expanded(
    node: &AstNode,
    source_file: &str,
    env: &HashMap<String, AstValue>,
    slot_binding: Option<&SlotBinding>,
    resources: &LoweringResources<'_>,
    state: &mut LoweringState,
) -> Result<String, String> {
    if node.name == "Slot" {
        return lower_slot_node(node, source_file, slot_binding, resources, state);
    }

    if !is_builtin(&node.name) {
        if let Some(target_component) = resources.component_map.get(&node.name) {
            return expand_component_call(
                node,
                target_component,
                source_file,
                env,
                resources,
                state,
            );
        }
    }

    let node_id = state.next_node_id();
    let mut child_ids = Vec::new();

    for child in &node.children {
        let child_id =
            lower_node_expanded(child, source_file, env, slot_binding, resources, state)?;
        child_ids.push(child_id);
    }

    let (props, mut style_refs) = lower_attributes_with_env(
        &node.attributes,
        source_file,
        env,
        resources.known_style_ids,
    )?;
    if let Some(auto_refs) = resources.auto_style_map.get(&node.name) {
        for auto_ref in auto_refs {
            if !style_refs.iter().any(|existing| existing == auto_ref) {
                style_refs.push(auto_ref.clone());
            }
        }
    }
    state.out_nodes.push(IrNode {
        id: node_id.clone(),
        kind: infer_node_kind(&node.name).to_string(),
        name: node.name.clone(),
        props,
        style_refs,
        children: child_ids,
        source: SourceLoc {
            file: source_file.to_string(),
            line: node.line,
            col: node.col,
        },
    });

    Ok(node_id)
}
