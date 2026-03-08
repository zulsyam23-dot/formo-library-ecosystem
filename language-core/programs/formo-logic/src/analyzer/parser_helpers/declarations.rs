use super::super::*;

pub(in crate::analyzer) fn parse_struct_decl(
    tokens: &[Tok],
    start: usize,
) -> Result<(LogicStructDecl, usize), String> {
    if !matches!(tokens.get(start), Some(Tok::Word(word)) if word == "struct") {
        return Err("internal parser error: expected `struct` declaration".to_string());
    }

    let name = match tokens.get(start + 1) {
        Some(Tok::Word(name)) => {
            if !starts_uppercase(name) {
                return Err(format!("struct name `{name}` must be PascalCase"));
            }
            name.clone()
        }
        _ => return Err("missing struct name after `struct`".to_string()),
    };

    if !matches!(tokens.get(start + 2), Some(Tok::LBrace)) {
        return Err(format!("struct `{name}` must start body with `{{`"));
    }

    let mut i = start + 3;
    let mut field_count = 0usize;
    let mut typed_field_count = 0usize;

    loop {
        match tokens.get(i) {
            Some(Tok::RBrace) => {
                return Ok((
                    LogicStructDecl {
                        name,
                        field_count,
                        typed_field_count,
                    },
                    i + 1,
                ));
            }
            Some(Tok::Word(field_name)) => {
                let field_name = field_name.clone();
                i += 1;
                if !matches!(tokens.get(i), Some(Tok::Colon)) {
                    return Err(format!(
                        "struct field `{field_name}` must declare type with `:`"
                    ));
                }
                i += 1;
                let Some(type_end) = parse_type_path(tokens, i) else {
                    return Err(format!(
                        "struct field `{field_name}` must declare type with `:`"
                    ));
                };
                i = type_end;
                field_count += 1;
                typed_field_count += 1;

                match tokens.get(i) {
                    Some(Tok::Comma) => i += 1,
                    Some(Tok::RBrace) => {}
                    Some(other) => {
                        return Err(format!(
                            "invalid token `{other:?}` in struct `{}`; expected `,` or `}}`",
                            name
                        ));
                    }
                    None => return Err(format!("unterminated struct `{name}`")),
                }
            }
            Some(Tok::Comma) => {
                i += 1;
            }
            Some(other) => {
                return Err(format!(
                    "invalid token `{other:?}` in struct `{}`; expected field name",
                    name
                ));
            }
            None => return Err(format!("unterminated struct `{name}`")),
        }
    }
}

pub(in crate::analyzer) fn parse_type_alias_decl(
    tokens: &[Tok],
    start: usize,
) -> Result<(LogicTypeAliasDecl, usize), String> {
    if !matches!(tokens.get(start), Some(Tok::Word(word)) if word == "type") {
        return Err("internal parser error: expected `type` alias declaration".to_string());
    }

    let name = match tokens.get(start + 1) {
        Some(Tok::Word(name)) => {
            if !starts_uppercase(name) {
                return Err(format!("type alias name `{name}` must be PascalCase"));
            }
            name.clone()
        }
        _ => return Err("missing type alias name after `type`".to_string()),
    };

    if !matches!(tokens.get(start + 2), Some(Tok::Eq)) {
        return Err(format!("type alias `{name}` must define target with `=`"));
    }

    let target_start = start + 3;
    let Some(target_end) = parse_type_path(tokens, target_start) else {
        return Err(format!("type alias `{name}` must define target type"));
    };

    if !matches!(tokens.get(target_end), Some(Tok::Semi)) {
        return Err(format!("type alias `{name}` must terminate with `;`"));
    }

    Ok((
        LogicTypeAliasDecl {
            name,
            target: render_type_path(tokens, target_start, target_end),
        },
        target_end + 1,
    ))
}

