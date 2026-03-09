use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    LogicActionKind, LogicScope, LogicSetOperand, LogicSetOperator, LogicSetValueHint, LogicUnit,
    LogicUnitKind, LogicUse,
};
use crate::parity::is_symmetric_platform_actions;
use crate::utils::{is_lower_camel_case, is_member_path, starts_uppercase};

const BUILTIN_RUNTIME_ALIASES: &[&str] = &["Browser", "Desktop", "Runtime", "State"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasPlatform {
    Any,
    WebOnly,
    DesktopOnly,
}

pub(crate) fn validate_program(
    module: &str,
    uses: &[LogicUse],
    units: &[LogicUnit],
) -> Result<(), String> {
    if !starts_uppercase(module) {
        return Err("module name must be PascalCase".to_string());
    }
    let mut aliases = BTreeSet::new();
    for item in uses {
        if !aliases.insert(item.alias.clone()) {
            return Err(format!(
                "duplicate use alias `{}` at {}:{}",
                item.alias, item.line, item.col
            ));
        }
    }
    let mut unit_names = BTreeSet::new();
    for unit in units {
        if !unit_names.insert(unit.name.clone()) {
            return Err(format!(
                "duplicate logic unit `{}` at {}:{}",
                unit.name, unit.line, unit.col
            ));
        }
    }

    let mut allowed_aliases = BTreeSet::new();
    for item in uses {
        allowed_aliases.insert(item.alias.clone());
    }
    for unit in units {
        allowed_aliases.insert(unit.name.clone());
    }
    for builtin in BUILTIN_RUNTIME_ALIASES {
        allowed_aliases.insert((*builtin).to_string());
    }
    let mut alias_platforms = BTreeMap::new();
    let mut alias_paths = BTreeMap::new();
    for item in uses {
        alias_platforms.insert(item.alias.clone(), classify_alias_platform(&item.path));
        alias_paths.insert(item.alias.clone(), item.path.clone());
    }

    for unit in units {
        let declared_state_fields: BTreeMap<String, String> = unit
            .state_fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect();
        let mut event_names = BTreeSet::new();
        for event in &unit.events {
            if !event_names.insert(event.name.clone()) {
                return Err(format!(
                    "duplicate event `{}` in {} `{}` at {}:{}",
                    event.name,
                    unit.kind.as_str(),
                    unit.name,
                    unit.line,
                    unit.col
                ));
            }
            if !is_lower_camel_case(&event.name) {
                return Err(format!(
                    "event `{}` in {} `{}` must be lowerCamelCase",
                    event.name,
                    unit.kind.as_str(),
                    unit.name
                ));
            }
            if matches!(unit.kind, LogicUnitKind::Contract) && !event.actions.is_empty() {
                return Err(format!(
                    "contract event `{}` in `{}` must not contain actions",
                    event.name, unit.name
                ));
            }
            if matches!(
                unit.kind,
                LogicUnitKind::Logic | LogicUnitKind::Service | LogicUnitKind::Adapter
            ) && event.actions.is_empty()
            {
                return Err(format!(
                    "event `{}` in {} `{}` must contain at least one action",
                    event.name,
                    unit.kind.as_str(),
                    unit.name
                ));
            }

            let mut has_global_action = false;
            let mut web_actions = 0usize;
            let mut desktop_actions = 0usize;
            let mut seen_platform_action = false;
            let mut seen_web_platform_action = false;
            let mut seen_desktop_platform_action = false;

            for (idx, action) in event.actions.iter().enumerate() {
                match action.scope {
                    crate::ast::LogicScope::Global => {
                        has_global_action = true;
                        if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Adapter)
                            && seen_platform_action
                        {
                            return Err(format!(
                                "event `{}` in {} `{}` must keep global actions before platform blocks",
                                event.name,
                                unit.kind.as_str(),
                                unit.name
                            ));
                        }
                    }
                    crate::ast::LogicScope::Web => {
                        web_actions += 1;
                        seen_platform_action = true;
                        seen_web_platform_action = true;
                        if matches!(unit.kind, LogicUnitKind::Logic)
                            && action.kind != LogicActionKind::Call
                        {
                            return Err(format!(
                                "logic event `{}` in `{}` only allows `action call` inside platform blocks",
                                event.name, unit.name
                            ));
                        }
                    }
                    crate::ast::LogicScope::Desktop => {
                        desktop_actions += 1;
                        seen_platform_action = true;
                        if matches!(unit.kind, LogicUnitKind::Logic)
                            && action.kind != LogicActionKind::Call
                        {
                            return Err(format!(
                                "logic event `{}` in `{}` only allows `action call` inside platform blocks",
                                event.name, unit.name
                            ));
                        }
                        if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Adapter)
                            && seen_web_platform_action
                            && seen_desktop_platform_action
                        {
                            return Err(format!(
                                "event `{}` in {} `{}` must keep platform actions grouped as desktop then web",
                                event.name,
                                unit.kind.as_str(),
                                unit.name
                            ));
                        }
                        seen_desktop_platform_action = true;
                    }
                }
                if matches!(unit.kind, LogicUnitKind::Adapter)
                    && action.kind != LogicActionKind::Call
                {
                    return Err(format!(
                        "adapter event `{}` in `{}` only allows `action call`",
                        event.name, unit.name
                    ));
                }
                if matches!(action.kind, LogicActionKind::Return) && idx + 1 != event.actions.len()
                {
                    return Err(format!(
                        "`action return` in event `{}` of `{}` must be the last action",
                        event.name, unit.name
                    ));
                }
                if action.kind == LogicActionKind::Set {
                    let Some(state_field_name) = &action.target else {
                        return Err(format!(
                            "missing state field target in `action set` for event `{}` of `{}`",
                            event.name, unit.name
                        ));
                    };
                    if !is_lower_camel_case(state_field_name) {
                        return Err(format!(
                            "state field target `{}` in event `{}` must be lowerCamelCase",
                            state_field_name, event.name
                        ));
                    }
                    if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Service)
                        && !declared_state_fields.contains_key(state_field_name)
                    {
                        return Err(format!(
                            "unknown state field `{}` in `action set` event `{}` of `{}`; declare it inside `state` block",
                            state_field_name, event.name, unit.name
                        ));
                    }
                    if let Some(state_type) = declared_state_fields.get(state_field_name) {
                        let Some(value_hint) = action.set_value_hint.as_ref() else {
                            return Err(format!(
                                "missing set value hint in `action set` for state field `{}` (event `{}` of `{}`)",
                                state_field_name, event.name, unit.name
                            ));
                        };
                        if !is_set_value_compatible(state_type, value_hint) {
                            return Err(format!(
                                "type mismatch in `action set {}` for event `{}` of `{}`: state field type `{}` is incompatible with assigned value",
                                state_field_name, event.name, unit.name, state_type
                            ));
                        }
                        validate_set_operands(
                            state_field_name,
                            state_type,
                            &action.set_operands,
                            &action.set_operators,
                            &declared_state_fields,
                            &event.name,
                            &unit.name,
                        )?;
                    }
                }
                if action.kind != LogicActionKind::Call {
                    continue;
                }
                let Some(target) = &action.target else {
                    return Err(format!(
                        "missing call target in event `{}` of `{}`",
                        event.name, unit.name
                    ));
                };
                let Some((alias, member)) = target.split_once('.') else {
                    return Err(format!(
                        "call target `{}` in event `{}` must follow `Alias.member`",
                        target, event.name
                    ));
                };
                if member.trim().is_empty() {
                    return Err(format!(
                        "call target `{}` in event `{}` must include member name",
                        target, event.name
                    ));
                }
                if !is_member_path(member) {
                    return Err(format!(
                        "call target `{}` in event `{}` must use lowerCamelCase member path",
                        target, event.name
                    ));
                }
                if !allowed_aliases.contains(alias) {
                    return Err(format!(
                        "unknown call alias `{}` in target `{}` (event `{}`)",
                        alias, target, event.name
                    ));
                }
                if let Some(alias_platform) = alias_platforms.get(alias) {
                    let alias_path = alias_paths.get(alias).map_or("<unknown>", String::as_str);
                    if matches!(action.scope, LogicScope::Web)
                        && matches!(alias_platform, AliasPlatform::DesktopOnly)
                    {
                        return Err(format!(
                            "alias `{}` from `{}` is desktop-only and cannot be used inside `platform web` block (event `{}` in `{}`)",
                            alias, alias_path, event.name, unit.name
                        ));
                    }
                    if matches!(action.scope, LogicScope::Desktop)
                        && matches!(alias_platform, AliasPlatform::WebOnly)
                    {
                        return Err(format!(
                            "alias `{}` from `{}` is web-only and cannot be used inside `platform desktop` block (event `{}` in `{}`)",
                            alias, alias_path, event.name, unit.name
                        ));
                    }
                }
                if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Service)
                    && matches!(alias, "Browser" | "Desktop")
                {
                    return Err(format!(
                        "direct runtime alias `{}` in {} `{}` event `{}` is not allowed; route via adapter/contract",
                        alias,
                        unit.kind.as_str(),
                        unit.name,
                        event.name
                    ));
                }
            }

            if matches!(unit.kind, LogicUnitKind::Logic) && !has_global_action {
                return Err(format!(
                    "logic event `{}` in `{}` must contain at least one global action",
                    event.name, unit.name
                ));
            }
            if matches!(unit.kind, LogicUnitKind::Service)
                && (web_actions > 0 || desktop_actions > 0)
            {
                return Err(format!(
                    "service event `{}` in `{}` must not contain platform blocks",
                    event.name, unit.name
                ));
            }
            if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Adapter)
                && (web_actions > 0 || desktop_actions > 0)
            {
                let first_platform_scope =
                    event.actions.iter().find_map(|action| match action.scope {
                        LogicScope::Desktop => Some("desktop"),
                        LogicScope::Web => Some("web"),
                        LogicScope::Global => None,
                    });
                if matches!(first_platform_scope, Some("web")) {
                    return Err(format!(
                        "event `{}` in {} `{}` must declare desktop platform actions before web (desktop-first policy)",
                        event.name,
                        unit.kind.as_str(),
                        unit.name
                    ));
                }
            }
            if matches!(unit.kind, LogicUnitKind::Logic | LogicUnitKind::Adapter)
                && !is_symmetric_platform_actions(web_actions, desktop_actions)
            {
                return Err(format!(
                    "event `{}` in {} `{}` must define symmetric web/desktop platform actions",
                    event.name,
                    unit.kind.as_str(),
                    unit.name
                ));
            }
        }
    }
    Ok(())
}

