use super::super::*;

pub(in crate::analyzer) fn tokenize(source: &str) -> Vec<Tok> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            c if c.is_whitespace() || c == '\u{feff}' => {
                i += 1;
            }
            '/' if chars.get(i + 1) == Some(&'/') => {
                i += 2;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                if i + 1 < chars.len() {
                    i += 2;
                }
            }
            '{' => {
                tokens.push(Tok::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Tok::RBrace);
                i += 1;
            }
            '(' => {
                tokens.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Tok::RParen);
                i += 1;
            }
            ';' => {
                tokens.push(Tok::Semi);
                i += 1;
            }
            ',' => {
                tokens.push(Tok::Comma);
                i += 1;
            }
            ':' => {
                tokens.push(Tok::Colon);
                i += 1;
            }
            '.' => {
                tokens.push(Tok::Dot);
                i += 1;
            }
            '+' => {
                tokens.push(Tok::Plus);
                i += 1;
            }
            '*' => {
                tokens.push(Tok::Star);
                i += 1;
            }
            '%' => {
                tokens.push(Tok::Percent);
                i += 1;
            }
            '=' if chars.get(i + 1) == Some(&'=') => {
                tokens.push(Tok::EqEq);
                i += 2;
            }
            '=' => {
                tokens.push(Tok::Eq);
                i += 1;
            }
            '!' if chars.get(i + 1) == Some(&'=') => {
                tokens.push(Tok::NotEq);
                i += 2;
            }
            '<' if chars.get(i + 1) == Some(&'=') => {
                tokens.push(Tok::LtEq);
                i += 2;
            }
            '<' => {
                tokens.push(Tok::Lt);
                i += 1;
            }
            '>' if chars.get(i + 1) == Some(&'=') => {
                tokens.push(Tok::GtEq);
                i += 2;
            }
            '>' => {
                tokens.push(Tok::Gt);
                i += 1;
            }
            '&' if chars.get(i + 1) == Some(&'&') => {
                tokens.push(Tok::AndAnd);
                i += 2;
            }
            '|' if chars.get(i + 1) == Some(&'|') => {
                tokens.push(Tok::OrOr);
                i += 2;
            }
            '-' if chars.get(i + 1) == Some(&'>') => {
                tokens.push(Tok::Arrow);
                i += 2;
            }
            '-' => {
                tokens.push(Tok::Minus);
                i += 1;
            }
            '/' => {
                tokens.push(Tok::Slash);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut out = String::new();
                while i < chars.len() {
                    if chars[i] == '"' {
                        i += 1;
                        break;
                    }
                    out.push(chars[i]);
                    i += 1;
                }
                tokens.push(Tok::StringLit);
            }
            c if c.is_ascii_digit() => {
                let mut out = String::new();
                out.push(c);
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    out.push(chars[i]);
                    i += 1;
                }
                tokens.push(Tok::Number(out));
            }
            c if is_ident_start(c) => {
                let mut out = String::new();
                out.push(c);
                i += 1;
                while i < chars.len() && is_ident_continue(chars[i]) {
                    out.push(chars[i]);
                    i += 1;
                }
                tokens.push(Tok::Word(out));
            }
            _ => {
                i += 1;
            }
        }
    }

    tokens
}
