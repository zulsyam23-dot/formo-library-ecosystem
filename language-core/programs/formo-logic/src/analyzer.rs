use std::collections::BTreeSet;

use crate::ast::{
    LogicAction, LogicActionKind, LogicEvent, LogicScope, LogicSetOperand, LogicSetOperator,
    LogicSetValueHint,
};
use crate::utils::{is_ident_continue, is_ident_start, is_lower_camel_case, starts_uppercase};

mod parser_helpers;
use parser_helpers::*;

#[derive(Debug, Clone)]
enum Tok {
    Word(String),
    Number(String),
    StringLit,
    Dot,
    Comma,
    Colon,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AndAnd,
    OrOr,
    Arrow,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semi,
}

#[derive(Debug, Clone, Copy)]
enum ControlKind {
    If,
    For,
    While,
    Match,
    Try,
    Catch,
}

impl ControlKind {
    fn as_str(&self) -> &'static str {
        match self {
            ControlKind::If => "if",
            ControlKind::For => "for",
            ControlKind::While => "while",
            ControlKind::Match => "match",
            ControlKind::Try => "try",
            ControlKind::Catch => "catch",
        }
    }
}

#[derive(Debug, Clone)]
struct ControlBlock {
    kind: ControlKind,
    event_idx: usize,
    depth: usize,
    action_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct LogicFunctionDecl {
    pub(crate) name: String,
    pub(crate) param_count: usize,
    pub(crate) typed_param_count: usize,
    pub(crate) has_return_type: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct LogicEnumDecl {
    pub(crate) name: String,
    pub(crate) variant_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct UnitAnalysis {
    pub(crate) events: Vec<LogicEvent>,
    pub(crate) state_fields: Vec<LogicStateFieldDecl>,
    pub(crate) functions: Vec<LogicFunctionDecl>,
    pub(crate) enums: Vec<LogicEnumDecl>,
    pub(crate) structs: Vec<LogicStructDecl>,
    pub(crate) type_aliases: Vec<LogicTypeAliasDecl>,
}

#[derive(Debug, Clone)]
pub(crate) struct LogicTypeAliasDecl {
    pub(crate) name: String,
    pub(crate) target: String,
}

#[derive(Debug, Clone)]
pub(crate) struct LogicStructDecl {
    pub(crate) name: String,
    pub(crate) field_count: usize,
    pub(crate) typed_field_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct LogicStateFieldDecl {
    pub(crate) name: String,
    pub(crate) ty: String,
}

pub(crate) fn analyze_unit_body(body: &str) -> Result<UnitAnalysis, String> {
    let tokens = tokenize(body);
    let mut i = 0usize;
    let mut depth = 0usize;
    let mut events: Vec<LogicEvent> = Vec::new();
    let mut state_fields: Vec<LogicStateFieldDecl> = Vec::new();
    let mut functions: Vec<LogicFunctionDecl> = Vec::new();
    let mut enums: Vec<LogicEnumDecl> = Vec::new();
    let mut structs: Vec<LogicStructDecl> = Vec::new();
    let mut type_aliases: Vec<LogicTypeAliasDecl> = Vec::new();
    let mut state_field_names = BTreeSet::new();
    let mut function_names = BTreeSet::new();
    let mut enum_names = BTreeSet::new();
    let mut struct_names = BTreeSet::new();
    let mut type_alias_names = BTreeSet::new();
    let mut pending_event: Option<usize> = None;
    let mut event_stack: Vec<(usize, usize)> = Vec::new();
    let mut pending_platform: Option<LogicScope> = None;
    let mut pending_control: Option<(ControlKind, usize)> = None;
    let mut scope_stack: Vec<(LogicScope, usize)> = Vec::new();
    let mut control_stack: Vec<ControlBlock> = Vec::new();

    while i < tokens.len() {
        match &tokens[i] {
            Tok::Word(w) if w == "function" && event_stack.is_empty() => {
                let (function_decl, next) = parse_function_decl(&tokens, i)?;
                if !function_names.insert(function_decl.name.clone()) {
                    return Err(format!(
                        "duplicate function `{}` in unit body",
                        function_decl.name
                    ));
                }
                functions.push(function_decl);
                i = next;
                continue;
            }
            Tok::Word(w) if w == "state" && event_stack.is_empty() => {
                let (state_decl_fields, next) = parse_state_block(&tokens, i)?;
                for field in state_decl_fields {
                    if !state_field_names.insert(field.name.clone()) {
                        return Err(format!(
                            "duplicate state field `{}` in unit body",
                            field.name
                        ));
                    }
                    state_fields.push(field);
                }
                i = next;
                continue;
            }
            Tok::Word(w) if w == "enum" && event_stack.is_empty() => {
                let (enum_decl, next) = parse_enum_decl(&tokens, i)?;
                if !enum_names.insert(enum_decl.name.clone()) {
                    return Err(format!("duplicate enum `{}` in unit body", enum_decl.name));
                }
                enums.push(enum_decl);
                i = next;
                continue;
            }
            Tok::Word(w) if w == "struct" && event_stack.is_empty() => {
                let (struct_decl, next) = parse_struct_decl(&tokens, i)?;
                if !struct_names.insert(struct_decl.name.clone()) {
                    return Err(format!(
                        "duplicate struct `{}` in unit body",
                        struct_decl.name
                    ));
                }
                structs.push(struct_decl);
                i = next;
                continue;
            }
            Tok::Word(w) if w == "type" && event_stack.is_empty() => {
                let (type_alias_decl, next) = parse_type_alias_decl(&tokens, i)?;
                if !type_alias_names.insert(type_alias_decl.name.clone()) {
                    return Err(format!(
                        "duplicate type alias `{}` in unit body",
                        type_alias_decl.name
                    ));
                }
                type_aliases.push(type_alias_decl);
                i = next;
                continue;
            }
            Tok::Word(w) if w == "event" => {
                if let Some(Tok::Word(name)) = tokens.get(i + 1) {
                    events.push(LogicEvent {
                        name: name.clone(),
                        actions: Vec::new(),
                        if_count: 0,
                        for_count: 0,
                        while_count: 0,
                        match_count: 0,
                        try_count: 0,
                        catch_count: 0,
                    });
                    pending_event = Some(events.len() - 1);
                    i += 2;
                    continue;
                }
                return Err("missing event name after `event`".to_string());
            }
            Tok::Word(w) if w == "platform" => {
                let Some(_) = event_stack.last() else {
                    return Err("`platform` found outside any `event`".to_string());
                };
                if let Some(Tok::Word(name)) = tokens.get(i + 1) {
                    pending_platform = match name.as_str() {
                        "web" => Some(LogicScope::Web),
                        "desktop" => Some(LogicScope::Desktop),
                        other => {
                            return Err(format!(
                                "unsupported platform `{other}` (expected `web` or `desktop`)"
                            ));
                        }
                    };
                    i += 2;
                    continue;
                }
                return Err("missing platform target after `platform`".to_string());
            }
            Tok::Word(w)
                if w == "if"
                    || w == "for"
                    || w == "while"
                    || w == "match"
                    || w == "try"
                    || w == "catch" =>
            {
                let Some((event_idx, _)) = event_stack.last().copied() else {
                    return Err(format!("`{w}` found outside any `event`"));
                };
                let kind = match w.as_str() {
                    "if" => {
                        events[event_idx].if_count += 1;
                        ControlKind::If
                    }
                    "for" => {
                        events[event_idx].for_count += 1;
                        ControlKind::For
                    }
                    "while" => {
                        events[event_idx].while_count += 1;
                        ControlKind::While
                    }
                    "match" => {
                        events[event_idx].match_count += 1;
                        ControlKind::Match
                    }
                    "try" => {
                        events[event_idx].try_count += 1;
                        ControlKind::Try
                    }
                    "catch" => {
                        events[event_idx].catch_count += 1;
                        if events[event_idx].catch_count > events[event_idx].try_count {
                            return Err(format!(
                                "`catch` in event `{}` must follow a `try` block",
                                events[event_idx].name
                            ));
                        }
                        ControlKind::Catch
                    }
                    _ => unreachable!(),
                };
                pending_control = Some((kind, event_idx));
                i += 1;
                continue;
            }
            Tok::Word(w) if w == "action" => {
                let Some((event_idx, _)) = event_stack.last().copied() else {
                    return Err("`action` found outside any `event`".to_string());
                };
                let scope = scope_stack
                    .last()
                    .map(|(s, _)| s.clone())
                    .unwrap_or(LogicScope::Global);
                let Some(Tok::Word(kind)) = tokens.get(i + 1) else {
                    return Err("missing action type after `action`".to_string());
                };
                let mut next_index = i + 2;
                match kind.as_str() {
                    "call" => {
                        let (target, next) = parse_call_target(&tokens, i + 2)?;
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Call,
                            scope,
                            target: Some(target),
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                        next_index = next;
                    }
                    "set" => {
                        let (state_target, value_hint, set_operands, set_operators, next) =
                            parse_set_assignment(&tokens, i + 2)?;
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Set,
                            scope,
                            target: Some(state_target),
                            set_value_hint: Some(value_hint),
                            set_operands,
                            set_operators,
                        });
                        next_index = next;
                    }
                    "emit" => {
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Emit,
                            scope,
                            target: None,
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                    }
                    "throw" => {
                        let in_try_or_catch = control_stack.iter().rev().any(|block| {
                            block.event_idx == event_idx
                                && matches!(block.kind, ControlKind::Try | ControlKind::Catch)
                        });
                        if !in_try_or_catch {
                            let event_name = events
                                .get(event_idx)
                                .map(|e| e.name.as_str())
                                .unwrap_or("unknown");
                            return Err(format!(
                                "`action throw` in event `{event_name}` must be inside `try` or `catch` block"
                            ));
                        }
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Throw,
                            scope,
                            target: None,
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                    }
                    "break" => {
                        let in_loop = control_stack.iter().rev().any(|block| {
                            block.event_idx == event_idx
                                && matches!(block.kind, ControlKind::For | ControlKind::While)
                        });
                        if !in_loop {
                            let event_name = events
                                .get(event_idx)
                                .map(|e| e.name.as_str())
                                .unwrap_or("unknown");
                            return Err(format!(
                                "`action break` in event `{event_name}` must be inside `for` or `while` block"
                            ));
                        }
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Break,
                            scope,
                            target: None,
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                    }
                    "continue" => {
                        let in_loop = control_stack.iter().rev().any(|block| {
                            block.event_idx == event_idx
                                && matches!(block.kind, ControlKind::For | ControlKind::While)
                        });
                        if !in_loop {
                            let event_name = events
                                .get(event_idx)
                                .map(|e| e.name.as_str())
                                .unwrap_or("unknown");
                            return Err(format!(
                                "`action continue` in event `{event_name}` must be inside `for` or `while` block"
                            ));
                        }
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Continue,
                            scope,
                            target: None,
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                    }
                    "return" => {
                        events[event_idx].actions.push(LogicAction {
                            kind: LogicActionKind::Return,
                            scope,
                            target: None,
                            set_value_hint: None,
                            set_operands: Vec::new(),
                            set_operators: Vec::new(),
                        });
                    }
                    _ => {
                        return Err(
                            "expected action type `call`, `set`, `emit`, `throw`, `break`, `continue`, or `return`"
                                .to_string(),
                        )
                    }
                }
                if let Some(block) = control_stack.last_mut() {
                    if block.event_idx == event_idx {
                        block.action_count += 1;
                    }
                }
                i = next_index;
                continue;
            }
            Tok::LBrace => {
                depth += 1;
                if let Some(event_idx) = pending_event.take() {
                    event_stack.push((event_idx, depth));
                }
                if let Some(scope) = pending_platform.take() {
                    scope_stack.push((scope, depth));
                }
                if let Some((kind, event_idx)) = pending_control.take() {
                    control_stack.push(ControlBlock {
                        kind,
                        event_idx,
                        depth,
                        action_count: 0,
                    });
                }
            }
            Tok::RBrace => {
                if let Some(block) = control_stack.last() {
                    if block.depth == depth {
                        if block.action_count == 0 {
                            let event_name = events
                                .get(block.event_idx)
                                .map(|e| e.name.as_str())
                                .unwrap_or("unknown");
                            return Err(format!(
                                "`{}` block in event `{}` must contain at least one `action`",
                                block.kind.as_str(),
                                event_name
                            ));
                        }
                        control_stack.pop();
                    }
                }
                if let Some((_, d)) = scope_stack.last() {
                    if *d == depth {
                        scope_stack.pop();
                    }
                }
                if let Some((_, d)) = event_stack.last() {
                    if *d == depth {
                        event_stack.pop();
                    }
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }

    if let Some((kind, event_idx)) = pending_control {
        let event_name = events
            .get(event_idx)
            .map(|e| e.name.as_str())
            .unwrap_or("unknown");
        return Err(format!(
            "missing block body for `{}` in event `{}`",
            kind.as_str(),
            event_name
        ));
    }
    if pending_event.is_some() {
        return Err("missing block body for `event` declaration".to_string());
    }

    Ok(UnitAnalysis {
        events,
        state_fields,
        functions,
        enums,
        structs,
        type_aliases,
    })
}
