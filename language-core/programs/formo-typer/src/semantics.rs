use crate::rules::{LocalScope, LocalValueKind, PropKind};
use formo_parser::{AstNode, AstParam, AstValue};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum IdentifierType {
    Known(String),
    DynamicUnknown,
}

pub(crate) fn value_matches_param_type(
    value: &AstValue,
    target_param: &AstParam,
    scope_params: &HashMap<String, &AstParam>,
    local_scope: &LocalScope,
) -> bool {
    match value {
        AstValue::Identifier(name) => {
            if let Some(source_param) = scope_params.get(name) {
                return match (&target_param.ty, &source_param.ty) {
                    (Some(expected), Some(actual)) => {
                        normalize_type_name(expected) == normalize_type_name(actual)
                    }
                    (Some(_), None) => true,
                    (None, _) => true,
                };
            }

            if let Some(local_kind) = local_scope.get(name) {
                return local_kind_matches_param_type(*local_kind, target_param.ty.as_deref());
            }

            if let Some(identifier_type) = resolve_identifier_type(name, scope_params, local_scope)
            {
                return identifier_type_matches_param_type(
                    &identifier_type,
                    target_param.ty.as_deref(),
                );
            }

            false
        }
        AstValue::Object(_) => match target_param.ty.as_deref() {
            Some("object") => true,
            Some(_) => false,
            None => true,
        },
        AstValue::List(items) => match target_param.ty.as_deref() {
            Some(ty) => list_value_matches_type(items, ty),
            None => true,
        },
        _ => match target_param.ty.as_deref() {
            Some("string") => matches!(value, AstValue::String(_)),
            Some("bool") => matches!(value, AstValue::Bool(_)),
            Some("int") => matches!(value, AstValue::Int(_)),
            Some("float") => matches!(value, AstValue::Float(_) | AstValue::Int(_)),
            Some("len") => matches!(value, AstValue::Identifier(name) if is_len_literal(name)),
            Some("color") => matches!(value, AstValue::Identifier(name) if is_color_literal(name)),
            Some("object") => matches!(value, AstValue::Object(_)),
            Some(_) | None => true,
        },
    }
}

pub(crate) fn value_matches_kind(
    value: &AstValue,
    expected: PropKind,
    scope_params: &HashMap<String, &AstParam>,
    local_scope: &LocalScope,
) -> bool {
    match value {
        AstValue::Identifier(name) => {
            if let Some(param) = scope_params.get(name) {
                return param_matches_kind(param, expected);
            }
            if let Some(local_kind) = local_scope.get(name) {
                return local_kind_matches_prop_kind(*local_kind, expected);
            }
            if let Some(identifier_type) = resolve_identifier_type(name, scope_params, local_scope)
            {
                return identifier_type_matches_kind(&identifier_type, expected);
            }
            if name.contains('.') {
                return false;
            }
            literal_identifier_matches(name, expected)
        }
        AstValue::String(raw) => literal_string_matches(raw, expected),
        AstValue::Bool(_) => matches!(expected, PropKind::Bool | PropKind::BoolOrStateBool),
        AstValue::Int(_) => matches!(expected, PropKind::Int | PropKind::Float),
        AstValue::Float(_) => expected == PropKind::Float,
        AstValue::List(items) => literal_list_matches(items, expected),
        AstValue::Object(_) => false,
    }
}

fn param_matches_kind(param: &AstParam, expected: PropKind) -> bool {
    let Some(ty) = &param.ty else {
        return true;
    };

    normalized_type_matches_prop_kind(&normalize_type_name(ty), expected)
}

fn literal_identifier_matches(raw: &str, expected: PropKind) -> bool {
    match expected {
        PropKind::String => false,
        PropKind::Bool => false,
        PropKind::BoolOrStateBool => !raw.trim().is_empty(),
        PropKind::Int => false,
        PropKind::Float => false,
        PropKind::Len => is_len_literal(raw),
        PropKind::Color => is_color_literal(raw),
        PropKind::StringOrIdent => !raw.trim().is_empty(),
        PropKind::ListSource => !raw.trim().is_empty(),
        PropKind::StateString => !raw.trim().is_empty(),
        PropKind::StateBool => !raw.trim().is_empty(),
        PropKind::ActionVoid => !raw.trim().is_empty(),
        PropKind::ActionString => !raw.trim().is_empty(),
        PropKind::ActionBool => !raw.trim().is_empty(),
    }
}