pub(in crate::analyzer) fn parse_function_decl(
    tokens: &[Tok],
    start: usize,
) -> Result<(LogicFunctionDecl, usize), String> {
    if !matches!(tokens.get(start), Some(Tok::Word(word)) if word == "function") {
        return Err("internal parser error: expected `function` declaration".to_string());
    }

    let name = match tokens.get(start + 1) {
        Some(Tok::Word(name)) => {
            if !is_lower_camel_case(name) {
                return Err(format!("function name `{name}` must be lowerCamelCase"));
            }
            name.clone()
        }
        _ => return Err("missing function name after `function`".to_string()),
    };

    if !matches!(tokens.get(start + 2), Some(Tok::LParen)) {
        return Err(format!(
            "function `{name}` must declare parameters with `(...)`"
        ));
    }

    let mut i = start + 3;
    let mut param_count = 0usize;
    let mut typed_param_count = 0usize;

    while i < tokens.len() {
        match tokens.get(i) {
            Some(Tok::RParen) => {
                i += 1;
                break;
            }
            Some(Tok::Word(param_name)) => {
                let param_name = param_name.clone();
                i += 1;
                if !matches!(tokens.get(i), Some(Tok::Colon)) {
                    return Err(format!(
                        "function parameter `{param_name}` must declare type with `:`"
                    ));
                }
                i += 1;
                let Some(type_end) = parse_type_path(tokens, i) else {
                    return Err(format!(
                        "function parameter `{param_name}` must declare type with `:`"
                    ));
                };
                i = type_end;
                param_count += 1;
                typed_param_count += 1;

                match tokens.get(i) {
                    Some(Tok::Comma) => i += 1,
                    Some(Tok::RParen) => {
                        i += 1;
                        break;
                    }
                    Some(other) => {
                        return Err(format!(
                            "invalid token `{other:?}` in function `{name}` params; expected `,` or `)`"
                        ));
                    }
                    None => {
                        return Err(format!("unterminated parameter list in function `{name}`"))
                    }
                }
            }
            Some(Tok::Comma) => {
                i += 1;
            }
            Some(other) => {
                return Err(format!(
                    "invalid token `{other:?}` in function `{name}` params"
                ));
            }
            None => return Err(format!("unterminated parameter list in function `{name}`")),
        }
    }

    let mut has_return_type = false;
    if matches!(tokens.get(i), Some(Tok::Arrow)) {
        has_return_type = true;
        i += 1;
        let Some(type_end) = parse_type_path(tokens, i) else {
            return Err(format!(
                "function `{name}` return type must be declared after `->`"
            ));
        };
        i = type_end;
    }

    if !matches!(tokens.get(i), Some(Tok::LBrace)) {
        return Err(format!(
            "function `{name}` must define body with `{{ ... }}`"
        ));
    }

    let body_end = consume_balanced_block(tokens, i)?;
    if contains_action_keyword(tokens, i + 1, body_end.saturating_sub(1)) {
        return Err(format!(
            "function `{name}` body must not contain `action` keyword"
        ));
    }

    Ok((
        LogicFunctionDecl {
            name,
            param_count,
            typed_param_count,
            has_return_type,
        },
        body_end,
    ))
}

pub(in crate::analyzer) fn parse_enum_decl(
    tokens: &[Tok],
    start: usize,
) -> Result<(LogicEnumDecl, usize), String> {
    if !matches!(tokens.get(start), Some(Tok::Word(word)) if word == "enum") {
        return Err("internal parser error: expected `enum` declaration".to_string());
    }

    let name = match tokens.get(start + 1) {
        Some(Tok::Word(name)) => {
            if !starts_uppercase(name) {
                return Err(format!("enum name `{name}` must be PascalCase"));
            }
            name.clone()
        }
        _ => return Err("missing enum name after `enum`".to_string()),
    };

    if !matches!(tokens.get(start + 2), Some(Tok::LBrace)) {
        return Err(format!("enum `{name}` must start body with `{{`"));
    }

    let mut i = start + 3;
    let mut variant_count = 0usize;
    loop {
        match tokens.get(i) {
            Some(Tok::RBrace) => {
                return Ok((
                    LogicEnumDecl {
                        name,
                        variant_count,
                    },
                    i + 1,
                ));
            }
            Some(Tok::Word(_variant)) => {
                variant_count += 1;
                i += 1;
                match tokens.get(i) {
                    Some(Tok::Comma) => i += 1,
                    Some(Tok::RBrace) => {}
                    Some(other) => {
                        return Err(format!(
                            "invalid token `{other:?}` in enum `{}`; expected `,` or `}}`",
                            name
                        ));
                    }
                    None => return Err(format!("unterminated enum `{name}`")),
                }
            }
            Some(Tok::Comma) => {
                i += 1;
            }
            Some(other) => {
                return Err(format!(
                    "invalid token `{other:?}` in enum `{}`; expected variant name",
                    name
                ));
            }
            None => return Err(format!("unterminated enum `{name}`")),
        }
    }
}
