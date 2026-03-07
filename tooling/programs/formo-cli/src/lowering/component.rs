use super::node::lower_node_expanded;
use super::values::resolve_ast_value;
use super::{lowering_error, LoweringResources, LoweringState, SlotBinding};
use formo_parser::{AstComponent, AstNode, AstValue};
use std::collections::{HashMap, HashSet};

pub(super) fn expand_component_call(
    node: &AstNode,
    target_component: &AstComponent,
    caller_source_file: &str,
    caller_env: &HashMap<String, AstValue>,
    resources: &LoweringResources<'_>,
    state: &mut LoweringState,
) -> Result<String, String> {
    if state
        .expansion_stack
        .iter()
        .any(|name| name == &target_component.name)
    {
        let mut path = state.expansion_stack.clone();
        path.push(target_component.name.clone());
        return Err(lowering_error(
            "E1403",
            caller_source_file,
            node.line,
            node.col,
            &format!(
                "recursive component expansion detected: {}",
                path.join(" -> ")
            ),
        ));
    }

    if target_component.nodes.len() != 1 {
        let target_source = resources
            .component_origins
            .get(&target_component.name)
            .map_or("<unknown>", String::as_str);
        return Err(lowering_error(
            "E1404",
            target_source,
            target_component.line,
            target_component.col,
            &format!(
                "component `{}` must have exactly one root node for expansion",
                target_component.name
            ),
        ));
    }

    let target_source = resources
        .component_origins
        .get(&target_component.name)
        .map_or("<unknown>", String::as_str);

    let mut provided = HashMap::new();
    let mut seen_attr = HashSet::<String>::new();
    for attr in &node.attributes {
        if !seen_attr.insert(attr.name.clone()) {
            return Err(lowering_error(
                "E1405",
                caller_source_file,
                attr.line,
                attr.col,
                &format!(
                    "duplicate prop `{}` on component call `<{}>`",
                    attr.name, node.name
                ),
            ));
        }

        let resolved = resolve_ast_value(&attr.value, caller_env, 0).map_err(|msg| {
            lowering_error("E1411", caller_source_file, attr.line, attr.col, &msg)
        })?;
        provided.insert(attr.name.clone(), resolved);
    }

    let param_map = target_component
        .params
        .iter()
        .map(|param| (param.name.as_str(), param))
        .collect::<HashMap<_, _>>();

    for key in provided.keys() {
        if !param_map.contains_key(key.as_str()) {
            return Err(lowering_error(
                "E1406",
                caller_source_file,
                node.line,
                node.col,
                &format!(
                    "unknown prop `{}` when calling component `{}`",
                    key, target_component.name
                ),
            ));
        }
    }

    let mut next_env = HashMap::new();
    for param in &target_component.params {
        match provided.get(&param.name) {
            Some(value) => {
                next_env.insert(param.name.clone(), value.clone());
            }
            None if !param.optional => {
                return Err(lowering_error(
                    "E1407",
                    caller_source_file,
                    node.line,
                    node.col,
                    &format!(
                        "missing required prop `{}` when calling component `{}`",
                        param.name, target_component.name
                    ),
                ));
            }
            None => {}
        }
    }

    let slot_binding = SlotBinding {
        source_file: caller_source_file.to_string(),
        children: node.children.clone(),
        env: caller_env.clone(),
    };

    state.expansion_stack.push(target_component.name.clone());
    let expanded_root = lower_node_expanded(
        &target_component.nodes[0],
        target_source,
        &next_env,
        Some(&slot_binding),
        resources,
        state,
    );
    state.expansion_stack.pop();

    expanded_root
}
