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
#[logos(skip r"[ \t\r]+")]  // Skip whitespace including CR (but not LF - they're significant)
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

    // Attributes
    #[regex(r"@[a-zA-Z_][a-zA-Z0-9_]*")]
    Attribute,

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

    // Literals
    #[regex(r"[0-9]+")]
    IntLiteral,
    #[regex(r"[0-9]+\.[0-9]+")]
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
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
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

    // Special
    #[regex(r"\n")]
    Newline,
    #[regex(r"#[^\n]*")]
    Comment,

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
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize, span: std::ops::Range<usize>) -> Self {
        Self { kind, lexeme, line, column, span }
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
    pub fn new(source: &str) -> Self {
        let tokens = tokenize(source).unwrap_or_default();
        Lexer { tokens, index: 0 }
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

/// Tokenize source code into a vector of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(source);
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

                    // Check indentation of next line
                    let remaining = &source[lexer.span().end..];
                    let indent = remaining.chars().take_while(|c| *c == ' ').count();
                    let current_indent = *indent_stack.last().unwrap();

                    if indent > current_indent {
                        indent_stack.push(indent);
                        tokens.push(Token::new(TokenKind::Indent, String::new(), line, 1, span.clone()));
                    } else {
                        while indent < *indent_stack.last().unwrap() {
                            indent_stack.pop();
                            tokens.push(Token::new(TokenKind::Dedent, String::new(), line, 1, span.clone()));
                        }
                    }
                } else if kind != TokenKind::Comment {
                    if kind == TokenKind::CharLiteral {
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
                                    let hex = &s[3..s.len()-1];
                                    !hex.is_empty() && hex.len() <= 6 && hex.chars().all(|c| c.is_ascii_hexdigit())
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
            }
            Err(_) => {
                let char = source[span.start..].chars().next().unwrap_or('?');
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
        tokens.push(Token::new(TokenKind::Dedent, String::new(), line, column, source.len()..source.len()));
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
}