fn classify_alias_platform(path: &str) -> AliasPlatform {
    let lower = path.to_ascii_lowercase().replace('\\', "/");
    let is_web = lower.contains("/web/")
        || lower.contains("/web_adapter.")
        || lower.contains("/web-adapter.")
        || lower.contains("web_adapter")
        || lower.contains("web-adapter");
    let is_desktop = lower.contains("/desktop/")
        || lower.contains("/desktop_adapter.")
        || lower.contains("/desktop-adapter.")
        || lower.contains("desktop_adapter")
        || lower.contains("desktop-adapter");

    match (is_web, is_desktop) {
        (true, false) => AliasPlatform::WebOnly,
        (false, true) => AliasPlatform::DesktopOnly,
        _ => AliasPlatform::Any,
    }
}

fn is_set_value_compatible(state_type: &str, value_hint: &LogicSetValueHint) -> bool {
    let normalized = state_type.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "bool" | "boolean" => {
            matches!(
                value_hint,
                LogicSetValueHint::BoolLiteral | LogicSetValueHint::Expression
            )
        }
        "string" => matches!(
            value_hint,
            LogicSetValueHint::StringLiteral | LogicSetValueHint::Expression
        ),
        "int" => matches!(
            value_hint,
            LogicSetValueHint::IntLiteral | LogicSetValueHint::Expression
        ),
        "float" | "number" => matches!(
            value_hint,
            LogicSetValueHint::IntLiteral
                | LogicSetValueHint::FloatLiteral
                | LogicSetValueHint::Expression
        ),
        _ => true,
    }
}