fn literal_string_matches(raw: &str, expected: PropKind) -> bool {
    match expected {
        PropKind::String => true,
        PropKind::Bool => false,
        PropKind::BoolOrStateBool => {
            raw.eq_ignore_ascii_case("true") || raw.eq_ignore_ascii_case("false")
        }
        PropKind::Int => false,
        PropKind::Float => false,
        PropKind::Len => is_len_literal(raw),
        PropKind::Color => is_color_literal(raw),
        PropKind::StringOrIdent => true,
        PropKind::ListSource => !raw.trim().is_empty(),
        PropKind::StateString => false,
        PropKind::StateBool => false,
        PropKind::ActionVoid => false,
        PropKind::ActionString => false,
        PropKind::ActionBool => false,
    }
}

fn literal_list_matches(items: &[AstValue], expected: PropKind) -> bool {
    match expected {
        PropKind::ListSource => items.iter().all(is_supported_list_literal_item),
        _ => false,
    }
}

fn is_supported_list_literal_item(value: &AstValue) -> bool {
    match value {
        AstValue::String(_) | AstValue::Bool(_) | AstValue::Int(_) | AstValue::Float(_) => true,
        AstValue::Identifier(name) => !name.trim().is_empty(),
        AstValue::List(items) => items.iter().all(is_supported_list_literal_item),
        AstValue::Object(entries) => entries.values().all(is_supported_list_literal_item),
    }
}

fn list_value_matches_type(items: &[AstValue], raw_target_ty: &str) -> bool {
    let normalized = normalize_type_name(raw_target_ty);
    let inner = if normalized.starts_with("list<") && normalized.ends_with('>') {
        &normalized[5..normalized.len() - 1]
    } else if normalized.starts_with("state<list<") && normalized.ends_with(">>") {
        &normalized[11..normalized.len() - 2]
    } else {
        return false;
    };

    items
        .iter()
        .all(|item| list_item_matches_inner_type(item, inner))
}

fn list_item_matches_inner_type(item: &AstValue, inner_ty: &str) -> bool {
    if let Some(nested) = inner_ty
        .strip_prefix("list<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let AstValue::List(items) = item else {
            return false;
        };
        return items
            .iter()
            .all(|nested_item| list_item_matches_inner_type(nested_item, nested));
    }

    match inner_ty {
        "string" => matches!(item, AstValue::String(_)),
        "bool" => matches!(item, AstValue::Bool(_)),
        "int" => matches!(item, AstValue::Int(_)),
        "float" => matches!(item, AstValue::Float(_) | AstValue::Int(_)),
        "len" => matches!(item, AstValue::Identifier(name) if is_len_literal(name)),
        "color" => matches!(item, AstValue::Identifier(name) if is_color_literal(name)),
        "object" => matches!(item, AstValue::Object(_)),
        _ => true,
    }
}

fn local_kind_matches_param_type(kind: LocalValueKind, target_ty: Option<&str>) -> bool {
    let Some(target_ty) = target_ty else {
        return true;
    };

    let normalized = normalize_type_name(target_ty);
    match kind {
        LocalValueKind::Unknown => !matches!(
            normalized.as_str(),
            "state<string>" | "state<bool>" | "action<void>" | "action<string>" | "action<bool>"
        ),
        LocalValueKind::String => normalized == "string",
        LocalValueKind::Bool => normalized == "bool",
        LocalValueKind::Int => normalized == "int" || normalized == "float",
        LocalValueKind::Float => normalized == "float",
        LocalValueKind::Len => normalized == "len",
        LocalValueKind::Color => normalized == "color",
    }
}

