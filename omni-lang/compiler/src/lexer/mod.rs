// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]
//! Omni Lexer - Tokenization
//!
//! Converts source text into a stream of tokens using Logos for speed.

use logos::Logos;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character at position {position}: '{char}'")]
    UnexpectedChar { position: usize, char: char },

    #[error("Unterminated string literal starting at position {position}")]
    UnterminatedString { position: usize },

    #[error("Invalid number literal: {literal}")]
    InvalidNumber { literal: String },
}

/// Token types for the Omni language
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")] // Skip whitespace including CR (but not LF - they're significant)
pub enum TokenKind {
    // Keywords
    #[token("module")]
    Module,
    #[token("struct")]
    Struct,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("match")]
    Match,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("own")]
    Own,
    #[token("shared")]
    Shared,
    #[token("unsafe")]
    Unsafe,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("implements")]
    Implements,
    #[token("const")]
    Const,
    #[token("static")]
    Static,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("extern")]
    Extern,
    #[token("comptime")]
    Comptime,
    #[token("enum")]
    Enum,
    #[token("type")]
    Type,
    #[token("where")]
    Where,
    #[token("defer")]
    Defer,
    #[token("pub")]
    Pub,
    #[token("var")]
    Var,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("loop")]
    Loop,
    #[token("yield")]
    Yield,
    #[token("Self")]
    SelfType,
    #[token("None")]
    None_,
    #[token("Some")]
    Some_,
    #[token("Ok")]
    Ok_,
    #[token("Err")]
    Err_,
    #[token("dyn")]
    Dyn,
    #[token("spawn")]
    Spawn,
    #[token("select")]
    Select,
    #[token("case")]
    Case,
    #[token("macro")]
    Macro,
    #[token("pass")]
    Pass,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,
    #[token("elif")]
    Elif,
    #[token("self")]
    SelfValue,
    #[token("super")]
    Super,

    // Attributes — #[name] or #[name(args)]
    // The lexer produces a Hash token; the parser combines Hash + [ + ... + ] into an attribute.
    #[token("#")]
    Hash,

    // Types
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("usize")]
    Usize,
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("isize")]
    Isize,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("str")]
    Str,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Literals — hex/binary/octal BEFORE generic int so logos matches them first
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*")]
    HexLiteral,
    #[regex(r"0[bB][01][01_]*")]
    BinaryLiteral,
    #[regex(r"0[oO][0-7][0-7_]*")]
    OctalLiteral,
    #[regex(r"[0-9][0-9_]*")]
    IntLiteral,
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?")]
    FloatLiteral,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r"'([^'\\]|\\.)'")]
    CharLiteral,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    #[token("and")]
    And,
    #[token("||")]
    #[token("or")]
    Or,
    #[token("!")]
    #[token("not")]
    Not,
    #[token("=")]
    Eq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("::")]
    DoubleColon,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("?")]
    Question,
    #[token("~")]
    Tilde,
    #[token("^")]
    Caret,

    // Special
    #[regex(r"\n")]
    Newline,
    // Single-line comments
    #[regex(r"//[^\n]*")]
    Comment,
    // Block comments are stripped by strip_block_comments() before tokenization
    BlockComment,

    // Indentation (handled specially)
    Indent,
    Dedent,
}

/// A token with its position in the source
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub span: std::ops::Range<usize>,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: String,
        line: usize,
        column: usize,
        span: std::ops::Range<usize>,
    ) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
            span,
        }
    }
}

/// Iterator-based Lexer API for convenience
/// Wraps the `tokenize` function to provide an iterator interface.
pub struct Lexer {
    tokens: Vec<Token>,
    index: usize,
}

impl Lexer {
    /// Create a new Lexer from source code
    pub fn new(source: &str) -> Result<Self, LexerError> {
        let tokens = tokenize(source)?;
        Ok(Lexer { tokens, index: 0 })
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}

/// Strip nested block comments (`/* ... */`) from source, preserving newlines for line counting.
fn strip_block_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut depth = 0;

    while let Some(c) = chars.next() {
        if depth > 0 {
            // Inside a block comment — skip content but keep newlines for line tracking
            if c == '/' && chars.peek() == Some(&'*') {
                chars.next();
                depth += 1;
            } else if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                depth -= 1;
            } else if c == '\n' {
                result.push('\n');
            }
        } else {
            if c == '/' && chars.peek() == Some(&'*') {
                chars.next();
                depth = 1;
            } else {
                result.push(c);
            }
        }
    }
    result
}