fn validate_set_operands(
    state_field_name: &str,
    target_type: &str,
    operands: &[LogicSetOperand],
    operators: &[LogicSetOperator],
    declared_state_fields: &BTreeMap<String, String>,
    event_name: &str,
    unit_name: &str,
) -> Result<(), String> {
    let mut operand_types = Vec::with_capacity(operands.len());
    for operand in operands {
        let operand_type = match operand {
            LogicSetOperand::StateRef(name) => {
                let Some(ty) = declared_state_fields.get(name) else {
                    return Err(format!(
                        "unknown state reference `{}` in `action set {}` expression (event `{}` of `{}`)",
                        name, state_field_name, event_name, unit_name
                    ));
                };
                ty.clone()
            }
            LogicSetOperand::BoolLiteral(_) => "bool".to_string(),
            LogicSetOperand::StringLiteral(_) => "string".to_string(),
            LogicSetOperand::IntLiteral(_) => "int".to_string(),
            LogicSetOperand::FloatLiteral(_) => "float".to_string(),
        };
        operand_types.push(operand_type);
    }

    let inferred_expr_type =
        infer_expression_type(&operand_types, operators).map_err(|reason| {
            format!(
                "invalid expression in `action set {}` (event `{}` of `{}`): {}",
                state_field_name, event_name, unit_name, reason
            )
        })?;
    if let Some(expr_type) = inferred_expr_type {
        if !is_type_assignable(target_type, &expr_type) {
            return Err(format!(
                "type mismatch in `action set {}` expression (event `{}` of `{}`): target type `{}` is incompatible with operand type `{}`",
                state_field_name, event_name, unit_name, target_type, expr_type
            ));
        }
    }
    Ok(())
}

