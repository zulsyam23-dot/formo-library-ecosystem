use super::super::*;

enum SetExprStackToken {
    Operator(LogicSetOperator),
    LParen,
}

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
        Vec<LogicSetExprToken>,
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
    let set_expression_rpn = collect_set_expression_rpn(expr_tokens)
        .map_err(|reason| format!("invalid `action set {target}` expression: {reason}"))?;

    Ok((
        target,
        value_hint,
        operands,
        operators,
        set_expression_rpn,
        i + 1,
    ))
}

pub(in crate::analyzer) fn derive_set_value_hint(tokens: &[Tok]) -> LogicSetValueHint {
    if tokens.len() == 1 {
        match &tokens[0] {
            Tok::Word(word) if word == "true" || word == "false" => {
                return LogicSetValueHint::BoolLiteral;
            }
            Tok::StringLit(_) => return LogicSetValueHint::StringLiteral,
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
        if let Some(operand) = token_to_set_operand(token) {
            out.push(operand);
        }
    }
    out
}

pub(in crate::analyzer) fn collect_set_operators(tokens: &[Tok]) -> Vec<LogicSetOperator> {
    let mut out = Vec::new();
    for token in tokens {
        if let Some(operator) = token_to_set_operator(token) {
            out.push(operator);
        }
    }
    out
}

pub(in crate::analyzer) fn collect_set_expression_rpn(
    tokens: &[Tok],
) -> Result<Vec<LogicSetExprToken>, String> {
    if tokens.is_empty() {
        return Err("expression must not be empty".to_string());
    }

    let mut output = Vec::new();
    let mut stack = Vec::new();
    let mut expect_operand = true;

    for token in tokens {
        if let Some(operand) = token_to_set_operand(token) {
            if !expect_operand {
                return Err("missing operator between operands".to_string());
            }
            output.push(LogicSetExprToken::Operand(operand));
            expect_operand = false;
            continue;
        }

        if let Some(operator) = token_to_set_operator(token) {
            if expect_operand {
                return Err(format!(
                    "operator `{}` requires left operand",
                    set_operator_symbol(&operator)
                ));
            }

            while let Some(SetExprStackToken::Operator(top_operator)) = stack.last() {
                if set_operator_precedence(top_operator) < set_operator_precedence(&operator) {
                    break;
                }
                let popped = match stack.pop() {
                    Some(SetExprStackToken::Operator(op)) => op,
                    _ => unreachable!("set expression operator stack is consistent"),
                };
                output.push(LogicSetExprToken::Operator(popped));
            }

            stack.push(SetExprStackToken::Operator(operator));
            expect_operand = true;
            continue;
        }

        match token {
            Tok::LParen => {
                if !expect_operand {
                    return Err("missing operator before `(`".to_string());
                }
                stack.push(SetExprStackToken::LParen);
            }
            Tok::RParen => {
                if expect_operand {
                    return Err("missing operand before `)`".to_string());
                }
                let mut found_lparen = false;
                while let Some(entry) = stack.pop() {
                    match entry {
                        SetExprStackToken::Operator(operator) => {
                            output.push(LogicSetExprToken::Operator(operator));
                        }
                        SetExprStackToken::LParen => {
                            found_lparen = true;
                            break;
                        }
                    }
                }
                if !found_lparen {
                    return Err("unbalanced `)` in expression".to_string());
                }
                expect_operand = false;
            }
            other => {
                return Err(format!(
                    "unsupported token `{}` in expression",
                    token_label(other)
                ));
            }
        }
    }

    if expect_operand {
        return Err("expression cannot end with operator".to_string());
    }

    while let Some(entry) = stack.pop() {
        match entry {
            SetExprStackToken::Operator(operator) => {
                output.push(LogicSetExprToken::Operator(operator));
            }
            SetExprStackToken::LParen => return Err("unbalanced `(` in expression".to_string()),
        }
    }

    Ok(output)
}

fn token_to_set_operand(token: &Tok) -> Option<LogicSetOperand> {
    match token {
        Tok::Word(word) if word == "true" || word == "false" => {
            Some(LogicSetOperand::BoolLiteral(word == "true"))
        }
        Tok::Word(word) => Some(LogicSetOperand::StateRef(word.clone())),
        Tok::StringLit(value) => Some(LogicSetOperand::StringLiteral(value.clone())),
        Tok::Number(number) if number.contains('.') => {
            Some(LogicSetOperand::FloatLiteral(number.clone()))
        }
        Tok::Number(number) => Some(LogicSetOperand::IntLiteral(number.clone())),
        _ => None,
    }
}

fn token_to_set_operator(token: &Tok) -> Option<LogicSetOperator> {
    match token {
        Tok::Plus => Some(LogicSetOperator::Add),
        Tok::Minus => Some(LogicSetOperator::Sub),
        Tok::Star => Some(LogicSetOperator::Mul),
        Tok::Slash => Some(LogicSetOperator::Div),
        Tok::Percent => Some(LogicSetOperator::Mod),
        Tok::EqEq => Some(LogicSetOperator::Eq),
        Tok::NotEq => Some(LogicSetOperator::NotEq),
        Tok::Lt => Some(LogicSetOperator::Lt),
        Tok::LtEq => Some(LogicSetOperator::LtEq),
        Tok::Gt => Some(LogicSetOperator::Gt),
        Tok::GtEq => Some(LogicSetOperator::GtEq),
        Tok::AndAnd => Some(LogicSetOperator::And),
        Tok::OrOr => Some(LogicSetOperator::Or),
        _ => None,
    }
}

fn set_operator_precedence(operator: &LogicSetOperator) -> usize {
    match operator {
        LogicSetOperator::Or => 1,
        LogicSetOperator::And => 2,
        LogicSetOperator::Eq | LogicSetOperator::NotEq => 3,
        LogicSetOperator::Lt
        | LogicSetOperator::LtEq
        | LogicSetOperator::Gt
        | LogicSetOperator::GtEq => 4,
        LogicSetOperator::Add | LogicSetOperator::Sub => 5,
        LogicSetOperator::Mul | LogicSetOperator::Div | LogicSetOperator::Mod => 6,
    }
}

fn set_operator_symbol(operator: &LogicSetOperator) -> &'static str {
    match operator {
        LogicSetOperator::Add => "+",
        LogicSetOperator::Sub => "-",
        LogicSetOperator::Mul => "*",
        LogicSetOperator::Div => "/",
        LogicSetOperator::Mod => "%",
        LogicSetOperator::Eq => "==",
        LogicSetOperator::NotEq => "!=",
        LogicSetOperator::Lt => "<",
        LogicSetOperator::LtEq => "<=",
        LogicSetOperator::Gt => ">",
        LogicSetOperator::GtEq => ">=",
        LogicSetOperator::And => "&&",
        LogicSetOperator::Or => "||",
    }
}

fn token_label(token: &Tok) -> &'static str {
    match token {
        Tok::Word(_) => "word",
        Tok::Number(_) => "number",
        Tok::StringLit(_) => "string",
        Tok::Dot => ".",
        Tok::Comma => ",",
        Tok::Colon => ":",
        Tok::Eq => "=",
        Tok::EqEq => "==",
        Tok::NotEq => "!=",
        Tok::Lt => "<",
        Tok::LtEq => "<=",
        Tok::Gt => ">",
        Tok::GtEq => ">=",
        Tok::Plus => "+",
        Tok::Minus => "-",
        Tok::Star => "*",
        Tok::Slash => "/",
        Tok::Percent => "%",
        Tok::AndAnd => "&&",
        Tok::OrOr => "||",
        Tok::Arrow => "->",
        Tok::LBrace => "{",
        Tok::RBrace => "}",
        Tok::LParen => "(",
        Tok::RParen => ")",
        Tok::Semi => ";",
    }
}