fn local_kind_matches_prop_kind(kind: LocalValueKind, expected: PropKind) -> bool {
    match kind {
        LocalValueKind::Unknown => matches!(
            expected,
            PropKind::String
                | PropKind::Bool
                | PropKind::BoolOrStateBool
                | PropKind::Int
                | PropKind::Float
                | PropKind::Len
                | PropKind::Color
                | PropKind::StringOrIdent
                | PropKind::ListSource
        ),
        LocalValueKind::String => matches!(expected, PropKind::String | PropKind::StringOrIdent),
        LocalValueKind::Bool => matches!(
            expected,
            PropKind::Bool | PropKind::BoolOrStateBool | PropKind::StringOrIdent
        ),
        LocalValueKind::Int => matches!(
            expected,
            PropKind::Int | PropKind::Float | PropKind::StringOrIdent
        ),
        LocalValueKind::Float => matches!(expected, PropKind::Float | PropKind::StringOrIdent),
        LocalValueKind::Len => matches!(expected, PropKind::Len | PropKind::StringOrIdent),
        LocalValueKind::Color => matches!(expected, PropKind::Color | PropKind::StringOrIdent),
    }
}

fn identifier_type_matches_kind(identifier_type: &IdentifierType, expected: PropKind) -> bool {
    match identifier_type {
        IdentifierType::Known(actual_ty) => normalized_type_matches_prop_kind(actual_ty, expected),
        IdentifierType::DynamicUnknown => {
            local_kind_matches_prop_kind(LocalValueKind::Unknown, expected)
        }
    }
}

fn identifier_type_matches_param_type(
    identifier_type: &IdentifierType,
    target_ty: Option<&str>,
) -> bool {
    match identifier_type {
        IdentifierType::Known(actual_ty) => type_assignable_to_param(actual_ty, target_ty),
        IdentifierType::DynamicUnknown => {
            local_kind_matches_param_type(LocalValueKind::Unknown, target_ty)
        }
    }
}

fn type_assignable_to_param(actual_ty: &str, target_ty: Option<&str>) -> bool {
    let Some(target_ty) = target_ty else {
        return true;
    };

    let target_ty = normalize_type_name(target_ty);
    actual_ty == target_ty || (actual_ty == "int" && target_ty == "float")
}

fn normalized_type_matches_prop_kind(ty: &str, expected: PropKind) -> bool {
    match expected {
        PropKind::String => ty == "string",
        PropKind::Bool => ty == "bool",
        PropKind::BoolOrStateBool => ty == "bool" || ty == "state<bool>",
        PropKind::Int => ty == "int",
        PropKind::Float => ty == "float" || ty == "int",
        PropKind::Len => ty == "len",
        PropKind::Color => ty == "color",
        PropKind::StringOrIdent => {
            ty == "string" || ty == "id" || ty == "icon" || ty == "asset" || ty.starts_with("enum<")
        }
        PropKind::ListSource => {
            ty == "string" || ty.starts_with("list<") || ty.starts_with("state<list<")
        }
        PropKind::StateString => ty == "state<string>",
        PropKind::StateBool => ty == "state<bool>",
        PropKind::ActionVoid => ty == "action<void>" || ty == "action",
        PropKind::ActionString => ty == "action<string>",
        PropKind::ActionBool => ty == "action<bool>",
    }
}

fn resolve_identifier_type(
    name: &str,
    scope_params: &HashMap<String, &AstParam>,
    local_scope: &LocalScope,
) -> Option<IdentifierType> {
    if let Some(kind) = local_scope.get(name) {
        return match kind {
            LocalValueKind::Unknown => Some(IdentifierType::DynamicUnknown),
            _ => Some(IdentifierType::Known(
                local_kind_type_name(*kind).to_string(),
            )),
        };
    }

    if let Some(param) = scope_params.get(name) {
        return match &param.ty {
            Some(ty) => Some(IdentifierType::Known(normalize_type_name(ty))),
            None => Some(IdentifierType::DynamicUnknown),
        };
    }

    let mut parts = name.split('.');
    let root = parts.next()?;
    let segments = parts.collect::<Vec<_>>();
    if segments.is_empty() {
        return None;
    }

    if let Some(kind) = local_scope.get(root) {
        return match kind {
            LocalValueKind::Unknown => Some(IdentifierType::DynamicUnknown),
            _ => None,
        };
    }

    if let Some(param) = scope_params.get(root) {
        return match &param.ty {
            Some(ty) => resolve_path_from_param_type(ty, &segments),
            None => Some(IdentifierType::DynamicUnknown),
        };
    }

    None
}

