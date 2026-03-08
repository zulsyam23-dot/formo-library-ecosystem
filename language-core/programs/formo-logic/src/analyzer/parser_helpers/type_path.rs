use super::super::*;

pub(in crate::analyzer) fn parse_type_path(tokens: &[Tok], start: usize) -> Option<usize> {
    let mut i = start;
    let mut expect_word = true;

    while i < tokens.len() {
        match tokens.get(i) {
            Some(Tok::Word(_)) if expect_word => {
                expect_word = false;
                i += 1;
            }
            Some(Tok::Dot) if !expect_word => {
                expect_word = true;
                i += 1;
            }
            _ => break,
        }
    }

    if i == start || expect_word {
        None
    } else {
        Some(i)
    }
}

pub(in crate::analyzer) fn render_type_path(tokens: &[Tok], start: usize, end: usize) -> String {
    let mut out = String::new();
    for token in &tokens[start..end] {
        match token {
            Tok::Word(word) => out.push_str(word),
            Tok::Dot => out.push('.'),
            _ => {}
        }
    }
    out
}

pub(in crate::analyzer) fn consume_balanced_block(
    tokens: &[Tok],
    open_idx: usize,
) -> Result<usize, String> {
    if !matches!(tokens.get(open_idx), Some(Tok::LBrace)) {
        return Err("internal parser error: expected `{`".to_string());
    }

    let mut depth = 1usize;
    let mut i = open_idx + 1;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Tok::LBrace) => depth += 1,
            Some(Tok::RBrace) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }

    Err("unterminated block body".to_string())
}

pub(in crate::analyzer) fn contains_action_keyword(
    tokens: &[Tok],
    start: usize,
    end: usize,
) -> bool {
    tokens
        .get(start..end)
        .unwrap_or(&[])
        .iter()
        .any(|token| matches!(token, Tok::Word(word) if word == "action"))
}
