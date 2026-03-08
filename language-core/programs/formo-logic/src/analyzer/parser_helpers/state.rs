use super::super::*;

pub(in crate::analyzer) fn parse_state_block(
    tokens: &[Tok],
    start: usize,
) -> Result<(Vec<LogicStateFieldDecl>, usize), String> {
    if !matches!(tokens.get(start), Some(Tok::Word(word)) if word == "state") {
        return Err("internal parser error: expected `state` block".to_string());
    }
    if !matches!(tokens.get(start + 1), Some(Tok::LBrace)) {
        return Err("state block must start with `{`".to_string());
    }

    let mut fields = Vec::new();
    let mut i = start + 2;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Tok::RBrace) => return Ok((fields, i + 1)),
            Some(Tok::Word(field_name)) => {
                let name = field_name.clone();
                if !is_lower_camel_case(&name) {
                    return Err(format!("state field `{name}` must be lowerCamelCase"));
                }
                i += 1;

                if !matches!(tokens.get(i), Some(Tok::Colon)) {
                    return Err(format!("state field `{name}` must declare type with `:`"));
                }
                i += 1;

                let Some(type_end) = parse_type_path(tokens, i) else {
                    return Err(format!("state field `{name}` must declare type with `:`"));
                };
                let ty = render_type_path(tokens, i, type_end);
                i = type_end;

                if !matches!(tokens.get(i), Some(Tok::Eq)) {
                    return Err(format!(
                        "state field `{name}` must define initializer with `=`"
                    ));
                }
                i += 1;

                let init_start = i;
                while !matches!(tokens.get(i), Some(Tok::Semi) | Some(Tok::RBrace) | None) {
                    i += 1;
                }

                if matches!(tokens.get(i), Some(Tok::RBrace) | None) {
                    return Err(format!("state field `{name}` must terminate with `;`"));
                }

                let init_tokens = &tokens[init_start..i];
                validate_state_initializer(&name, &ty, init_tokens)?;

                fields.push(LogicStateFieldDecl { name, ty });
                i += 1;
            }
            Some(other) => {
                return Err(format!(
                    "invalid token in state block: expected field declaration, got `{other:?}`"
                ));
            }
            None => break,
        }
    }

    Err("unterminated `state` block".to_string())
}

pub(in crate::analyzer) fn validate_state_initializer(
    field_name: &str,
    ty: &str,
    tokens: &[Tok],
) -> Result<(), String> {
    if tokens.is_empty() {
        return Err(format!(
            "state field `{field_name}` must define initializer with `=`"
        ));
    }

    let normalized = ty.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "bool" | "boolean" => {
            if matches!(tokens, [Tok::Word(word)] if word == "true" || word == "false") {
                return Ok(());
            }
            Err(format!(
                "state field `{field_name}` with type `{ty}` must initialize with `true` or `false`"
            ))
        }
        "string" => {
            if matches!(tokens, [Tok::StringLit]) {
                Ok(())
            } else {
                Err(format!(
                    "state field `{field_name}` with type `{ty}` must initialize with string literal"
                ))
            }
        }
        "int" => {
            if matches!(tokens, [Tok::Number(number)] if !number.contains('.')) {
                Ok(())
            } else {
                Err(format!(
                    "state field `{field_name}` with type `{ty}` must initialize with integer literal"
                ))
            }
        }
        "float" | "number" => {
            if matches!(tokens, [Tok::Number(_)]) {
                Ok(())
            } else {
                Err(format!(
                    "state field `{field_name}` with type `{ty}` must initialize with numeric literal"
                ))
            }
        }
        _ => Ok(()),
    }
}
