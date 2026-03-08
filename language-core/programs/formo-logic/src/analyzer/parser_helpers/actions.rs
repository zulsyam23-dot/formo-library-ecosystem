use super::super::*;

pub(in crate::analyzer) fn parse_call_target(
    tokens: &[Tok],
    start: usize,
) -> Result<(String, usize), String> {
    let mut i = start;
    let mut parts = Vec::new();

    match tokens.get(i) {
        Some(Tok::Word(word)) => {
            parts.push(word.clone());
            i += 1;
        }
        _ => return Err("missing call target after `action call`".to_string()),
    }

    while matches!(tokens.get(i), Some(Tok::Dot)) {
        i += 1;
        match tokens.get(i) {
            Some(Tok::Word(word)) => {
                parts.push(word.clone());
                i += 1;
            }
            _ => return Err("invalid call target path after `.`".to_string()),
        }
    }

    if parts.len() < 2 {
        return Err("call target must follow `Alias.member`".to_string());
    }

    if !matches!(tokens.get(i), Some(Tok::Semi)) {
        return Err("`action call` must terminate with `;`".to_string());
    }

    Ok((parts.join("."), i + 1))
}

pub(in crate::analyzer) fn parse_set_assignment(
    tokens: &[Tok],
    start: usize,
) -> Result<
    (
        String,
        LogicSetValueHint,
        Vec<LogicSetOperand>,
        Vec<LogicSetOperator>,
        usize,
    ),
    String,
> {
    let target = match tokens.get(start) {
        Some(Tok::Word(word)) => {
            if !is_lower_camel_case(word) {
                return Err(format!(
                    "state field target `{word}` in `action set` must be lowerCamelCase"
                ));
            }
            word.clone()
        }
        _ => return Err("missing state field target after `action set`".to_string()),
    };

    if !matches!(tokens.get(start + 1), Some(Tok::Eq)) {
        return Err(format!("`action set {target}` must define value with `=`"));
    }

    let mut i = start + 2;
    let expr_start = i;
    while !matches!(tokens.get(i), Some(Tok::Semi) | Some(Tok::RBrace) | None) {
        if matches!(tokens.get(i), Some(Tok::Word(word)) if word == "action") {
            return Err(format!("`action set {target}` must terminate with `;`"));
        }
        i += 1;
    }

    if !matches!(tokens.get(i), Some(Tok::Semi)) {
        return Err(format!("`action set {target}` must terminate with `;`"));
    }
    if i == expr_start {
        return Err(format!(
            "`action set {target}` must define value expression"
        ));
    }

    let expr_tokens = &tokens[expr_start..i];
    let value_hint = derive_set_value_hint(expr_tokens);
    let operands = collect_set_operands(expr_tokens);
    let operators = collect_set_operators(expr_tokens);

    Ok((target, value_hint, operands, operators, i + 1))
}

pub(in crate::analyzer) fn derive_set_value_hint(tokens: &[Tok]) -> LogicSetValueHint {
    if tokens.len() == 1 {
        match &tokens[0] {
            Tok::Word(word) if word == "true" || word == "false" => {
                return LogicSetValueHint::BoolLiteral;
            }
            Tok::StringLit => return LogicSetValueHint::StringLiteral,
            Tok::Number(number) if number.contains('.') => return LogicSetValueHint::FloatLiteral,
            Tok::Number(_) => return LogicSetValueHint::IntLiteral,
            _ => {}
        }
    }
    LogicSetValueHint::Expression
}

pub(in crate::analyzer) fn collect_set_operands(tokens: &[Tok]) -> Vec<LogicSetOperand> {
    let mut out = Vec::new();
    for token in tokens {
        match token {
            Tok::Word(word) if word == "true" || word == "false" => {
                out.push(LogicSetOperand::BoolLiteral)
            }
            Tok::Word(word) => out.push(LogicSetOperand::StateRef(word.clone())),
            Tok::StringLit => out.push(LogicSetOperand::StringLiteral),
            Tok::Number(number) if number.contains('.') => out.push(LogicSetOperand::FloatLiteral),
            Tok::Number(_) => out.push(LogicSetOperand::IntLiteral),
            _ => {}
        }
    }
    out
}

pub(in crate::analyzer) fn collect_set_operators(tokens: &[Tok]) -> Vec<LogicSetOperator> {
    let mut out = Vec::new();
    for token in tokens {
        match token {
            Tok::Plus => out.push(LogicSetOperator::Add),
            Tok::Minus => out.push(LogicSetOperator::Sub),
            Tok::Star => out.push(LogicSetOperator::Mul),
            Tok::Slash => out.push(LogicSetOperator::Div),
            Tok::Percent => out.push(LogicSetOperator::Mod),
            Tok::EqEq => out.push(LogicSetOperator::Eq),
            Tok::NotEq => out.push(LogicSetOperator::NotEq),
            Tok::Lt => out.push(LogicSetOperator::Lt),
            Tok::LtEq => out.push(LogicSetOperator::LtEq),
            Tok::Gt => out.push(LogicSetOperator::Gt),
            Tok::GtEq => out.push(LogicSetOperator::GtEq),
            Tok::AndAnd => out.push(LogicSetOperator::And),
            Tok::OrOr => out.push(LogicSetOperator::Or),
            _ => {}
        }
    }
    out
}
