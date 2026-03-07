use crate::error_codes as code;
use crate::rules::{
    builtin_allows_children, builtin_prop_rules, is_builtin, prop_kind_name, LocalScope,
};
use crate::semantics::{
    extend_scope_for_for_node, is_non_empty_style_ref_list, is_string_type, value_matches_kind,
    value_matches_param_type,
};
use crate::TypeDiagnostic;
use formo_parser::{AstAttr, AstComponent, AstNode, AstParam, AstValue};
use std::collections::{HashMap, HashSet};

pub(crate) fn check_component(
    component: &AstComponent,
    file: &str,
    component_map: &HashMap<String, &AstComponent>,
    diagnostics: &mut Vec<TypeDiagnostic>,
) {
    validate_component_params(component, file, diagnostics);

    if component.nodes.is_empty() {
        diagnostics.push(TypeDiagnostic {
            code: code::COMPONENT_EMPTY_ROOT.to_string(),
            message: format!(
                "component `{}` must have at least one root node",
                component.name
            ),
            file: file.to_string(),
            line: component.line,
            col: component.col,
        });
        return;
    }

    if component.nodes.len() != 1 {
        diagnostics.push(TypeDiagnostic {
            code: code::COMPONENT_MULTI_ROOT.to_string(),
            message: format!(
                "component `{}` must have exactly one root node, found {}",
                component.name,
                component.nodes.len()
            ),
            file: file.to_string(),
            line: component.line,
            col: component.col,
        });
    }

    let scope_params = component
        .params
        .iter()
        .map(|param| (param.name.clone(), param))
        .collect::<HashMap<_, _>>();

    for node in &component.nodes {
        let local_scope = HashMap::new();
        let mut ctx = ValidateCtx {
            component_map,
            scope_params: &scope_params,
            file,
            diagnostics,
        };
        validate_node(node, &local_scope, &mut ctx);
    }
}

fn validate_component_params(component: &AstComponent, file: &str, out: &mut Vec<TypeDiagnostic>) {
    let mut seen = HashSet::new();
    for param in &component.params {
        if !seen.insert(param.name.clone()) {
            out.push(TypeDiagnostic {
                code: code::COMPONENT_DUPLICATE_PARAM.to_string(),
                message: format!(
                    "duplicate parameter `{}` in component `{}`",
                    param.name, component.name
                ),
                file: file.to_string(),
                line: component.line,
                col: component.col,
            });
        }
    }
}

struct ValidateCtx<'a, 'b> {
    component_map: &'a HashMap<String, &'a AstComponent>,
    scope_params: &'a HashMap<String, &'a AstParam>,
    file: &'a str,
    diagnostics: &'b mut Vec<TypeDiagnostic>,
}

fn validate_node(node: &AstNode, local_scope: &LocalScope, ctx: &mut ValidateCtx<'_, '_>) {
    if !node
        .name
        .chars()
        .next()
        .map(|ch| ch.is_ascii_uppercase())
        .unwrap_or(false)
    {
        push_diag(
            ctx,
            code::NODE_NAME_UPPERCASE,
            format!("node `{}` must start with uppercase letter", node.name),
            node.line,
            node.col,
        );
    }

    validate_duplicate_attributes(&node.attributes, ctx);

    if is_builtin(&node.name) {
        validate_builtin_node(node, local_scope, ctx);
    } else if let Some(target_component) = ctx.component_map.get(&node.name) {
        validate_custom_component_call(node, target_component, local_scope, ctx);
    } else {
        push_diag(
            ctx,
            code::NODE_UNKNOWN,
            format!("unknown node `{}`", node.name),
            node.line,
            node.col,
        );
    }

    let for_child_scope = if node.name == "For" {
        Some(extend_scope_for_for_node(
            node,
            local_scope,
            ctx.scope_params,
        ))
    } else {
        None
    };
    let child_scope = for_child_scope.as_ref().unwrap_or(local_scope);

    for child in &node.children {
        validate_node(child, child_scope, ctx);
    }
}

fn validate_duplicate_attributes(attrs: &[AstAttr], ctx: &mut ValidateCtx<'_, '_>) {
    let mut seen = HashSet::new();
    for attr in attrs {
        if !seen.insert(attr.name.clone()) {
            push_diag(
                ctx,
                code::ATTR_DUPLICATE,
                format!("duplicate attribute `{}`", attr.name),
                attr.line,
                attr.col,
            );
        }
    }
}

