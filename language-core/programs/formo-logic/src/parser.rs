use std::collections::BTreeSet;

use crate::analyzer::analyze_unit_body;
use crate::ast::{LogicProgram, LogicStateField, LogicUnit, LogicUnitKind, LogicUse};
use crate::parity::{event_platform_action_counts, is_symmetric_platform_actions};
use crate::utils::{is_ident_continue, is_ident_start, starts_uppercase};
use crate::validator::validate_program;

const LIBRARY_SCHEME: &str = "lib://";

pub(crate) fn parse(source: &str) -> Result<LogicProgram, String> {
    if source.trim().is_empty() {
        return Err("input source is empty".to_string());
    }
    let mut parser = Parser::new(source);
    parser.parse_program()
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            source,
        }
    }

    fn parse_program(&mut self) -> Result<LogicProgram, String> {
        self.skip_ws()?;
        let module = self.parse_module()?;
        let mut uses = Vec::new();
        let mut units = Vec::new();

        loop {
            self.skip_ws()?;
            if self.eof() {
                break;
            }
            if self.starts_with_keyword("use") {
                uses.push(self.parse_use()?);
                continue;
            }
            if let Some(kind) = self.peek_unit_kind() {
                units.push(self.parse_unit(kind)?);
                continue;
            }
            return self.err("expected `use`, `logic`, `service`, `contract`, or `adapter`");
        }

        validate_program(&module, &uses, &units)?;

        Ok(LogicProgram {
            raw: self.source.to_string(),
            module,
            uses,
            units,
        })
    }

    fn parse_module(&mut self) -> Result<String, String> {
        self.expect_keyword("module")?;
        self.skip_ws()?;
        let name = self.parse_identifier()?;
        if !starts_uppercase(&name) {
            return self.err("module name must be PascalCase");
        }
        self.skip_ws()?;
        self.expect_char(';')?;
        Ok(name)
    }

    fn parse_use(&mut self) -> Result<LogicUse, String> {
        let (line, col) = self.cursor();
        self.expect_keyword("use")?;
        self.skip_ws()?;
        let path = self.parse_string_literal()?;
        if !is_valid_use_path(&path) {
            return self.err(
                "use path must target `.fl` file; library path must be `lib://<library>/<module>.fl`",
            );
        }
        self.skip_ws()?;
        self.expect_keyword("as")?;
        self.skip_ws()?;
        let alias = self.parse_identifier()?;
        if !starts_uppercase(&alias) {
            return self.err("use alias must be PascalCase");
        }
        self.skip_ws()?;
        self.expect_char(';')?;
        Ok(LogicUse {
            path,
            alias,
            line,
            col,
        })
    }

    fn parse_unit(&mut self, kind: LogicUnitKind) -> Result<LogicUnit, String> {
        let (line, col) = self.cursor();
        self.expect_keyword(kind.as_str())?;
        self.skip_ws()?;
        let name = self.parse_identifier()?;
        if !starts_uppercase(&name) {
            return self.err(&format!("{} name must be PascalCase", kind.as_str()));
        }
        self.skip_ws()?;
        self.expect_char('{')?;
        let body = self.read_balanced_block()?;
        let analysis = analyze_unit_body(&body)?;
        let events = analysis.events;
        if events.is_empty() {
            return self.err(&format!(
                "{} `{}` must declare at least one `event`",
                kind.as_str(),
                name
            ));
        }

        let mut platform_set = BTreeSet::new();
        for event in &events {
            for action in &event.actions {
                match action.scope {
                    crate::ast::LogicScope::Web => {
                        platform_set.insert("web".to_string());
                    }
                    crate::ast::LogicScope::Desktop => {
                        platform_set.insert("desktop".to_string());
                    }
                    crate::ast::LogicScope::Global => {}
                }
            }
        }
        let platforms = platform_set.into_iter().collect::<Vec<_>>();
        let parity_ready = events.iter().all(|event| {
            let (web_actions, desktop_actions) = event_platform_action_counts(event);
            is_symmetric_platform_actions(web_actions, desktop_actions)
        });

        Ok(LogicUnit {
            kind,
            name,
            state_fields: analysis
                .state_fields
                .iter()
                .map(|field| LogicStateField {
                    name: field.name.clone(),
                    ty: field.ty.clone(),
                })
                .collect(),
            events,
            platforms,
            parity_ready,
            state_field_count: analysis.state_fields.len(),
            typed_state_field_count: analysis
                .state_fields
                .iter()
                .filter(|state| !state.ty.trim().is_empty())
                .count(),
            function_count: analysis.functions.len(),
            typed_function_count: analysis
                .functions
                .iter()
                .filter(|f| f.param_count == f.typed_param_count)
                .count(),
            returning_function_count: analysis
                .functions
                .iter()
                .filter(|f| f.has_return_type)
                .count(),
            enum_count: analysis.enums.len(),
            enum_variant_count: analysis.enums.iter().map(|e| e.variant_count).sum(),
            struct_count: analysis.structs.len(),
            typed_struct_count: analysis
                .structs
                .iter()
                .filter(|s| s.field_count == s.typed_field_count)
                .count(),
            struct_field_count: analysis.structs.iter().map(|s| s.field_count).sum(),
            type_alias_count: analysis.type_aliases.len(),
            qualified_type_alias_count: analysis
                .type_aliases
                .iter()
                .filter(|t| t.target.contains('.'))
                .count(),
            line,
            col,
        })
    }

    fn read_balanced_block(&mut self) -> Result<String, String> {
        let mut out = String::new();
        let mut level = 1usize;
        while let Some(ch) = self.peek_char() {
            if ch == '{' {
                level += 1;
                out.push(ch);
                self.advance_char();
                continue;
            }
            if ch == '}' {
                level -= 1;
                self.advance_char();
                if level == 0 {
                    return Ok(out);
                }
                out.push('}');
                continue;
            }
            out.push(ch);
            self.advance_char();
        }
        self.err("unterminated block body")
    }

    fn parse_string_literal(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.advance_char();
                return Ok(out);
            }
            out.push(ch);
            self.advance_char();
        }
        self.err("unterminated string literal")
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let first = self
            .peek_char()
            .ok_or_else(|| self.format_err("expected identifier"))?;
        if !is_ident_start(first) {
            return self.err("identifier must start with alphabetic character or `_`");
        }
        let mut ident = String::new();
        ident.push(first);
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if !is_ident_continue(ch) {
                break;
            }
            ident.push(ch);
            self.advance_char();
        }
        Ok(ident)
    }

    fn skip_ws(&mut self) -> Result<(), String> {
        loop {
            while let Some(ch) = self.peek_char() {
                if !ch.is_whitespace() && ch != '\u{feff}' {
                    break;
                }
                self.advance_char();
            }
            if self.starts_with("//") {
                while let Some(ch) = self.peek_char() {
                    self.advance_char();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            if self.starts_with("/*") {
                let (line, col) = self.cursor();
                self.advance_char();
                self.advance_char();
                let mut closed = false;
                while !self.eof() {
                    if self.starts_with("*/") {
                        self.advance_char();
                        self.advance_char();
                        closed = true;
                        break;
                    }
                    self.advance_char();
                }
                if !closed {
                    return Err(format!("unterminated block comment at {line}:{col}"));
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn peek_unit_kind(&self) -> Option<LogicUnitKind> {
        if self.starts_with_keyword("logic") {
            Some(LogicUnitKind::Logic)
        } else if self.starts_with_keyword("service") {
            Some(LogicUnitKind::Service)
        } else if self.starts_with_keyword("contract") {
            Some(LogicUnitKind::Contract)
        } else if self.starts_with_keyword("adapter") {
            Some(LogicUnitKind::Adapter)
        } else {
            None
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if !self.starts_with_keyword(keyword) {
            return self.err(&format!("expected keyword `{keyword}`"));
        }
        for _ in keyword.chars() {
            self.advance_char();
        }
        Ok(())
    }

    fn starts_with_keyword(&self, keyword: &str) -> bool {
        if !self.starts_with(keyword) {
            return false;
        }
        let end = self.pos + keyword.chars().count();
        match self.chars.get(end).copied() {
            Some(ch) => !is_ident_continue(ch),
            None => true,
        }
    }

    fn starts_with(&self, text: &str) -> bool {
        let need = text.chars().count();
        if self.pos + need > self.chars.len() {
            return false;
        }
        for (i, ch) in text.chars().enumerate() {
            if self.chars[self.pos + i] != ch {
                return false;
            }
        }
        true
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                self.advance_char();
                Ok(())
            }
            Some(ch) => self.err(&format!("expected `{expected}`, found `{ch}`")),
            None => self.err(&format!("expected `{expected}`, found EOF")),
        }
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

    fn cursor(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn format_err(&self, message: &str) -> String {
        format!("{message} at {}:{}", self.line, self.col)
    }

    fn err<T>(&self, message: &str) -> Result<T, String> {
        Err(self.format_err(message))
    }
}

fn is_valid_use_path(path: &str) -> bool {
    if !path.ends_with(".fl") {
        return false;
    }
    if let Some(raw) = path.strip_prefix(LIBRARY_SCHEME) {
        if raw.trim().is_empty() {
            return false;
        }
        let Some((library_name, module_rel_path)) = raw.split_once('/') else {
            return false;
        };
        if library_name.trim().is_empty() || module_rel_path.trim().is_empty() {
            return false;
        }
        if module_rel_path.ends_with('/') || module_rel_path.starts_with('/') {
            return false;
        }
        return true;
    }
    !path.contains("://")
}