fn resolve_path_from_param_type(raw_ty: &str, segments: &[&str]) -> Option<IdentifierType> {
    let mut current = unwrap_state_type(normalize_type_name(raw_ty));

    for segment in segments {
        current = unwrap_state_type(current);

        if is_numeric_segment(segment) {
            let inner = extract_generic_inner(&current, "list")?;
            current = inner.to_string();
            continue;
        }

        if current == "object" {
            return Some(IdentifierType::DynamicUnknown);
        }

        return None;
    }

    Some(IdentifierType::Known(unwrap_state_type(current)))
}

fn local_kind_type_name(kind: LocalValueKind) -> &'static str {
    match kind {
        LocalValueKind::Unknown => "unknown",
        LocalValueKind::String => "string",
        LocalValueKind::Bool => "bool",
        LocalValueKind::Int => "int",
        LocalValueKind::Float => "float",
        LocalValueKind::Len => "len",
        LocalValueKind::Color => "color",
    }
}

fn is_numeric_segment(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(|ch| ch.is_ascii_digit())
}

fn unwrap_state_type(mut ty: String) -> String {
    while let Some(inner) = extract_generic_inner(&ty, "state") {
        ty = inner.to_string();
    }
    ty
}

fn extract_generic_inner<'a>(ty: &'a str, container: &str) -> Option<&'a str> {
    let prefix = format!("{container}<");
    if !ty.starts_with(&prefix) || !ty.ends_with('>') {
        return None;
    }

    Some(&ty[prefix.len()..ty.len() - 1])
}

pub(crate) fn extend_scope_for_for_node(
    node: &AstNode,
    base_scope: &LocalScope,
    scope_params: &HashMap<String, &AstParam>,
) -> LocalScope {
    let mut next_scope = base_scope.clone();
    let alias_name = for_alias_name(node).unwrap_or_else(|| "item".to_string());
    let alias_kind = infer_for_alias_kind(node, scope_params);
    next_scope.insert(alias_name.clone(), alias_kind);
    next_scope.insert(format!("{alias_name}Index"), LocalValueKind::Int);
    for (field_path, field_kind) in infer_for_alias_field_kinds(node) {
        next_scope.insert(format!("{alias_name}.{field_path}"), field_kind);
    }
    next_scope
}

fn for_alias_name(node: &AstNode) -> Option<String> {
    let attr = node.attributes.iter().find(|attr| attr.name == "as")?;
    let raw = match &attr.value {
        AstValue::Identifier(name) | AstValue::String(name) => name.trim(),
        _ => return None,
    };
    if raw.is_empty() {
        return None;
    }
    Some(raw.to_string())
}

fn infer_for_alias_kind(
    node: &AstNode,
    scope_params: &HashMap<String, &AstParam>,
) -> LocalValueKind {
    let Some(each_attr) = node.attributes.iter().find(|attr| attr.name == "each") else {
        return LocalValueKind::Unknown;
    };

    match &each_attr.value {
        AstValue::Identifier(name) => {
            let Some(param) = scope_params.get(name) else {
                return LocalValueKind::Unknown;
            };
            let Some(param_ty) = &param.ty else {
                return LocalValueKind::Unknown;
            };
            infer_list_item_kind_from_type_name(param_ty)
        }
        AstValue::List(items) => infer_list_item_kind_from_literal(items),
        _ => LocalValueKind::Unknown,
    }
}

fn infer_for_alias_field_kinds(node: &AstNode) -> HashMap<String, LocalValueKind> {
    let Some(each_attr) = node.attributes.iter().find(|attr| attr.name == "each") else {
        return HashMap::new();
    };

    match &each_attr.value {
        AstValue::List(items) => infer_object_field_kinds_from_list(items),
        _ => HashMap::new(),
    }
}

