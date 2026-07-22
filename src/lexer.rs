//! Indentation-aware lexer for TinyS.
//!
//! Produces a flat token stream that includes synthetic `Newline`, `Indent`,
//! and `Dedent` tokens so the parser can treat blocks structurally, the way a
//! Python-style layout language requires. Newlines and indentation are
//! suppressed while inside `()`/`[]`/`{}` so expressions may span lines.

use crate::diagnostics::{Span, Stage, TinysError};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Literals
    Int(i64),
    Float(f64),
    /// Inner text of a normal string literal, exactly as written (escapes kept).
    Str(String),
    /// A raw string literal captured verbatim, including the `r"..."`/`r#"..."#`.
    RawStr(String),
    Ident(String),
    /// A lifetime written as `.name` in type/generic position → Rust `'name`.
    Lifetime(String),
    /// The raw contents of an attribute `#[...]` (without the `#[` `]`).
    Attribute(String),

    // Keywords
    Def,
    Return,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    Match,
    Case,
    Struct,
    Enum,
    Impl,
    Trait,
    Mut,
    Ref,
    At,
    Move,
    Clone,
    Pub,
    And,
    Or,
    Not,
    Pass,
    As,
    With,
    Unsafe,
    Async,
    Await,
    From,
    Import,
    Void,
    True,
    False,
    Fn,

    // Punctuation / operators
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Dot,
    Arrow,     // ->
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,    // =
    EqEq,      // ==
    NotEq,     // !=
    Lt,
    Gt,
    Le,
    Ge,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    DotDot,    // ..
    DotDotEq,  // ..=
    Question,  // ?
    Pipe,      // |

    // Layout
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Tok,
    pub span: Span,
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    file: &'a str,
    tokens: Vec<Token>,
    indent_stack: Vec<usize>,
    paren_depth: i32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            file,
            tokens: Vec::new(),
            indent_stack: vec![0],
            paren_depth: 0,
        }
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn err(&self, msg: impl Into<String>) -> TinysError {
        TinysError::new(Stage::Lex, msg, self.file, self.span())
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn push(&mut self, kind: Tok, span: Span) {
        self.tokens.push(Token { kind, span });
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, TinysError> {
        self.handle_line_start()?;
        loop {
            self.skip_inline_ws();
            match self.peek() {
                None => break,
                Some('\n') => {
                    self.advance();
                    if self.paren_depth == 0 {
                        // Only emit a Newline if the line produced content.
                        if self.last_significant_is_content() {
                            self.push(Tok::Newline, self.span());
                        }
                        self.handle_line_start()?;
                    }
                }
                Some('/') if self.peek2() == Some('/') => {
                    // Comment to end of line.
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some(c) => self.lex_token(c)?,
            }
        }
        // Ensure a trailing newline separates the last statement.
        if self.last_significant_is_content() {
            self.push(Tok::Newline, self.span());
        }
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.push(Tok::Dedent, self.span());
        }
        self.push(Tok::Eof, self.span());
        Ok(self.tokens)
    }

    /// True when the most recent token is real content (not layout), so we know
    /// whether a Newline is meaningful.
    fn last_significant_is_content(&self) -> bool {
        matches!(
            self.tokens.last().map(|t| &t.kind),
            Some(k) if !matches!(k, Tok::Newline | Tok::Indent | Tok::Dedent)
        )
    }

    fn skip_inline_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Measure indentation at the start of a logical line and emit Indent/Dedent
    /// tokens. Blank lines and comment-only lines carry no indentation meaning.
    fn handle_line_start(&mut self) -> Result<(), TinysError> {
        loop {
            let mut indent = 0usize;
            while let Some(c) = self.peek() {
                match c {
                    ' ' => {
                        indent += 1;
                        self.advance();
                    }
                    '\t' => {
                        indent += 4;
                        self.advance();
                    }
                    _ => break,
                }
            }
            match self.peek() {
                None => return Ok(()),
                Some('\n') => {
                    self.advance();
                    continue;
                }
                Some('\r') => {
                    self.advance();
                    continue;
                }
                Some('/') if self.peek2() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }
                _ => {
                    let top = *self.indent_stack.last().unwrap();
                    if indent > top {
                        self.indent_stack.push(indent);
                        self.push(Tok::Indent, self.span());
                    } else if indent < top {
                        while indent < *self.indent_stack.last().unwrap() {
                            self.indent_stack.pop();
                            self.push(Tok::Dedent, self.span());
                        }
                        if indent != *self.indent_stack.last().unwrap() {
                            return Err(self.err(format!(
                                "inconsistent indentation (found {} spaces)",
                                indent
                            )));
                        }
                    }
                    return Ok(());
                }
            }
        }
    }

    fn lex_token(&mut self, c: char) -> Result<(), TinysError> {
        let start = self.span();
        // Raw strings: r"..." or r#"..."#
        if c == 'r' && matches!(self.peek2(), Some('"') | Some('#')) {
            return self.lex_raw_string(start);
        }
        if c.is_ascii_digit() {
            return self.lex_number(start);
        }
        if c.is_alphabetic() || c == '_' {
            return self.lex_ident(start);
        }
        if c == '"' {
            return self.lex_string(start);
        }
        if c == '#' {
            return self.lex_attribute(start);
        }
        self.lex_symbol(c, start)
    }

    fn lex_number(&mut self, start: Span) -> Result<(), TinysError> {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        // A single '.' followed by a digit means a fractional part; '..' is a range.
        let is_float = self.peek() == Some('.')
            && self.peek2().map(|c| c.is_ascii_digit()).unwrap_or(false);
        if is_float {
            text.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let cleaned: String = text.chars().filter(|c| *c != '_').collect();
            let value: f64 = cleaned
                .parse()
                .map_err(|_| self.err(format!("invalid float literal `{}`", text)))?;
            self.push(Tok::Float(value), start);
        } else {
            let cleaned: String = text.chars().filter(|c| *c != '_').collect();
            let value: i64 = cleaned
                .parse()
                .map_err(|_| self.err(format!("invalid integer literal `{}`", text)))?;
            self.push(Tok::Int(value), start);
        }
        Ok(())
    }

    fn lex_ident(&mut self, start: Span) -> Result<(), TinysError> {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match text.as_str() {
            "def" => Tok::Def,
            "return" => Tok::Return,
            "if" => Tok::If,
            "elif" => Tok::Elif,
            "else" => Tok::Else,
            "while" => Tok::While,
            "for" => Tok::For,
            "in" => Tok::In,
            "loop" => Tok::Loop,
            "break" => Tok::Break,
            "continue" => Tok::Continue,
            "match" => Tok::Match,
            "case" => Tok::Case,
            "struct" => Tok::Struct,
            "enum" => Tok::Enum,
            "impl" => Tok::Impl,
            "trait" => Tok::Trait,
            "mut" => Tok::Mut,
            "ref" => Tok::Ref,
            "at" => Tok::At,
            "move" => Tok::Move,
            "clone" => Tok::Clone,
            "pub" => Tok::Pub,
            "and" => Tok::And,
            "or" => Tok::Or,
            "not" => Tok::Not,
            "pass" => Tok::Pass,
            "as" => Tok::As,
            "with" => Tok::With,
            "unsafe" => Tok::Unsafe,
            "async" => Tok::Async,
            "await" => Tok::Await,
            "from" => Tok::From,
            "import" => Tok::Import,
            "void" => Tok::Void,
            "true" => Tok::True,
            "false" => Tok::False,
            "fn" => Tok::Fn,
            _ => Tok::Ident(text),
        };
        self.push(kind, start);
        Ok(())
    }

    fn lex_string(&mut self, start: Span) -> Result<(), TinysError> {
        self.advance(); // opening quote
        let mut text = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string literal")),
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    text.push('\\');
                    self.advance();
                    if let Some(escaped) = self.peek() {
                        text.push(escaped);
                        self.advance();
                    }
                }
                Some('\n') => return Err(self.err("unterminated string literal")),
                Some(c) => {
                    text.push(c);
                    self.advance();
                }
            }
        }
        self.push(Tok::Str(text), start);
        Ok(())
    }

    fn lex_raw_string(&mut self, start: Span) -> Result<(), TinysError> {
        let mut literal = String::from("r");
        self.advance(); // consume 'r'
        let mut hashes = 0;
        while self.peek() == Some('#') {
            hashes += 1;
            literal.push('#');
            self.advance();
        }
        if self.peek() != Some('"') {
            return Err(self.err("malformed raw string literal"));
        }
        literal.push('"');
        self.advance();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated raw string literal")),
                Some('"') => {
                    // Check for closing quote followed by the right number of #.
                    let mut lookahead = 1;
                    let mut matched = true;
                    for _ in 0..hashes {
                        if self.chars.get(self.pos + lookahead).copied() != Some('#') {
                            matched = false;
                            break;
                        }
                        lookahead += 1;
                    }
                    if matched {
                        literal.push('"');
                        self.advance();
                        for _ in 0..hashes {
                            literal.push('#');
                            self.advance();
                        }
                        break;
                    } else {
                        literal.push('"');
                        self.advance();
                    }
                }
                Some(c) => {
                    literal.push(c);
                    self.advance();
                }
            }
        }
        self.push(Tok::RawStr(literal), start);
        Ok(())
    }

    fn lex_attribute(&mut self, start: Span) -> Result<(), TinysError> {
        self.advance(); // consume '#'
        // Inner attribute `#!` (module docs style) — treat like an outer attr body.
        let inner_bang = self.peek() == Some('!');
        if inner_bang {
            self.advance();
        }
        if self.peek() != Some('[') {
            return Err(self.err("expected `[` after `#` in attribute"));
        }
        self.advance(); // consume '['
        let mut depth = 1;
        let mut body = String::new();
        if inner_bang {
            body.push('!');
        }
        while depth > 0 {
            match self.peek() {
                None => return Err(self.err("unterminated attribute")),
                Some('[') => {
                    depth += 1;
                    body.push('[');
                    self.advance();
                }
                Some(']') => {
                    depth -= 1;
                    if depth > 0 {
                        body.push(']');
                    }
                    self.advance();
                }
                Some(c) => {
                    body.push(c);
                    self.advance();
                }
            }
        }
        self.push(Tok::Attribute(body), start);
        Ok(())
    }

    fn lex_symbol(&mut self, c: char, start: Span) -> Result<(), TinysError> {
        macro_rules! two {
            ($second:expr, $kind:expr, $single:expr) => {{
                self.advance();
                if self.peek() == Some($second) {
                    self.advance();
                    self.push($kind, start);
                } else {
                    self.push($single, start);
                }
            }};
        }
        match c {
            '(' => {
                self.advance();
                self.paren_depth += 1;
                self.push(Tok::LParen, start);
            }
            ')' => {
                self.advance();
                self.paren_depth -= 1;
                self.push(Tok::RParen, start);
            }
            '[' => {
                self.advance();
                self.paren_depth += 1;
                self.push(Tok::LBracket, start);
            }
            ']' => {
                self.advance();
                self.paren_depth -= 1;
                self.push(Tok::RBracket, start);
            }
            '{' => {
                self.advance();
                self.paren_depth += 1;
                self.push(Tok::LBrace, start);
            }
            '}' => {
                self.advance();
                self.paren_depth -= 1;
                self.push(Tok::RBrace, start);
            }
            ':' => {
                self.advance();
                self.push(Tok::Colon, start);
            }
            ',' => {
                self.advance();
                self.push(Tok::Comma, start);
            }
            '.' => {
                // '..' / '..=' range, '.name' lifetime, or a plain dot.
                if self.peek2() == Some('.') {
                    self.advance();
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.push(Tok::DotDotEq, start);
                    } else {
                        self.push(Tok::DotDot, start);
                    }
                } else if self
                    .peek2()
                    .map(|c| c.is_alphabetic() || c == '_')
                    .unwrap_or(false)
                    && self.lifetime_position()
                {
                    self.advance(); // consume '.'
                    let mut name = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            name.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.push(Tok::Lifetime(name), start);
                } else {
                    self.advance();
                    self.push(Tok::Dot, start);
                }
            }
            '-' => {
                if self.peek2() == Some('>') {
                    self.advance();
                    self.advance();
                    self.push(Tok::Arrow, start);
                } else {
                    two!('=', Tok::MinusEq, Tok::Minus);
                }
            }
            '+' => two!('=', Tok::PlusEq, Tok::Plus),
            '*' => two!('=', Tok::StarEq, Tok::Star),
            '/' => two!('=', Tok::SlashEq, Tok::Slash),
            '%' => two!('=', Tok::PercentEq, Tok::Percent),
            '=' => two!('=', Tok::EqEq, Tok::Assign),
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.push(Tok::NotEq, start);
                } else {
                    return Err(self.err("unexpected `!` (use `not` for negation)"));
                }
            }
            '<' => two!('=', Tok::Le, Tok::Lt),
            '>' => two!('=', Tok::Ge, Tok::Gt),
            '?' => {
                self.advance();
                self.push(Tok::Question, start);
            }
            '|' => {
                self.advance();
                self.push(Tok::Pipe, start);
            }
            other => return Err(self.err(format!("unexpected character `{}`", other))),
        }
        Ok(())
    }

    /// A `.name` is a lifetime only where a type could appear: right after
    /// `[`, `,`, `(`, `:`, `->`, `<`, or `ref`. Everywhere else a dot is field
    /// access. This keeps `user.name` distinct from `ref[.a] str`.
    fn lifetime_position(&self) -> bool {
        matches!(
            self.tokens.last().map(|t| &t.kind),
            Some(Tok::LBracket)
                | Some(Tok::Comma)
                | Some(Tok::LParen)
                | Some(Tok::Colon)
                | Some(Tok::Arrow)
                | Some(Tok::Lt)
                | Some(Tok::Ref)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Tok> {
        Lexer::new(src, "test.sn")
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn emits_indent_and_dedent_for_blocks() {
        let toks = kinds("def f():\n    return 1\n");
        let indents = toks.iter().filter(|t| matches!(t, Tok::Indent)).count();
        let dedents = toks.iter().filter(|t| matches!(t, Tok::Dedent)).count();
        assert_eq!(indents, 1);
        assert_eq!(dedents, 1);
        assert!(matches!(toks.last(), Some(Tok::Eof)));
    }

    #[test]
    fn blank_and_comment_lines_do_not_affect_indentation() {
        let toks = kinds("def f():\n\n    // a comment\n    return 1\n");
        let indents = toks.iter().filter(|t| matches!(t, Tok::Indent)).count();
        assert_eq!(indents, 1, "comments/blank lines must not open a block");
    }

    #[test]
    fn newlines_are_suppressed_inside_brackets() {
        let toks = kinds("x = [\n    1,\n    2,\n]\n");
        // Only the trailing newline after `]` should survive.
        let newlines = toks.iter().filter(|t| matches!(t, Tok::Newline)).count();
        assert_eq!(newlines, 1);
    }

    #[test]
    fn distinguishes_range_from_float() {
        assert_eq!(
            kinds("x = 0..10\n"),
            vec![
                Tok::Ident("x".into()),
                Tok::Assign,
                Tok::Int(0),
                Tok::DotDot,
                Tok::Int(10),
                Tok::Newline,
                Tok::Eof,
            ]
        );
        assert!(kinds("x = 3.14\n").contains(&Tok::Float(3.14)));
    }

    #[test]
    fn lifetime_vs_field_access() {
        assert!(kinds("v: ref[.a] str\n").contains(&Tok::Lifetime("a".into())));
        // A dot after an identifier is field access, not a lifetime.
        assert!(kinds("x = user.name\n").contains(&Tok::Dot));
        assert!(!kinds("x = user.name\n")
            .iter()
            .any(|t| matches!(t, Tok::Lifetime(_))));
    }

    #[test]
    fn raw_and_normal_strings() {
        assert!(kinds("x = \"hi\"\n").contains(&Tok::Str("hi".into())));
        assert!(kinds("x = r#\"a\"b\"#\n")
            .iter()
            .any(|t| matches!(t, Tok::RawStr(_))));
    }
}