/// Compute the indentation of the next non-blank line in `remaining`.
/// Blank lines (lines with only spaces/tabs followed by a newline or at EOF) are skipped.
fn next_nonblank_indent(remaining: &str) -> usize {
    let mut s = remaining;
    loop {
        let spaces = s.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let after = &s[spaces..];
        if after.starts_with('\n') {
            // Blank line (spaces + newline) — skip past the newline
            s = &after[1..];
            continue;
        }
        if after.starts_with("\r\n") {
            s = &after[2..];
            continue;
        }
        // Non-blank line (or EOF) — return its indentation
        return spaces;
    }
}

/// Tokenize source code into a vector of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
    // Strip block comments before tokenization (preserves newlines for line tracking)
    let stripped = strip_block_comments(source);
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(&stripped);
    let mut line = 1;
    let mut column = 1;
    let mut indent_stack = vec![0usize];

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        let lexeme = lexer.slice().to_string();

        match result {
            Ok(kind) => {
                // Handle newlines for line tracking
                if kind == TokenKind::Newline {
                    tokens.push(Token::new(kind, lexeme, line, column, span.clone()));
                    line += 1;
                    column = 1;

                    // Check indentation of next non-blank line
                    let remaining = &stripped[lexer.span().end..];
                    let indent = next_nonblank_indent(remaining);
                    let current_indent = *indent_stack.last().unwrap();

                    if indent > current_indent {
                        indent_stack.push(indent);
                        tokens.push(Token::new(
                            TokenKind::Indent,
                            String::new(),
                            line,
                            1,
                            span.clone(),
                        ));
                    } else {
                        while indent < *indent_stack.last().unwrap() {
                            indent_stack.pop();
                            tokens.push(Token::new(
                                TokenKind::Dedent,
                                String::new(),
                                line,
                                1,
                                span.clone(),
                            ));
                        }
                    }
                } else if kind == TokenKind::Comment {
                    // Skip single-line comments — they are discarded
                } else if kind == TokenKind::IntLiteral
                    || kind == TokenKind::HexLiteral
                    || kind == TokenKind::BinaryLiteral
                    || kind == TokenKind::OctalLiteral
                    || kind == TokenKind::FloatLiteral
                {
                    // Strip underscores from number literals for the lexeme
                    let clean_lexeme: String = lexeme.chars().filter(|c| *c != '_').collect();
                    tokens.push(Token::new(kind, clean_lexeme, line, column, span.clone()));
                    column += lexeme.len();
                } else if kind == TokenKind::Hash {
                    // # can be: attribute start (#[name]) or legacy comment (#...)
                    let remaining = &stripped[span.end..];
                    let next_char = remaining.chars().next();
                    if next_char == Some('[') {
                        // Attribute syntax: #[name] — emit the Hash token
                        tokens.push(Token::new(kind, lexeme.clone(), line, column, span.clone()));
                        column += lexeme.len();
                    } else {
                        // Legacy comment: # not followed by [
                        // Skip everything until end of line by advancing the logos lexer
                        loop {
                            match lexer.next() {
                                Some(Ok(TokenKind::Newline)) => {
                                    // Emit the newline and handle indentation
                                    let nl_span = lexer.span();
                                    tokens.push(Token::new(
                                        TokenKind::Newline,
                                        "\n".to_string(),
                                        line,
                                        column,
                                        nl_span.clone(),
                                    ));
                                    line += 1;
                                    column = 1;
                                    // Handle indentation after the newline
                                    let remaining2 = &stripped[lexer.span().end..];
                                    let indent = next_nonblank_indent(remaining2);
                                    let current_indent = *indent_stack.last().unwrap();
                                    if indent > current_indent {
                                        indent_stack.push(indent);
                                        tokens.push(Token::new(
                                            TokenKind::Indent,
                                            String::new(),
                                            line,
                                            1,
                                            nl_span.clone(),
                                        ));
                                    } else {
                                        while indent < *indent_stack.last().unwrap() {
                                            indent_stack.pop();
                                            tokens.push(Token::new(
                                                TokenKind::Dedent,
                                                String::new(),
                                                line,
                                                1,
                                                nl_span.clone(),
                                            ));
                                        }
                                    }
                                    break;
                                }
                                None => break, // EOF
                                _ => continue, // Skip non-newline tokens in comment
                            }
                        }
                    }
                } else if kind == TokenKind::CharLiteral {
                    // Handle character literals with escape sequence support
                    let char_content = &lexeme[1..lexeme.len() - 1]; // Strip surrounding single quotes
                    let is_valid = if char_content.starts_with('\\') {
                        // Escape sequence: \n, \t, \r, \\, \', \0, \x??, \u{????}
                        match char_content {
                            "\\n" | "\\t" | "\\r" | "\\\\" | "\\'" | "\\0" => true,
                            s if s.starts_with("\\x") && s.len() == 4 => {
                                // \xHH - two hex digits
                                s[2..].chars().all(|c| c.is_ascii_hexdigit())
                            }
                            s if s.starts_with("\\u{") && s.ends_with('}') => {
                                // \u{HHHH} - Unicode escape
                                let hex = &s[3..s.len() - 1];
                                !hex.is_empty()
                                    && hex.len() <= 6
                                    && hex.chars().all(|c| c.is_ascii_hexdigit())
                            }
                            _ => false,
                        }
                    } else {
                        // Single character (non-escape)
                        char_content.chars().count() == 1
                    };

                    if is_valid {
                        tokens.push(Token::new(kind, lexeme.clone(), line, column, span.clone()));
                    } else {
                        return Err(LexerError::UnexpectedChar {
                            position: span.start,
                            char: char_content.chars().next().unwrap_or('?'),
                        });
                    }
                    column += lexeme.len();
                } else {
                    tokens.push(Token::new(kind, lexeme.clone(), line, column, span.clone()));
                    column += lexeme.len();
                }
            }
            Err(_) => {
                let char = stripped[span.start..].chars().next().unwrap_or('?');
                return Err(LexerError::UnexpectedChar {
                    position: span.start,
                    char,
                });
            }
        }
    }

    // Add remaining dedents at end of file
    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::new(
            TokenKind::Dedent,
            String::new(),
            line,
            column,
            stripped.len()..stripped.len(),
        ));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "fn main() -> i32:\n    return 42";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].lexeme, "main");
    }

    #[test]
    fn test_struct_definition() {
        let source = "struct Neuron:\n    weights: &mut [f32]\n    bias: f32";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Struct);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_pass_keyword() {
        let source = "fn empty():\n    pass";
        let tokens = tokenize(source).unwrap();
        // fn, empty, (, ), :, newline, indent, pass
        let pass_token = tokens.iter().find(|t| t.kind == TokenKind::Pass);
        assert!(pass_token.is_some(), "pass keyword should be recognized");
    }

    #[test]
    fn test_line_comments() {
        let source = "let x = 42 // this is a comment\nlet y = 10";
        let tokens = tokenize(source).unwrap();
        // Comments should be stripped out
        assert!(!tokens.iter().any(|t| t.lexeme.contains("comment")));
        // But the code tokens should exist
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Let));
    }

    #[test]
    fn test_block_comments() {
        let source = "let x = /* this is a\nblock comment */ 42";
        let tokens = tokenize(source).unwrap();
        assert!(!tokens.iter().any(|t| t.lexeme.contains("comment")));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Let));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::IntLiteral));
    }

    #[test]
    fn test_nested_block_comments() {
        let source = "let x = /* outer /* inner */ outer */ 42";
        let tokens = tokenize(source).unwrap();
        assert!(!tokens.iter().any(|t| t.lexeme.contains("outer")));
        assert!(!tokens.iter().any(|t| t.lexeme.contains("inner")));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::IntLiteral));
    }

    #[test]
    fn test_hash_is_not_comment() {
        let source = "#[test]\nfn my_test():\n    pass";
        let tokens = tokenize(source).unwrap();
        // # should produce a Hash token, not be treated as a comment
        assert_eq!(tokens[0].kind, TokenKind::Hash);
        assert_eq!(tokens[1].kind, TokenKind::LBracket);
    }

    #[test]
    fn test_attribute_tokens() {
        let source = "#[inline]\nfn fast():\n    pass";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Hash);
        assert_eq!(tokens[1].kind, TokenKind::LBracket);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].lexeme, "inline");
        assert_eq!(tokens[3].kind, TokenKind::RBracket);
    }

    #[test]
    fn test_all_keywords() {
        let keywords = vec![
            ("module", TokenKind::Module),
            ("struct", TokenKind::Struct),
            ("fn", TokenKind::Fn),
            ("let", TokenKind::Let),
            ("mut", TokenKind::Mut),
            ("if", TokenKind::If),
            ("else", TokenKind::Else),
            ("for", TokenKind::For),
            ("in", TokenKind::In),
            ("while", TokenKind::While),
            ("return", TokenKind::Return),
            ("match", TokenKind::Match),
            ("async", TokenKind::Async),
            ("await", TokenKind::Await),
            ("import", TokenKind::Import),
            ("pass", TokenKind::Pass),
            ("try", TokenKind::Try),
            ("catch", TokenKind::Catch),
            ("finally", TokenKind::Finally),
            ("break", TokenKind::Break),
            ("continue", TokenKind::Continue),
            ("loop", TokenKind::Loop),
            ("yield", TokenKind::Yield),
            ("const", TokenKind::Const),
            ("enum", TokenKind::Enum),
            ("trait", TokenKind::Trait),
            ("impl", TokenKind::Impl),
            ("pub", TokenKind::Pub),
            ("var", TokenKind::Var),
            ("defer", TokenKind::Defer),
            ("spawn", TokenKind::Spawn),
            ("select", TokenKind::Select),
            ("macro", TokenKind::Macro),
            ("elif", TokenKind::Elif),
        ];
        for (text, expected_kind) in keywords {
            let tokens = tokenize(text).unwrap();
            assert_eq!(
                tokens[0].kind, expected_kind,
                "Keyword '{}' should tokenize correctly",
                text
            );
        }
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / % == != < > <= >= && || ! = -> => :: .. ..= ? ~ ^";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Plus));
        assert!(kinds.contains(&&TokenKind::Minus));
        assert!(kinds.contains(&&TokenKind::EqEq));
        assert!(kinds.contains(&&TokenKind::Arrow));
        assert!(kinds.contains(&&TokenKind::FatArrow));
        assert!(kinds.contains(&&TokenKind::Question));
    }

    #[test]
    fn test_shift_operators() {
        let source = "x << 3 >> 1";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Shl);
        assert_eq!(tokens[3].kind, TokenKind::Shr);
    }

    #[test]
    fn test_string_literal() {
        let source = r#""hello world""#;
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn test_numeric_literals() {
        let source = "42 3.14";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
    }

    #[test]
    fn test_hex_literal() {
        let source = "0xFF 0x1A2B";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::HexLiteral);
        assert_eq!(tokens[0].lexeme, "0xFF");
        assert_eq!(tokens[1].kind, TokenKind::HexLiteral);
    }

    #[test]
    fn test_binary_literal() {
        let source = "0b1010 0B11110000";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::BinaryLiteral);
        assert_eq!(tokens[0].lexeme, "0b1010");
        assert_eq!(tokens[1].kind, TokenKind::BinaryLiteral);
    }

    #[test]
    fn test_octal_literal() {
        let source = "0o777 0O123";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::OctalLiteral);
        assert_eq!(tokens[0].lexeme, "0o777");
        assert_eq!(tokens[1].kind, TokenKind::OctalLiteral);
    }

    #[test]
    fn test_float_exponent() {
        let source = "1.5e10 2.0E-3 3.14e+2";
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[0].lexeme, "1.5e10");
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "2.0E-3");
        assert_eq!(tokens[2].kind, TokenKind::FloatLiteral);
    }

    #[test]
    fn test_underscores_in_numbers() {
        let source = "1_000_000 3.141_592 0xFF_FF 0b1010_0101 0o7_7_7";
        let tokens = tokenize(source).unwrap();
        // IntLiteral: underscores stripped
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].lexeme, "1000000");
        // FloatLiteral: underscores stripped
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "3.141592");
        // HexLiteral: underscores stripped
        assert_eq!(tokens[2].kind, TokenKind::HexLiteral);
        assert_eq!(tokens[2].lexeme, "0xFFFF");
        // BinaryLiteral: underscores stripped
        assert_eq!(tokens[3].kind, TokenKind::BinaryLiteral);
        assert_eq!(tokens[3].lexeme, "0b10100101");
        // OctalLiteral: underscores stripped
        assert_eq!(tokens[4].kind, TokenKind::OctalLiteral);
        assert_eq!(tokens[4].lexeme, "0o777");
    }

    #[test]
    fn test_tabs_counted_in_indentation() {
        let source = "fn main():\n\treturn 42";
        let tokens = tokenize(source).unwrap();
        // Should produce Indent after newline
        let indent_token = tokens.iter().find(|t| t.kind == TokenKind::Indent);
        assert!(
            indent_token.is_some(),
            "tab indentation should be recognized"
        );
    }

    #[test]
    fn test_lexer_new_returns_result() {
        let result = Lexer::new("fn main():\n    pass");
        assert!(result.is_ok());
    }
}