fn infer_object_field_kinds_from_list(items: &[AstValue]) -> HashMap<String, LocalValueKind> {
    let mut aggregate: Option<HashMap<String, LocalValueKind>> = None;

    for item in items {
        let AstValue::Object(entries) = item else {
            return HashMap::new();
        };

        let mut current = HashMap::new();
        flatten_object_kind_map("", entries, &mut current);

        match &mut aggregate {
            None => aggregate = Some(current),
            Some(existing) => {
                existing.retain(|key, kind| {
                    current.get(key).is_some_and(|candidate| candidate == kind)
                });
            }
        }
    }

    aggregate.unwrap_or_default()
}

fn flatten_object_kind_map(
    prefix: &str,
    entries: &std::collections::BTreeMap<String, AstValue>,
    out: &mut HashMap<String, LocalValueKind>,
) {
    for (key, value) in entries {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        flatten_value_kind_map(&path, value, out);
    }
}

fn flatten_value_kind_map(path: &str, value: &AstValue, out: &mut HashMap<String, LocalValueKind>) {
    let kind = match value {
        AstValue::String(_) => LocalValueKind::String,
        AstValue::Bool(_) => LocalValueKind::Bool,
        AstValue::Int(_) => LocalValueKind::Int,
        AstValue::Float(_) => LocalValueKind::Float,
        AstValue::Identifier(_) | AstValue::List(_) | AstValue::Object(_) => {
            LocalValueKind::Unknown
        }
    };
    out.insert(path.to_string(), kind);

    match value {
        AstValue::Object(nested) => {
            for (child_key, child_value) in nested {
                let child_path = format!("{path}.{child_key}");
                flatten_value_kind_map(&child_path, child_value, out);
            }
        }
        AstValue::List(items) => {
            for (index, item) in items.iter().enumerate() {
                let child_path = format!("{path}.{index}");
                flatten_value_kind_map(&child_path, item, out);
            }
        }
        _ => {}
    }
}

fn infer_list_item_kind_from_type_name(raw: &str) -> LocalValueKind {
    let ty = normalize_type_name(raw);
    let inner = if ty.starts_with("list<") && ty.ends_with('>') {
        &ty[5..ty.len() - 1]
    } else if ty.starts_with("state<list<") && ty.ends_with(">>") {
        &ty[11..ty.len() - 2]
    } else {
        return LocalValueKind::Unknown;
    };

    match inner {
        "string" => LocalValueKind::String,
        "bool" => LocalValueKind::Bool,
        "int" => LocalValueKind::Int,
        "float" => LocalValueKind::Float,
        "len" => LocalValueKind::Len,
        "color" => LocalValueKind::Color,
        _ => LocalValueKind::Unknown,
    }
}

fn infer_list_item_kind_from_literal(items: &[AstValue]) -> LocalValueKind {
    let mut inferred = None;
    for item in items {
        let item_kind = match item {
            AstValue::String(_) => LocalValueKind::String,
            AstValue::Bool(_) => LocalValueKind::Bool,
            AstValue::Int(_) => LocalValueKind::Int,
            AstValue::Float(_) => LocalValueKind::Float,
            _ => LocalValueKind::Unknown,
        };

        if item_kind == LocalValueKind::Unknown {
            return LocalValueKind::Unknown;
        }

        match inferred {
            None => inferred = Some(item_kind),
            Some(existing) if existing == item_kind => {}
            Some(_) => return LocalValueKind::Unknown,
        }
    }

    inferred.unwrap_or(LocalValueKind::Unknown)
}

fn is_len_literal(raw: &str) -> bool {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("auto") {
        return true;
    }

    let units = ["dp", "px", "%", "vw", "vh", "rem", "em"];
    for unit in units {
        if let Some(number) = text.strip_suffix(unit) {
            return number.trim().parse::<f64>().is_ok();
        }
    }

    false
}

fn is_color_literal(raw: &str) -> bool {
    let Some(hex) = raw.strip_prefix('#') else {
        return false;
    };

    matches!(hex.len(), 6 | 8) && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

pub(crate) fn is_string_type(raw: &str) -> bool {
    normalize_type_name(raw) == "string"
}

fn normalize_type_name(raw: &str) -> String {
    raw.chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

pub(crate) fn is_non_empty_style_ref_list(raw: &str) -> bool {
    raw.split(',').any(|piece| !piece.trim().is_empty())
}
