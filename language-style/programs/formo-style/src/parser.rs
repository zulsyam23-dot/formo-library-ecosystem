use crate::allowlist::{is_allowed_style_property, is_token_key};
use crate::diagnostics::format_style_diag;
use crate::value::parse_style_value;
use formo_ir::{IrStyle, StyleSelector, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(crate) struct StyleModule {
    pub(crate) tokens: BTreeMap<String, Value>,
    pub(crate) styles: Vec<IrStyle>,
}

pub(crate) fn parse_style_module(
    source: &str,
    file: &str,
    initial_tokens: &BTreeMap<String, Value>,
) -> Result<StyleModule, String> {
    let mut parser = StyleParser::new(source, file, initial_tokens);
    parser.parse_all()
}

struct StyleParser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    file: &'a str,
    available_tokens: BTreeMap<String, Value>,
    defined_tokens: BTreeMap<String, Value>,
    referenced_tokens: BTreeSet<String>,
    styles: Vec<IrStyle>,
}

impl<'a> StyleParser<'a> {
    fn new(source: &str, file: &'a str, initial_tokens: &BTreeMap<String, Value>) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            file,
            available_tokens: initial_tokens.clone(),
            defined_tokens: BTreeMap::new(),
            referenced_tokens: BTreeSet::new(),
            styles: Vec::new(),
        }
    }

    fn parse_all(&mut self) -> Result<StyleModule, String> {
        self.skip_ws();
        while !self.eof() {
            if self.starts_with_keyword("token") {
                self.parse_token_block()?;
            } else if self.starts_with_keyword("style") {
                self.parse_style_block()?;
            } else {
                return Err(self.format_err("expected `token` or `style`"));
            }
            self.skip_ws();
        }

        let unused_tokens = self
            .defined_tokens
            .keys()
            .filter(|key| !self.referenced_tokens.contains(*key))
            .cloned()
            .collect::<Vec<_>>();
        if !unused_tokens.is_empty() {
            return Err(self.format_err_with_code(
                "E1304",
                &format!("unused token(s): {}", unused_tokens.join(", ")),
            ));
        }

        Ok(StyleModule {
            tokens: self.defined_tokens.clone(),
            styles: self.styles.clone(),
        })
    }

    fn parse_token_block(&mut self) -> Result<(), String> {
        self.expect_keyword("token")?;
        self.skip_ws();
        self.expect_char('{')?;
        let body_raw = self.read_block_body()?;

        for segment in body_raw.split(';') {
            let piece = segment.trim();
            if piece.is_empty() {
                continue;
            }

            let (key_raw, value_raw) = piece
                .split_once('=')
                .ok_or_else(|| self.format_err(&format!("invalid token declaration `{piece}`")))?;

            let key = key_raw.trim();
            if !is_token_key(key) {
                return Err(self.format_err(&format!("invalid token key `{key}`")));
            }

            if self.available_tokens.contains_key(key) {
                return Err(self.format_err(&format!("duplicate token `{key}`")));
            }

            let value_text = value_raw.trim();
            if value_text.is_empty() {
                return Err(self.format_err(&format!("token `{key}` has empty value")));
            }

            let value = parse_style_value(
                value_text,
                &self.available_tokens,
                &mut self.referenced_tokens,
            )
            .map_err(|msg| self.format_err(&msg))?;
            self.available_tokens.insert(key.to_string(), value.clone());
            self.defined_tokens.insert(key.to_string(), value);
        }

        Ok(())
    }

    fn parse_style_block(&mut self) -> Result<(), String> {
        self.expect_keyword("style")?;
        self.skip_ws();

        let selector_raw = self.read_until_char('{')?;
        self.expect_char('{')?;
        let body_raw = self.read_block_body()?;

        let (style_id, selector) =
            parse_selector(selector_raw.trim()).map_err(|msg| self.format_err(&msg))?;
        let decls = parse_decls(
            &body_raw,
            &self.available_tokens,
            &mut self.referenced_tokens,
        )
        .map_err(|msg| self.format_err(&msg))?;

        self.styles.push(IrStyle {
            id: style_id,
            selector,
            decls,
        });

        Ok(())
    }

    fn skip_ws(&mut self) {
        loop {
            while let Some(ch) = self.peek_char() {
                if !ch.is_whitespace() && ch != '\u{feff}' {
                    break;
                }
                self.advance_char();
            }

            if self.starts_with("//") {
                self.read_until_newline();
                continue;
            }

            break;
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), String> {
        if !self.starts_with_keyword(kw) {
            return Err(self.format_err(&format!("expected keyword `{kw}`")));
        }

        for _ in kw.chars() {
            self.advance_char();
        }

        Ok(())
    }

    fn starts_with_keyword(&self, keyword: &str) -> bool {
        if !self.starts_with(keyword) {
            return false;
        }

        let end_pos = self.pos + keyword.chars().count();
        match self.chars.get(end_pos).copied() {
            Some(ch) => !(ch.is_ascii_alphanumeric() || ch == '_'),
            None => true,
        }
    }

    fn read_block_body(&mut self) -> Result<String, String> {
        let mut depth = 1usize;
        let mut out = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == '{' {
                depth += 1;
                out.push(ch);
                self.advance_char();
                continue;
            }

            if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    self.advance_char();
                    return Ok(out);
                }
                out.push(ch);
                self.advance_char();
                continue;
            }

            out.push(ch);
            self.advance_char();
        }

        Err(self.format_err("unterminated block"))
    }

    fn read_until_char(&mut self, end: char) -> Result<String, String> {
        let mut out = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == end {
                return Ok(out);
            }
            out.push(ch);
            self.advance_char();
        }

        Err(self.format_err(&format!("expected `{end}` before EOF")))
    }

    fn read_until_newline(&mut self) {
        while let Some(ch) = self.peek_char() {
            self.advance_char();
            if ch == '\n' {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                self.advance_char();
                Ok(())
            }
            Some(ch) => Err(self.format_err(&format!("expected `{expected}`, found `{ch}`"))),
            None => Err(self.format_err(&format!("expected `{expected}`, found EOF"))),
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let needed = s.chars().count();
        if self.pos + needed > self.chars.len() {
            return false;
        }

        for (offset, expected) in s.chars().enumerate() {
            if self.chars[self.pos + offset] != expected {
                return false;
            }
        }

        true
    }

    fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn format_err(&self, message: &str) -> String {
        self.format_err_with_code("E1301", message)
    }

    fn format_err_with_code(&self, code: &str, message: &str) -> String {
        format_style_diag(code, self.file, self.line, self.col, message)
    }
}

