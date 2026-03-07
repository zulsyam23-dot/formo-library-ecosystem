use formo_lexer::{lex, Token};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct AstProgram {
    pub raw: String,
    pub tokens: Vec<Token>,
    pub imports: Vec<AstImport>,
    pub components: Vec<AstComponent>,
}

#[derive(Debug, Clone)]
pub struct ParseRecovery {
    pub ast: AstProgram,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AstImport {
    pub path: String,
    pub alias: Option<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct AstParam {
    pub name: String,
    pub ty: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct AstComponent {
    pub name: String,
    pub params: Vec<AstParam>,
    pub nodes: Vec<AstNode>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub name: String,
    pub attributes: Vec<AstAttr>,
    pub children: Vec<AstNode>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct AstAttr {
    pub name: String,
    pub value: AstValue,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum AstValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Identifier(String),
    List(Vec<AstValue>),
    Object(BTreeMap<String, AstValue>),
}

pub fn parse(source: &str) -> Result<AstProgram, String> {
    if source.trim().is_empty() {
        return Err("input source is empty".to_string());
    }

    let mut parser = Parser::new(source);
    let (imports, components) = parser.parse_program()?;

    Ok(AstProgram {
        raw: source.to_string(),
        tokens: lex(source),
        imports,
        components,
    })
}

pub fn parse_with_recovery(source: &str) -> ParseRecovery {
    let mut parser = Parser::new(source);
    let mut diagnostics = Vec::new();

    let (imports, components) = if source.trim().is_empty() {
        diagnostics.push("input source is empty".to_string());
        (Vec::new(), Vec::new())
    } else {
        parser.parse_program_with_recovery(&mut diagnostics)
    };

    ParseRecovery {
        ast: AstProgram {
            raw: source.to_string(),
            tokens: lex(source),
            imports,
            components,
        },
        diagnostics,
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _source: &'a str,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _source: source,
        }
    }

    fn parse_program(&mut self) -> Result<(Vec<AstImport>, Vec<AstComponent>), String> {
        let mut imports = Vec::new();
        let mut components = Vec::new();

        self.skip_ws();
        while !self.eof() {
            if self.starts_with_keyword("import") {
                imports.push(self.parse_import()?);
            } else if self.starts_with_keyword("component") {
                components.push(self.parse_component()?);
            } else {
                return self.err("expected `import` or `component`");
            }
            self.skip_ws();
        }

        if components.is_empty() {
            return self.err("at least one component is required");
        }

        Ok((imports, components))
    }

    fn parse_program_with_recovery(
        &mut self,
        diagnostics: &mut Vec<String>,
    ) -> (Vec<AstImport>, Vec<AstComponent>) {
        let mut imports = Vec::new();
        let mut components = Vec::new();

        self.skip_ws();
        while !self.eof() {
            let start_pos = self.pos;
            if self.starts_with_keyword("import") {
                match self.parse_import() {
                    Ok(import_item) => imports.push(import_item),
                    Err(err) => {
                        diagnostics.push(err);
                        self.recover_to_next_top_level_item(start_pos);
                    }
                }
            } else if self.starts_with_keyword("component") {
                match self.parse_component() {
                    Ok(component) => components.push(component),
                    Err(err) => {
                        diagnostics.push(err);
                        self.recover_to_next_top_level_item(start_pos);
                    }
                }
            } else {
                diagnostics.push(self.format_err("expected `import` or `component`"));
                self.recover_to_next_top_level_item(start_pos);
            }
            self.skip_ws();
        }

        if components.is_empty() {
            diagnostics.push(self.format_err("at least one component is required"));
        }

        (imports, components)
    }

    fn parse_import(&mut self) -> Result<AstImport, String> {
        let (line, col) = self.cursor();
        self.expect_keyword("import")?;
        self.skip_ws();

        let path = self.parse_string_literal()?;
        self.skip_ws();

        let alias = if self.starts_with_keyword("as") {
            self.expect_keyword("as")?;
            self.skip_ws();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_ws();
        self.expect_char(';')?;

        Ok(AstImport {
            path,
            alias,
            line,
            col,
        })
    }

    fn parse_component(&mut self) -> Result<AstComponent, String> {
        let (line, col) = self.cursor();
        self.expect_keyword("component")?;
        self.skip_ws();

        let name = self.parse_identifier()?;
        self.skip_ws();
        self.expect_char('(')?;
        let params_raw = self.read_until_char(')')?;
        self.expect_char(')')?;
        let params = parse_component_params(&params_raw)
            .map_err(|msg| format!("{msg} at {}:{}", self.line, self.col))?;

        self.skip_ws();
        self.expect_char('{')?;

        let mut nodes = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_char() == Some('}') {
                self.advance_char();
                break;
            }

            nodes.push(self.parse_node()?);
        }

        Ok(AstComponent {
            name,
            params,
            nodes,
            line,
            col,
        })
    }

    fn parse_node(&mut self) -> Result<AstNode, String> {
        let (line, col) = self.cursor();
        self.expect_char('<')?;

        if self.peek_char() == Some('/') {
            return self.err("unexpected closing tag");
        }

        let name = self.parse_identifier()?;
        let mut attributes = Vec::new();

        loop {
            self.skip_ws();

            if self.starts_with("/>") {
                self.advance_char();
                self.advance_char();
                return Ok(AstNode {
                    name,
                    attributes,
                    children: Vec::new(),
                    line,
                    col,
                });
            }

            if self.peek_char() == Some('>') {
                self.advance_char();
                break;
            }

            attributes.push(self.parse_attribute()?);
        }

        let mut children = Vec::new();
        loop {
            self.skip_ws();

            if self.starts_with("</") {
                self.expect_char('<')?;
                self.expect_char('/')?;
                let close_name = self.parse_identifier()?;
                if close_name != name {
                    return self.err(&format!(
                        "closing tag mismatch: expected `</{}>` found `</{}>`",
                        name, close_name
                    ));
                }
                self.skip_ws();
                self.expect_char('>')?;
                break;
            }

            if self.peek_char() == Some('<') {
                children.push(self.parse_node()?);
                continue;
            }

            if self.eof() {
                return self.err(&format!("unexpected EOF, missing `</{}>`", name));
            }

            let raw_text = self.read_until_char('<')?;
            if !raw_text.trim().is_empty() {
                return self.err("raw text node is not supported; use `<Text value=\"...\"/>`");
            }
        }

        Ok(AstNode {
            name,
            attributes,
            children,
            line,
            col,
        })
    }

    fn parse_attribute(&mut self) -> Result<AstAttr, String> {
        let (line, col) = self.cursor();
        let name = self.parse_identifier()?;
        self.skip_ws();
        self.expect_char('=')?;
        self.skip_ws();
        let value = self.parse_value()?;

        Ok(AstAttr {
            name,
            value,
            line,
            col,
        })
    }

    fn parse_value(&mut self) -> Result<AstValue, String> {
        if self.peek_char() == Some('"') {
            return Ok(AstValue::String(self.parse_string_literal()?));
        }

        if self.peek_char() == Some('[') {
            return self.parse_list_literal();
        }

        if self.peek_char() == Some('{') {
            return self.parse_object_literal();
        }

        let token = self.read_while(|ch| {
            !ch.is_whitespace()
                && ch != '>'
                && ch != '/'
                && ch != ','
                && ch != ']'
                && ch != '}'
                && ch != ':'
        });
        if token.is_empty() {
            return self.err("expected value");
        }

        if token == "true" {
            return Ok(AstValue::Bool(true));
        }

        if token == "false" {
            return Ok(AstValue::Bool(false));
        }

        if let Ok(int_value) = token.parse::<i64>() {
            return Ok(AstValue::Int(int_value));
        }

        if let Ok(float_value) = token.parse::<f64>() {
            return Ok(AstValue::Float(float_value));
        }

        Ok(AstValue::Identifier(token))
    }

    fn parse_list_literal(&mut self) -> Result<AstValue, String> {
        self.expect_char('[')?;
        self.skip_ws();

        let mut items = Vec::new();
        if self.peek_char() == Some(']') {
            self.advance_char();
            return Ok(AstValue::List(items));
        }

        loop {
            self.skip_ws();
            items.push(self.parse_value()?);
            self.skip_ws();

            match self.peek_char() {
                Some(',') => {
                    self.advance_char();
                    self.skip_ws();
                    if self.peek_char() == Some(']') {
                        return self.err("invalid trailing comma in list literal");
                    }
                }
                Some(']') => {
                    self.advance_char();
                    break;
                }
                Some(ch) => {
                    return self.err(&format!(
                        "expected `,` or `]` in list literal, found `{ch}`"
                    ));
                }
                None => {
                    return self.err("unterminated list literal");
                }
            }
        }

        Ok(AstValue::List(items))
    }

    fn parse_object_literal(&mut self) -> Result<AstValue, String> {
        self.expect_char('{')?;
        self.skip_ws();

        let mut entries = BTreeMap::new();
        if self.peek_char() == Some('}') {
            self.advance_char();
            return Ok(AstValue::Object(entries));
        }

        loop {
            self.skip_ws();
            let key = if self.peek_char() == Some('"') {
                self.parse_string_literal()?
            } else {
                self.parse_identifier()?
            };
            self.skip_ws();
            self.expect_char(':')?;
            self.skip_ws();
            let value = self.parse_value()?;

            if entries.insert(key.clone(), value).is_some() {
                return self.err(&format!("duplicate object key `{key}`"));
            }

            self.skip_ws();
            match self.peek_char() {
                Some(',') => {
                    self.advance_char();
                    self.skip_ws();
                    if self.peek_char() == Some('}') {
                        return self.err("invalid trailing comma in object literal");
                    }
                }
                Some('}') => {
                    self.advance_char();
                    break;
                }
                Some(ch) => {
                    return self.err(&format!(
                        "expected `,` or `}}` in object literal, found `{ch}`"
                    ));
                }
                None => return self.err("unterminated object literal"),
            }
        }

        Ok(AstValue::Object(entries))
    }

    fn parse_string_literal(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.advance_char();
                return Ok(out);
            }

            if ch == '\\' {
                self.advance_char();
                let escaped = self.peek_char().ok_or_else(|| {
                    self.format_err("unterminated escape sequence in string literal")
                })?;

                let mapped = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    other => other,
                };

                out.push(mapped);
                self.advance_char();
                continue;
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

        let end_pos = self.pos + keyword.chars().count();
        match self.chars.get(end_pos).copied() {
            Some(ch) => !is_ident_continue(ch),
            None => true,
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

    fn read_until_char(&mut self, end: char) -> Result<String, String> {
        let mut out = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == end {
                return Ok(out);
            }
            out.push(ch);
            self.advance_char();
        }

        self.err(&format!("expected `{end}` before EOF"))
    }

    fn read_while<F>(&mut self, cond: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut out = String::new();
        while let Some(ch) = self.peek_char() {
            if !cond(ch) {
                break;
            }
            out.push(ch);
            self.advance_char();
        }
        out
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() && ch != '\u{feff}' {
                break;
            }
            self.advance_char();
        }
    }

    fn recover_to_next_top_level_item(&mut self, start_pos: usize) {
        if self.pos == start_pos && !self.eof() {
            self.advance_char();
        }

        while !self.eof() {
            self.skip_ws();
            if self.starts_with_keyword("import") || self.starts_with_keyword("component") {
                return;
            }
            self.advance_char();
        }
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

fn parse_component_params(raw: &str) -> Result<Vec<AstParam>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for segment in trimmed.split(',') {
        let piece = segment.trim();
        if piece.is_empty() {
            return Err("invalid trailing comma in component params".to_string());
        }

        let (name_piece, ty_piece) = match piece.split_once(':') {
            Some((left, right)) => (left.trim(), Some(right.trim())),
            None => (piece, None),
        };

        let optional = name_piece.ends_with('?');
        let base_name = if optional {
            &name_piece[..name_piece.len() - 1]
        } else {
            name_piece
        }
        .trim();

        if !is_valid_ident(base_name) {
            return Err(format!("invalid parameter name `{base_name}`"));
        }

        let ty = match ty_piece {
            Some(ty_name) if !ty_name.is_empty() => Some(ty_name.to_string()),
            Some(_) => return Err(format!("missing type for parameter `{base_name}`")),
            None => None,
        };

        out.push(AstParam {
            name: base_name.to_string(),
            ty,
            optional,
        });
    }

    Ok(out)
}

fn is_valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !is_ident_start(first) {
        return false;
    }

    chars.all(is_ident_continue)
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_component_tree() {
        let src = r#"
component App() {
  <Page>
    <Text value="Hello"/>
  </Page>
}
"#;
        let ast = parse(src).expect("parse ok");
        assert_eq!(ast.components.len(), 1);
        assert_eq!(ast.components[0].name, "App");
        assert_eq!(ast.components[0].nodes[0].name, "Page");
        assert_eq!(ast.components[0].nodes[0].children[0].name, "Text");
    }

    #[test]
    fn parse_component_params_typed_and_optional() {
        let src = r#"
component Header(title: string, subtitle?: string) {
  <Text value=title/>
}
"#;
        let ast = parse(src).expect("parse ok");
        let params = &ast.components[0].params;
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "title");
        assert_eq!(params[0].ty.as_deref(), Some("string"));
        assert!(!params[0].optional);
        assert_eq!(params[1].name, "subtitle");
        assert_eq!(params[1].ty.as_deref(), Some("string"));
        assert!(params[1].optional);
    }

    #[test]
    fn parse_list_literal_attribute() {
        let src = r#"
component App() {
  <For each=["A", "B", 3, true] as=item>
    <Text value=item/>
  </For>
}
"#;
        let ast = parse(src).expect("parse ok");
        let for_node = &ast.components[0].nodes[0];
        let each_attr = for_node
            .attributes
            .iter()
            .find(|attr| attr.name == "each")
            .expect("each attr exists");
        match &each_attr.value {
            AstValue::List(items) => assert_eq!(items.len(), 4),
            other => panic!("expected list literal, got {other:?}"),
        }
    }

    #[test]
    fn parse_object_literal_in_list() {
        let src = r#"
component App() {
  <For each=[{name: "A", active: true}, {name: "B", active: false}] as=item>
    <Text value=item.name/>
  </For>
}
"#;
        let ast = parse(src).expect("parse ok");
        let for_node = &ast.components[0].nodes[0];
        let each_attr = for_node
            .attributes
            .iter()
            .find(|attr| attr.name == "each")
            .expect("each attr exists");
        match &each_attr.value {
            AstValue::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], AstValue::Object(_)));
            }
            other => panic!("expected list literal, got {other:?}"),
        }
    }

    #[test]
    fn reject_raw_text_node() {
        let src = r#"
component App() {
  <Page>
    halo
  </Page>
}
"#;
        let err = parse(src).expect_err("raw text should fail");
        assert!(err.contains("raw text node is not supported"));
    }

    #[test]
    fn reject_closing_tag_mismatch() {
        let src = r#"
component App() {
  <Page>
    <Text value="x"></Page>
  </Text>
}
"#;
        let err = parse(src).expect_err("mismatched closing tag should fail");
        assert!(err.contains("closing tag mismatch"));
    }

    #[test]
    fn reject_list_trailing_comma() {
        let src = r#"
component App() {
  <For each=["A",] as=item>
    <Text value=item/>
  </For>
}
"#;
        let err = parse(src).expect_err("list trailing comma should fail");
        assert!(err.contains("invalid trailing comma in list literal"));
    }

    #[test]
    fn reject_object_duplicate_key() {
        let src = r#"
component App() {
  <For each=[{name: "A", name: "B"}] as=item>
    <Text value=item.name/>
  </For>
}
"#;
        let err = parse(src).expect_err("duplicate key should fail");
        assert!(err.contains("duplicate object key"));
    }

    #[test]
    fn reject_object_trailing_comma() {
        let src = r#"
component App() {
  <For each=[{name: "A",}] as=item>
    <Text value=item.name/>
  </For>
}
"#;
        let err = parse(src).expect_err("object trailing comma should fail");
        assert!(err.contains("invalid trailing comma in object literal"));
    }

    #[test]
    fn reject_empty_source() {
        let err = parse("   ").expect_err("empty source should fail");
        assert!(err.contains("input source is empty"));
    }

    #[test]
    fn reject_import_without_semicolon() {
        let src = r#"
import "views/header.fm" as Header
component App() {
  <Page/>
}
"#;
        let err = parse(src).expect_err("import without semicolon should fail");
        assert!(err.contains("expected `;`"));
    }

    #[test]
    fn reject_import_alias_with_invalid_identifier() {
        let src = r#"
import "views/header.fm" as 2Header;
component App() {
  <Page/>
}
"#;
        let err = parse(src).expect_err("invalid import alias should fail");
        assert!(err.contains("identifier must start"));
    }

    #[test]
    fn reject_component_params_trailing_comma() {
        let src = r#"
component App(title: string,) {
  <Text value=title/>
}
"#;
        let err = parse(src).expect_err("params trailing comma should fail");
        assert!(err.contains("invalid trailing comma in component params"));
    }

    #[test]
    fn reject_component_param_missing_type_after_colon() {
        let src = r#"
component App(title:) {
  <Text value="x"/>
}
"#;
        let err = parse(src).expect_err("missing type should fail");
        assert!(err.contains("missing type for parameter `title`"));
    }

    #[test]
    fn reject_component_param_invalid_name() {
        let src = r#"
component App(1title: string) {
  <Text value="x"/>
}
"#;
        let err = parse(src).expect_err("invalid param name should fail");
        assert!(err.contains("invalid parameter name"));
    }

    #[test]
    fn reject_unexpected_closing_tag_at_node_start() {
        let src = r#"
component App() {
  </Page>
}
"#;
        let err = parse(src).expect_err("unexpected closing tag should fail");
        assert!(err.contains("unexpected closing tag"));
    }

    #[test]
    fn reject_attribute_missing_value() {
        let src = r#"
component App() {
  <Text value=/>
}
"#;
        let err = parse(src).expect_err("missing attribute value should fail");
        assert!(err.contains("expected value"));
    }

    #[test]
    fn reject_unterminated_list_literal() {
        let src = r#"
component App() {
  <For each=["A", "B" as=item>
    <Text value=item/>
  </For>
}
"#;
        let err = parse(src).expect_err("unterminated list literal should fail");
        assert!(err.contains("expected `,` or `]` in list literal"));
    }

    #[test]
    fn reject_unterminated_object_literal() {
        let src = r#"
component App() {
  <For each=[{name: "A", active: true] as=item>
    <Text value=item.name/>
  </For>
}
"#;
        let err = parse(src).expect_err("unterminated object literal should fail");
        assert!(err.contains("expected `,` or `}` in object literal"));
    }

    #[test]
    fn reject_unterminated_string_literal() {
        let src = r#"
component App() {
  <Text value="hello/>
}
"#;
        let err = parse(src).expect_err("unterminated string should fail");
        assert!(err.contains("unterminated string literal"));
    }

    #[test]
    fn recovery_continues_after_invalid_component_and_parses_next_component() {
        let src = r#"
component Broken() {
  <Page>
    teks bebas
  </Page>
}

component App() {
  <Page>
    <Text value="ok"/>
  </Page>
}
"#;

        let report = parse_with_recovery(src);
        assert!(
            !report.diagnostics.is_empty(),
            "recovery should collect diagnostics"
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diag| diag.contains("raw text node is not supported")),
            "expected raw text diagnostic in recovery output"
        );
        assert_eq!(report.ast.components.len(), 1);
        assert_eq!(report.ast.components[0].name, "App");
    }

    #[test]
    fn recovery_skips_invalid_top_level_tokens_and_keeps_valid_component() {
        let src = r#"
@@@
component App() {
  <Page/>
}
"#;

        let report = parse_with_recovery(src);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diag| diag.contains("expected `import` or `component`")),
            "expected top-level token diagnostic"
        );
        assert_eq!(report.ast.components.len(), 1);
        assert_eq!(report.ast.components[0].name, "App");
    }
}
