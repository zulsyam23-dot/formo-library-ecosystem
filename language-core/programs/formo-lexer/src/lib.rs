#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: String,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexDiagnostic {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

pub fn lex(source: &str) -> Vec<Token> {
    lex_with_diagnostics(source).tokens
}

pub fn lex_with_diagnostics(source: &str) -> LexOutput {
    Lexer::new(source).run()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<LexDiagnostic>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> LexOutput {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '\u{feff}' {
                self.advance();
                continue;
            }

            if ch == '/' && self.peek_next() == Some('/') {
                self.skip_line_comment();
                continue;
            }

            if ch == '/' && self.peek_next() == Some('*') {
                self.skip_block_comment();
                continue;
            }

            if ch == '"' {
                self.lex_string();
                continue;
            }

            if is_ident_start(ch) {
                self.lex_identifier();
                continue;
            }

            if ch.is_ascii_digit() {
                self.lex_number();
                continue;
            }

            if is_symbol(ch) {
                let (line, col) = self.cursor();
                self.push_token("symbol", ch.to_string(), line, col);
                self.advance();
                continue;
            }

            let (line, col) = self.cursor();
            self.push_diag("E1000", &format!("invalid character `{ch}`"), line, col);
            self.advance();
        }

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let (start_line, start_col) = self.cursor();
        self.advance();
        self.advance();

        while let Some(ch) = self.peek() {
            if ch == '*' && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                return;
            }
            self.advance_char(ch);
        }

        self.push_diag(
            "E1001",
            "unterminated block comment",
            start_line,
            start_col,
        );
    }

    fn lex_string(&mut self) {
        let (start_line, start_col) = self.cursor();
        self.advance();
        let mut out = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                self.push_token("string", out, start_line, start_col);
                return;
            }

            if ch == '\n' {
                self.push_diag(
                    "E1002",
                    "unterminated string literal",
                    start_line,
                    start_col,
                );
                return;
            }

            if ch == '\\' {
                self.advance();
                let Some(escaped) = self.peek() else {
                    self.push_diag(
                        "E1003",
                        "unterminated escape sequence in string",
                        start_line,
                        start_col,
                    );
                    return;
                };

                let mapped = match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    other => {
                        let (line, col) = self.cursor();
                        self.push_diag(
                            "E1004",
                            &format!("unknown escape sequence `\\{other}`"),
                            line,
                            col,
                        );
                        other
                    }
                };
                out.push(mapped);
                self.advance();
                continue;
            }

            out.push(ch);
            self.advance();
        }

        self.push_diag(
            "E1002",
            "unterminated string literal",
            start_line,
            start_col,
        );
    }

    fn lex_identifier(&mut self) {
        let (line, col) = self.cursor();
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if !is_ident_continue(ch) {
                break;
            }
            ident.push(ch);
            self.advance();
        }

        let kind = if ident == "true" || ident == "false" {
            "bool"
        } else {
            "ident"
        };
        self.push_token(kind, ident, line, col);
    }

    fn lex_number(&mut self) {
        let (line, col) = self.cursor();
        let mut num = String::new();

        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            num.push(ch);
            self.advance();
        }

        if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            num.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if !ch.is_ascii_digit() {
                    break;
                }
                num.push(ch);
                self.advance();
            }
            self.push_token("float", num, line, col);
            return;
        }

        self.push_token("int", num, line, col);
    }

    fn push_token(&mut self, kind: &str, lexeme: String, line: usize, col: usize) {
        self.tokens.push(Token {
            kind: kind.to_string(),
            lexeme,
            line,
            col,
        });
    }

    fn push_diag(&mut self, code: &str, message: &str, line: usize, col: usize) {
        self.diagnostics.push(LexDiagnostic {
            code: code.to_string(),
            message: message.to_string(),
            line,
            col,
        });
    }

    fn cursor(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.advance_char(ch);
        }
    }

    fn advance_char(&mut self, ch: char) {
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
}

fn is_symbol(ch: char) -> bool {
    matches!(
        ch,
        '<' | '>' | '/' | '=' | '(' | ')' | '{' | '}' | '[' | ']' | ':' | ';' | ',' | '.'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_handles_comments_symbols_and_basic_kinds() {
        let src = r#"
// line comment
component App() {
  <Text value="Hello"/>
  /* block
     comment */
  <For each=[1, 2.5, true] as=item/>
}
"#;
        let out = lex_with_diagnostics(src);
        assert!(out.diagnostics.is_empty(), "unexpected diagnostics");
        assert!(out.tokens.iter().any(|t| t.kind == "ident" && t.lexeme == "component"));
        assert!(out.tokens.iter().any(|t| t.kind == "string" && t.lexeme == "Hello"));
        assert!(out.tokens.iter().any(|t| t.kind == "int" && t.lexeme == "1"));
        assert!(out.tokens.iter().any(|t| t.kind == "float" && t.lexeme == "2.5"));
        assert!(out.tokens.iter().any(|t| t.kind == "bool" && t.lexeme == "true"));
    }

    #[test]
    fn lex_reports_precise_location_for_invalid_character() {
        let out = lex_with_diagnostics("component @App() {}");
        assert_eq!(out.diagnostics.len(), 1);
        let diag = &out.diagnostics[0];
        assert_eq!(diag.code, "E1000");
        assert_eq!(diag.line, 1);
        assert_eq!(diag.col, 11);
    }

    #[test]
    fn lex_reports_unterminated_block_comment() {
        let out = lex_with_diagnostics("/* abc");
        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].code, "E1001");
        assert_eq!(out.diagnostics[0].line, 1);
        assert_eq!(out.diagnostics[0].col, 1);
    }

    #[test]
    fn lex_reports_unterminated_string_literal() {
        let out = lex_with_diagnostics("\"hello");
        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].code, "E1002");
        assert_eq!(out.diagnostics[0].line, 1);
        assert_eq!(out.diagnostics[0].col, 1);
    }

    #[test]
    fn lex_reports_unknown_escape_sequence() {
        let out = lex_with_diagnostics(r#""\q""#);
        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].code, "E1004");
        assert_eq!(out.tokens.len(), 1);
        assert_eq!(out.tokens[0].kind, "string");
        assert_eq!(out.tokens[0].lexeme, "q");
    }

    #[test]
    fn lex_keeps_token_line_and_column_positions() {
        let out = lex_with_diagnostics("a\n  <Text/>");
        let text = out
            .tokens
            .iter()
            .find(|token| token.lexeme == "Text")
            .expect("Text token should exist");
        assert_eq!(text.line, 2);
        assert_eq!(text.col, 4);
    }
}