fn parse_selector(raw: &str) -> Result<(String, StyleSelector), String> {
    if raw.is_empty() {
        return Err("style selector cannot be empty".to_string());
    }

    let (component, part) = match raw.split_once(':') {
        Some((left, right)) => {
            let c = left.trim();
            let p = right.trim();
            if c.is_empty() || p.is_empty() {
                return Err(format!("invalid selector `{raw}`"));
            }
            (c.to_string(), p.to_string())
        }
        None => (raw.to_string(), "root".to_string()),
    };

    let id = if part == "root" {
        component.clone()
    } else {
        format!("{component}:{part}")
    };

    Ok((id, StyleSelector { component, part }))
}

fn parse_decls(
    raw: &str,
    available_tokens: &BTreeMap<String, Value>,
    referenced_tokens: &mut BTreeSet<String>,
) -> Result<BTreeMap<String, Value>, String> {
    let mut out = BTreeMap::new();

    for segment in raw.split(';') {
        let piece = segment.trim();
        if piece.is_empty() {
            continue;
        }

        let (key_raw, value_raw) = piece
            .split_once(':')
            .ok_or_else(|| format!("invalid declaration `{piece}`, expected `key: value;`"))?;

        let key = key_raw.trim();
        if key.is_empty() {
            return Err("style declaration key cannot be empty".to_string());
        }
        if !is_allowed_style_property(key) {
            return Err(format!("unknown style property `{key}`"));
        }

        let value_text = value_raw.trim();
        if value_text.is_empty() {
            return Err(format!("style declaration `{key}` has empty value"));
        }

        out.insert(
            key.to_string(),
            parse_style_value(value_text, available_tokens, referenced_tokens)?,
        );
    }

    Ok(out)
}