fn infer_expression_type(
    operand_types: &[String],
    operators: &[LogicSetOperator],
) -> Result<Option<String>, String> {
    if operand_types.is_empty() {
        return Ok(None);
    }
    let has_logical = operators
        .iter()
        .any(|op| matches!(op, LogicSetOperator::And | LogicSetOperator::Or));
    let has_comparison = operators.iter().any(|op| {
        matches!(
            op,
            LogicSetOperator::Eq
                | LogicSetOperator::NotEq
                | LogicSetOperator::Lt
                | LogicSetOperator::LtEq
                | LogicSetOperator::Gt
                | LogicSetOperator::GtEq
        )
    });
    let has_relational = operators.iter().any(|op| {
        matches!(
            op,
            LogicSetOperator::Lt
                | LogicSetOperator::LtEq
                | LogicSetOperator::Gt
                | LogicSetOperator::GtEq
        )
    });
    let has_arithmetic = operators.iter().any(|op| {
        matches!(
            op,
            LogicSetOperator::Add
                | LogicSetOperator::Sub
                | LogicSetOperator::Mul
                | LogicSetOperator::Div
                | LogicSetOperator::Mod
        )
    });
    let arithmetic_only_add = operators
        .iter()
        .filter(|op| {
            matches!(
                op,
                LogicSetOperator::Add
                    | LogicSetOperator::Sub
                    | LogicSetOperator::Mul
                    | LogicSetOperator::Div
                    | LogicSetOperator::Mod
            )
        })
        .all(|op| matches!(op, LogicSetOperator::Add));

    let has_string = operand_types
        .iter()
        .any(|ty| normalize_basic_type(ty) == "string");
    let has_bool = operand_types
        .iter()
        .any(|ty| normalize_basic_type(ty) == "bool");
    let all_numeric = operand_types
        .iter()
        .all(|ty| matches!(normalize_basic_type(ty).as_str(), "int" | "float"));
    let any_float = operand_types
        .iter()
        .any(|ty| normalize_basic_type(ty) == "float");
    let all_bool = operand_types
        .iter()
        .all(|ty| normalize_basic_type(ty) == "bool");

    if has_logical {
        if has_comparison {
            return Ok(Some("bool".to_string()));
        }
        if !all_bool {
            return Err("logical operators require boolean operands".to_string());
        }
        return Ok(Some("bool".to_string()));
    }

    if has_comparison {
        if has_relational && !all_numeric {
            return Err("relational operators require numeric operands".to_string());
        }
        return Ok(Some("bool".to_string()));
    }

    if has_arithmetic {
        if arithmetic_only_add && has_string && !has_bool {
            return Ok(Some("string".to_string()));
        }
        if !all_numeric {
            return Err("arithmetic operators require numeric operands".to_string());
        }
        if operators
            .iter()
            .any(|op| matches!(op, LogicSetOperator::Div))
            || any_float
        {
            return Ok(Some("float".to_string()));
        }
        return Ok(Some("int".to_string()));
    }

    if operand_types.len() == 1 {
        return Ok(Some(normalize_basic_type(&operand_types[0])));
    }
    if all_bool {
        return Ok(Some("bool".to_string()));
    }
    if all_numeric {
        if any_float {
            return Ok(Some("float".to_string()));
        }
        return Ok(Some("int".to_string()));
    }
    if has_string && !has_bool {
        return Ok(Some("string".to_string()));
    }

    Ok(None)
}

fn normalize_basic_type(ty: &str) -> String {
    let normalized = ty.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "boolean" => "bool".to_string(),
        "number" => "float".to_string(),
        other => other.to_string(),
    }
}

fn is_type_assignable(target_type: &str, source_type: &str) -> bool {
    let target = target_type.trim().to_ascii_lowercase();
    let source = source_type.trim().to_ascii_lowercase();
    match target.as_str() {
        "bool" | "boolean" => matches!(source.as_str(), "bool" | "boolean"),
        "string" => source == "string",
        "int" => source == "int",
        "float" | "number" => matches!(source.as_str(), "int" | "float" | "number"),
        _ => true,
    }
}