fn validate_builtin_node(node: &AstNode, local_scope: &LocalScope, ctx: &mut ValidateCtx<'_, '_>) {
    if !builtin_allows_children(&node.name) && !node.children.is_empty() {
        push_diag(
            ctx,
            code::BUILTIN_CHILDREN_FORBIDDEN,
            format!("`{}` does not accept child nodes", node.name),
            node.line,
            node.col,
        );
    }

    let rules = builtin_prop_rules(&node.name);

    for rule in rules {
        if rule.required && !node.attributes.iter().any(|attr| attr.name == rule.name) {
            push_diag(
                ctx,
                code::BUILTIN_REQUIRED_PROP_MISSING,
                format!("`{}` requires prop `{}`", node.name, rule.name),
                node.line,
                node.col,
            );
        }
    }

    for attr in &node.attributes {
        if attr.name == "style" {
            if node.name == "Slot" {
                push_diag(
                    ctx,
                    code::STYLE_SLOT_ATTR_FORBIDDEN,
                    "`Slot` does not accept attributes".to_string(),
                    attr.line,
                    attr.col,
                );
            } else {
                validate_style_attr(attr, ctx);
            }
            continue;
        }

        if let Some(rule) = rules.iter().find(|rule| rule.name == attr.name) {
            if !value_matches_kind(&attr.value, rule.kind, ctx.scope_params, local_scope) {
                push_diag(
                    ctx,
                    code::BUILTIN_INVALID_PROP_TYPE,
                    format!(
                        "invalid type for `{}.{}`; expected {}",
                        node.name,
                        attr.name,
                        prop_kind_name(rule.kind)
                    ),
                    attr.line,
                    attr.col,
                );
            }
        } else {
            push_diag(
                ctx,
                code::BUILTIN_UNKNOWN_PROP,
                format!("unknown prop `{}` on built-in `{}`", attr.name, node.name),
                attr.line,
                attr.col,
            );
        }
    }
}

fn validate_style_attr(attr: &AstAttr, ctx: &mut ValidateCtx<'_, '_>) {
    match &attr.value {
        AstValue::String(value) => {
            if !is_non_empty_style_ref_list(value) {
                push_diag(
                    ctx,
                    code::STYLE_EMPTY,
                    "style attribute cannot be empty".to_string(),
                    attr.line,
                    attr.col,
                );
            }
        }
        AstValue::Identifier(name) => {
            if let Some(param) = ctx.scope_params.get(name) {
                if let Some(param_ty) = &param.ty {
                    if !is_string_type(param_ty) {
                        push_diag(
                            ctx,
                            code::STYLE_INVALID_TYPE,
                            format!(
                                "style attribute parameter `{}` must be string-compatible",
                                name
                            ),
                            attr.line,
                            attr.col,
                        );
                    }
                }
            } else if name.trim().is_empty() {
                push_diag(
                    ctx,
                    code::STYLE_EMPTY,
                    "style attribute cannot be empty".to_string(),
                    attr.line,
                    attr.col,
                );
            }
        }
        _ => {
            push_diag(
                ctx,
                code::STYLE_INVALID_TYPE,
                "style attribute must be string or identifier".to_string(),
                attr.line,
                attr.col,
            );
        }
    }
}

fn validate_custom_component_call(
    node: &AstNode,
    target: &&AstComponent,
    local_scope: &LocalScope,
    ctx: &mut ValidateCtx<'_, '_>,
) {
    let has_slot = component_has_slot(target);
    if !node.children.is_empty() && !has_slot {
        push_diag(
            ctx,
            code::CUSTOM_INLINE_CHILDREN_FORBIDDEN,
            format!(
                "component `{}` does not declare `<Slot/>`, so inline children are not allowed",
                target.name
            ),
            node.line,
            node.col,
        );
    }

    let target_params = target
        .params
        .iter()
        .map(|param| (param.name.as_str(), param))
        .collect::<HashMap<_, _>>();

    for param in &target.params {
        if !param.optional && !node.attributes.iter().any(|attr| attr.name == param.name) {
            push_diag(
                ctx,
                code::CUSTOM_REQUIRED_PROP_MISSING,
                format!(
                    "missing required prop `{}` when calling component `{}`",
                    param.name, target.name
                ),
                node.line,
                node.col,
            );
        }
    }

    for attr in &node.attributes {
        match target_params.get(attr.name.as_str()) {
            Some(param) => {
                if !value_matches_param_type(&attr.value, param, ctx.scope_params, local_scope) {
                    push_diag(
                        ctx,
                        code::CUSTOM_PROP_TYPE_MISMATCH,
                        format!(
                            "prop `{}` does not match expected type {:?} for component `{}`",
                            attr.name, param.ty, target.name
                        ),
                        attr.line,
                        attr.col,
                    );
                }
            }
            None => {
                push_diag(
                    ctx,
                    code::CUSTOM_UNKNOWN_PROP,
                    format!(
                        "unknown prop `{}` when calling component `{}`",
                        attr.name, target.name
                    ),
                    attr.line,
                    attr.col,
                );
            }
        }
    }
}

fn component_has_slot(component: &AstComponent) -> bool {
    component.nodes.iter().any(node_has_slot)
}

fn node_has_slot(node: &AstNode) -> bool {
    if node.name == "Slot" {
        return true;
    }

    node.children.iter().any(node_has_slot)
}

fn push_diag(ctx: &mut ValidateCtx<'_, '_>, code: &str, message: String, line: usize, col: usize) {
    ctx.diagnostics.push(TypeDiagnostic {
        code: code.to_string(),
        message,
        file: ctx.file.to_string(),
        line,
        col,
    });
}
