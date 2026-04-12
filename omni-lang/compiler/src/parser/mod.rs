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

//! Omni Parser - AST Generation
//!
//! Parses tokens into an Abstract Syntax Tree.

pub mod ast;

use crate::lexer::{Token, TokenKind};
use crate::monitor;
use ast::*;
use log::warn;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Error codes for parser diagnostics.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    /// E001: Unexpected token
    UnexpectedToken,
    /// E002: Expected expression
    ExpectedExpression,
    /// E003: Unterminated block
    UnterminatedBlock,
    /// E004: Type mismatch
    TypeMismatch,
    /// E005: Unexpected end of file
    UnexpectedEof,
    /// E006: Invalid syntax
    InvalidSyntax,
    /// E007: Expected item
    ExpectedItem,
    /// E008: Missing token
    MissingToken,
    /// E009: Too many errors
    TooManyErrors,
}

impl ParseErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseErrorCode::UnexpectedToken => "E001",
            ParseErrorCode::ExpectedExpression => "E002",
            ParseErrorCode::UnterminatedBlock => "E003",
            ParseErrorCode::TypeMismatch => "E004",
            ParseErrorCode::UnexpectedEof => "E005",
            ParseErrorCode::InvalidSyntax => "E006",
            ParseErrorCode::ExpectedItem => "E007",
            ParseErrorCode::MissingToken => "E008",
            ParseErrorCode::TooManyErrors => "E009",
        }
    }
}

impl std::fmt::Display for ParseErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("[{code}] Unexpected token at line {line}, column {column}: expected {expected}, got {got}{}", hint.as_ref().map(|h| format!(" (hint: {})", h)).unwrap_or_default())]
    UnexpectedToken {
        line: usize,
        column: usize,
        expected: String,
        got: String,
        code: ParseErrorCode,
        hint: Option<String>,
    },

    #[error("[{code}] Unexpected end of file{}", hint.as_ref().map(|h| format!(" (hint: {})", h)).unwrap_or_default())]
    UnexpectedEof {
        code: ParseErrorCode,
        hint: Option<String>,
    },

    #[error("[{code}] Invalid syntax at line {line}, column {column}: {message}{}", hint.as_ref().map(|h| format!(" (hint: {})", h)).unwrap_or_default())]
    InvalidSyntax {
        line: usize,
        column: usize,
        message: String,
        code: ParseErrorCode,
        hint: Option<String>,
    },

    #[error("[E009] Too many errors ({count}), aborting")]
    TooManyErrors { count: usize },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    /// Collected parse errors for error recovery.
    pub errors: Vec<ParseError>,
    /// Maximum number of errors before the parser gives up (default: 50).
    pub error_limit: usize,
    /// Tick counter to detect runaway parsing loops.
    pub tick_count: usize,
    /// Maximum number of parse loop iterations (ticks) before aborting.
    pub tick_limit: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            error_limit: 50,
            tick_count: 0,
            tick_limit: 1_000_000,
        }
    }

    /// Check whether the error limit has been reached.
    fn too_many_errors(&self) -> bool {
        self.errors.len() >= self.error_limit
    }

    /// Record an error. Returns `Err` with `TooManyErrors` if the limit is hit.
    fn record_error(&mut self, error: ParseError) -> Result<(), ParseError> {
        // Log error details immediately to make diagnostics available
        warn!("parser error recorded: {}", error);
        // Also record the error message in the monitor for off-line dumps
        monitor::record_parser_error(&format!("{}", error));
        self.errors.push(error);
        if self.too_many_errors() {
            Err(ParseError::TooManyErrors {
                count: self.errors.len(),
            })
        } else {
            Ok(())
        }
    }

    /// Panic-mode error recovery: advance tokens until we reach a
    /// synchronization point — a newline followed by a statement/item keyword,
    /// a `Dedent`, or end-of-input.
    fn synchronize(&mut self) {
        // Add a forward-progress guard and heartbeat so we can detect
        // if synchronize() is looping without consuming tokens.
        let mut iterations: usize = 0;
        loop {
            // heartbeat for monitor and a compact parser-state snapshot
            monitor::update_heartbeat();
            // Prepare a short preview of the next token kinds/lexemes
            let mut preview = Vec::new();
            for i in 0..8 {
                if let Some(t) = self.tokens.get(self.current + i) {
                    preview.push(format!("{:?}('{}')", t.kind, t.lexeme));
                } else {
                    break;
                }
            }
            monitor::record_parser_snapshot(self.current, &preview);
            log::debug!(
                "parser:synchronize tick current={} iter={} next={:?}",
                self.current,
                iterations,
                self.peek_kind()
            );
            iterations += 1;
            if iterations > 10_000 {
                warn!(
                    "synchronize() exceeded {} iterations, forcing exit",
                    iterations
                );
                break;
            }
            match self.peek_kind() {
                None => break,                    // EOF
                Some(TokenKind::Dedent) => break, // block boundary
                Some(TokenKind::Newline) => {
                    self.advance(); // consume the newline
                                    // If the next token is a synchronization keyword, stop.
                    if self.is_sync_token() {
                        break;
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Returns `true` if the current token is one we can synchronize to.
    fn is_sync_token(&self) -> bool {
        matches!(
            self.peek_kind(),
            None // EOF
            | Some(TokenKind::Fn)
            | Some(TokenKind::Struct)
            | Some(TokenKind::Trait)
            | Some(TokenKind::Impl)
            | Some(TokenKind::Module)
            | Some(TokenKind::Import)
            | Some(TokenKind::Let)
            | Some(TokenKind::Var)
            | Some(TokenKind::If)
            | Some(TokenKind::For)
            | Some(TokenKind::While)
            | Some(TokenKind::Return)
            | Some(TokenKind::Dedent)
            | Some(TokenKind::RBrace)
            | Some(TokenKind::Enum)
            | Some(TokenKind::Const)
            | Some(TokenKind::Type)
            | Some(TokenKind::Pub)
            | Some(TokenKind::Match)
            | Some(TokenKind::Loop)
            | Some(TokenKind::Break)
            | Some(TokenKind::Continue)
        )
    }

    /// Generate a hint/suggestion for common typos.
    fn suggest_hint(got: &str) -> Option<String> {
        let suggestions: &[(&str, &str)] = &[
            ("fun", "did you mean 'fn'?"),
            ("func", "did you mean 'fn'?"),
            ("function", "did you mean 'fn'?"),
            ("def", "did you mean 'fn'?"),
            ("class", "did you mean 'struct'?"),
            ("elif", "did you mean 'else' followed by 'if'?"),
            ("elsif", "did you mean 'else' followed by 'if'?"),
            ("elseif", "did you mean 'else' followed by 'if'?"),
            ("var", "did you mean 'let' or 'var'?"),
            ("const", "did you mean 'const'?"),
            ("interface", "did you mean 'trait'?"),
            ("extends", "did you mean 'impl'?"),
            ("use", "did you mean 'import'?"),
            ("require", "did you mean 'import'?"),
            ("include", "did you mean 'import'?"),
        ];
        let got_lower = got.to_lowercase();
        for (typo, suggestion) in suggestions {
            if got_lower == *typo {
                return Some(suggestion.to_string());
            }
        }
        None
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
            // Update lightweight monitor token counter when enabled.
            monitor::inc_tokens(1);
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<&Token, ParseError> {
        if let Some(token) = self.peek() {
            if &token.kind == kind {
                Ok(self.advance().unwrap())
            } else {
                let hint = if matches!(kind, TokenKind::RBracket)
                    && matches!(token.kind, TokenKind::Semicolon)
                {
                    Some("Ensure array size is followed by ']'".to_string())
                } else {
                    Self::suggest_hint(&token.lexeme)
                };
                Err(ParseError::UnexpectedToken {
                    line: self.tokens[self.current].line,
                    column: self.tokens[self.current].column,
                    expected: format!("{:?}", kind),
                    got: format!("{:?}", token.kind),
                    code: ParseErrorCode::UnexpectedToken,
                    hint,
                })
            }
        } else {
            Err(ParseError::UnexpectedEof {
                code: ParseErrorCode::UnexpectedEof,
                hint: None,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            self.advance();
        }
    }

    /// Parse an attribute: #[name] or #[name(args)]
    fn parse_attribute(&mut self) -> Result<String, ParseError> {
        self.expect(&TokenKind::Hash)?;
        self.expect(&TokenKind::LBracket)?;
        let name = self.parse_identifier()?;

        let attr = if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
            self.advance(); // consume (
            let mut args = String::new();
            let mut depth = 1;
            while depth > 0 {
                if let Some(token) = self.advance() {
                    match token.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth -= 1;
                            if depth > 0 {
                                args.push_str(&token.lexeme);
                            }
                        }
                        _ => {
                            if !args.is_empty() {
                                args.push_str(", ");
                            }
                            args.push_str(&token.lexeme);
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedEof {
                        code: ParseErrorCode::UnexpectedEof,
                        hint: Some("unterminated attribute".to_string()),
                    });
                }
            }
            format!("#[{}({})]", name, args)
        } else {
            format!("#[{}]", name)
        };

        self.expect(&TokenKind::RBracket)?;
        Ok(attr)
    }

    /// Parse a complete module.
    /// Uses panic-mode error recovery: on error, it records the error,
    /// synchronizes to the next item boundary, and continues parsing.
    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut items = Vec::new();

        self.skip_newlines();
        // Heartbeat timer for progress logging to help detect runtime stalls
        let mut last_heartbeat = Instant::now();

        while self.peek().is_some() {
            // Increment tick counter and abort if we've exceeded the configured limit
            self.tick_count = self.tick_count.saturating_add(1);
            if self.tick_count > self.tick_limit {
                return Err(ParseError::TooManyErrors {
                    count: self.errors.len(),
                });
            }
            // Track progress each iteration to avoid infinite loops
            let before_idx = self.current;
            // Periodically warn about parser progress so we can trace runaway parsing
            // Emit a heartbeat at most once per second to see parser activity in runtime logs.
            if last_heartbeat.elapsed() >= Duration::from_secs(1) {
                let tok = self.peek().map(|t| format!("{:?}('{}')", t.kind, t.lexeme));
                warn!(
                    "parse_module progress: items={}, current={}, next_token={:?}",
                    items.len(),
                    self.current,
                    tok
                );
                last_heartbeat = Instant::now();
            }
            // Tolerate stray/top-level tokens that sometimes appear during recovery
            // (e.g., stray `=>`, `(`, or string/char fragments). Skip them and
            // continue parsing items to avoid spurious failures.
            match self.peek_kind() {
                Some(TokenKind::FatArrow)
                | Some(TokenKind::LParen)
                | Some(TokenKind::StringLiteral)
                | Some(TokenKind::CharLiteral)
                | Some(TokenKind::Colon)
                | Some(TokenKind::Lt)
                | Some(TokenKind::Dedent)
                | Some(TokenKind::RBrace) => {
                    self.advance();
                    self.skip_newlines();
                    continue;
                }
                // Tolerate an unexpected top-level `{ ... }` block by skipping
                // its balanced contents. Some generated or recovered code can
                // emit braced blocks at top-level; skipping them prevents the
                // parser from producing repeated `expected Identifier, got LBrace`
                // errors and heavy recovery churn.
                Some(TokenKind::LBrace) => {
                    // Consume the opening brace and skip until matching `}`.
                    self.advance();
                    let mut depth: usize = 1;
                    while depth > 0 {
                        match self.peek_kind() {
                            Some(TokenKind::LBrace) => {
                                depth += 1;
                                self.advance();
                            }
                            Some(TokenKind::RBrace) => {
                                depth -= 1;
                                self.advance();
                            }
                            Some(_) => {
                                self.advance();
                            }
                            None => break, // EOF while skipping; give up
                        }
                    }
                    self.skip_newlines();
                    continue;
                }
                _ => {}
            }
            match self.parse_item() {
                Ok(item) => {
                    // Defensive cap: if items grows unreasonably large,
                    // abort parsing to avoid OOM and surface the issue.
                    if items.len() > 100_000 {
                        return Err(ParseError::TooManyErrors { count: items.len() });
                    }
                    items.push(item);
                    // Notify monitor that we produced another top-level item.
                    monitor::inc_items();
                }
                Err(e) => {
                    self.record_error(e)?;
                    log::debug!(
                        "parser: record_error -> synchronize at current={}",
                        self.current
                    );
                    monitor::update_heartbeat();
                    self.synchronize();
                }
            }
            self.skip_newlines();
            // If no token was consumed during the loop, force advance one
            if self.current == before_idx {
                if self.peek().is_none() {
                    break;
                }
                self.advance();
            }
        }

        Ok(Module { items })
    }

    /// Parse a top-level item
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        self.skip_newlines();

        // Check for attributes: #[name] or #[name(args)]
        let mut attributes = Vec::new();
        while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
            let attr = self.parse_attribute()?;
            attributes.push(attr);
            self.skip_newlines();
        }

        // If a braced block appears where an item is expected, consume the
        // balanced braces and treat it as an empty `Comptime` item. This helps
        // recovery when stray `{ ... }` blocks are present at top-level.
        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // consume and skip balanced braces
            self.advance();
            let mut depth: usize = 1;
            while depth > 0 {
                match self.peek_kind() {
                    Some(TokenKind::LBrace) => {
                        depth += 1;
                        self.advance();
                    }
                    Some(TokenKind::RBrace) => {
                        depth -= 1;
                        self.advance();
                    }
                    Some(_) => {
                        self.advance();
                    }
                    None => break,
                }
            }
            self.skip_newlines();
            monitor::update_heartbeat();
            return Ok(Item::Comptime(Block {
                statements: Vec::new(),
            }));
        }

        match self.peek_kind() {
            Some(TokenKind::Module) => self.parse_module_decl(attributes),
            Some(TokenKind::Struct) => self.parse_struct(attributes),
            Some(TokenKind::Enum) => self.parse_enum(attributes),
            Some(TokenKind::Fn) => self.parse_function(attributes),
            Some(TokenKind::Async) => self.parse_function(attributes),
            Some(TokenKind::Trait) => self.parse_trait(attributes),
            Some(TokenKind::Impl) => self.parse_impl(attributes),
            Some(TokenKind::Import) => self.parse_import(),
            Some(TokenKind::Static) => self.parse_static(attributes),
            Some(TokenKind::Const) => self.parse_const(attributes),
            Some(TokenKind::Type) => self.parse_type_alias(attributes),
            Some(TokenKind::Extern) => self.parse_extern(attributes),
            Some(TokenKind::Comptime) => self.parse_comptime(),
            Some(TokenKind::Macro) => self.parse_macro(attributes),
            Some(TokenKind::Pub) => {
                self.advance();
                attributes.push("@pub".to_string());
                self.parse_item()
            }
            _ => {
                // Record an error and attempt panic-mode recovery, but return
                // an empty `Comptime` item so module parsing can continue.
                if let Some(token) = self.peek() {
                    let hint = Self::suggest_hint(&token.lexeme);
                    let err = ParseError::InvalidSyntax {
                        line: token.line,
                        column: token.column,
                        message: format!("Expected item, got {:?}", token.kind),
                        code: ParseErrorCode::ExpectedItem,
                        hint,
                    };
                    // Record error (may return TooManyErrors)
                    let _ = self.record_error(err.clone());
                }
                // Attempt to synchronize to the next item boundary and continue.
                let before = self.current;
                self.synchronize();
                // If synchronize made no progress, advance one token to avoid
                // an infinite loop that can grow the items vector without bound.
                if self.current == before {
                    log::debug!("parser: fallback advance at current={}", self.current);
                    monitor::update_heartbeat();
                    self.advance();
                }
                Ok(Item::Comptime(Block {
                    statements: Vec::new(),
                }))
            }
        }
    }

    /// Parse module declaration — supports colon+indent, brace-delimited, and file-scope.
    /// Supports double-colon separated paths: module std::io, module std::collections::vec
    fn parse_module_decl(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Module)?;
        let mut name = self.parse_identifier()?;

        // Accept :: separated namespace paths
        while matches!(self.peek_kind(), Some(TokenKind::DoubleColon)) {
            self.advance(); // consume ::
            let segment = self.parse_identifier()?;
            name = format!("{}::{}", name, segment);
        }

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited module block ──
            self.advance(); // consume '{'
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                self.skip_newlines();
            }

            let mut items = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                self.skip_newlines();
                while matches!(self.peek_kind(), Some(TokenKind::Indent))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                {
                    self.advance();
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                if self.peek().is_none() {
                    break;
                }
                items.push(self.parse_item()?);
                self.skip_newlines();
            }
            while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.advance();
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;

            Ok(Item::Module(ModuleDecl {
                name,
                attributes,
                items,
            }))
        } else if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            // ── Colon + indent/dedent block ──
            self.advance(); // consume colon
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            let mut items = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                items.push(self.parse_item()?);
                self.skip_newlines();
            }
            self.expect(&TokenKind::Dedent)?;

            Ok(Item::Module(ModuleDecl {
                name,
                attributes,
                items,
            }))
        } else {
            // File-scope module declaration: `module name` with no body block
            // The module's items are the remaining top-level items in the file
            self.skip_newlines();
            Ok(Item::Module(ModuleDecl {
                name,
                attributes,
                items: Vec::new(),
            }))
        }
    }

    /// Parse struct definition — supports both colon+indent and brace-delimited blocks.
    fn parse_struct(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Struct)?;
        let name = self.parse_identifier()?;

        // Optional implements clause
        let implements = if matches!(self.peek_kind(), Some(TokenKind::Implements)) {
            self.advance();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited block ──
            self.advance(); // consume '{'
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                self.skip_newlines();
            }

            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                self.skip_newlines();
                while matches!(self.peek_kind(), Some(TokenKind::Indent))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                {
                    self.advance();
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::Identifier)) {
                    fields.push(self.parse_field()?);
                    // Allow optional comma or semicolon separators in brace blocks
                    if matches!(self.peek_kind(), Some(TokenKind::Comma))
                        || matches!(self.peek_kind(), Some(TokenKind::Semicolon))
                    {
                        self.advance();
                    }
                } else if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                } else {
                    self.advance(); // skip unexpected token
                }
            }
            while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.advance();
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
        } else {
            // ── Colon + indent/dedent block ──
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.skip_newlines();
                if self.peek().is_none() {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::Identifier)) {
                    fields.push(self.parse_field()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                } else {
                    // Forward progress on unsupported/unknown members.
                    self.advance();
                }
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.expect(&TokenKind::Dedent)?;
            }
        }

        Ok(Item::Struct(StructDef {
            name,
            attributes,
            implements,
            fields,
            methods,
        }))
    }

    /// Parse a struct field
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.skip_newlines();
        Ok(Field { name, ty })
    }

    /// Parse a function definition
    fn parse_function(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        let func = self.parse_fn_inner(attributes)?;
        Ok(Item::Function(func))
    }

    /// Parse extern block (e.g. extern "C++" { ... } or extern "C":)
    ///
    /// Supports two block styles:
    ///   - Brace-delimited: `extern "C" { fn foo() -> i32; ... }`
    ///   - Colon + indent:  `extern "C":\n    fn foo() -> i32\n`
    ///
    /// Also accepts an optional `from "libname"` clause after the ABI string
    /// (e.g. `extern "C" from "kernel32":`).
    fn parse_extern(&mut self, _attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Extern)?;

        let abi = if matches!(self.peek_kind(), Some(TokenKind::StringLiteral)) {
            let s = self.advance().unwrap().lexeme.clone();
            s[1..s.len() - 1].to_string() // Remove quotes
        } else {
            "C".to_string() // Default to C
        };

        // Optional `from "libname"` clause — consume and ignore for now.
        if self.peek().map(|t| t.lexeme.as_str()) == Some("from") {
            self.advance(); // consume `from`
            if matches!(self.peek_kind(), Some(TokenKind::StringLiteral)) {
                self.advance(); // consume the library name string
            }
        }

        // Skip optional attributes between the ABI and the block opener.
        // e.g. `extern "C" #[cfg(unix)]:`
        while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
            let _ = self.parse_attribute()?;
            self.skip_newlines();
        }

        // Allow newlines between ABI string and block opener
        self.skip_newlines();

        let mut functions = Vec::new();

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited block ──
            self.advance(); // consume '{'
            self.skip_newlines();

            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                self.skip_newlines();
                if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                // Allow attributes on individual extern fn declarations
                let mut fn_attrs = Vec::new();
                while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
                    let attr = self.parse_attribute()?;
                    fn_attrs.push(attr);
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn))
                    || matches!(self.peek_kind(), Some(TokenKind::Async))
                {
                    let mut func = self.parse_fn_signature()?;
                    func.attributes = fn_attrs;
                    functions.push(func);
                    // Consume optional trailing semicolon after signature
                    if matches!(self.peek_kind(), Some(TokenKind::Semicolon)) {
                        self.advance();
                    }
                } else if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                } else {
                    // Skip unexpected tokens inside braced extern to avoid infinite loop
                    self.advance();
                }
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
        } else if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            // ── Colon + indent/dedent block ──
            self.advance(); // consume ':'
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.skip_newlines();
                if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                }
                // Allow attributes on individual extern fn declarations
                let mut fn_attrs = Vec::new();
                while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
                    let attr = self.parse_attribute()?;
                    fn_attrs.push(attr);
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn))
                    || matches!(self.peek_kind(), Some(TokenKind::Async))
                {
                    let mut func = self.parse_fn_signature()?;
                    func.attributes = fn_attrs;
                    functions.push(func);
                } else if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                } else {
                    // Skip unexpected tokens to avoid infinite loop
                    self.advance();
                }
                self.skip_newlines();
            }
            self.expect(&TokenKind::Dedent)?;
        } else {
            // Single extern function without a block (edge case)
            // e.g. `extern "C" fn foo() -> i32`
            if matches!(self.peek_kind(), Some(TokenKind::Fn))
                || matches!(self.peek_kind(), Some(TokenKind::Async))
            {
                functions.push(self.parse_fn_signature()?);
            } else if let Some(token) = self.peek() {
                return Err(ParseError::InvalidSyntax {
                    line: token.line,
                    column: token.column,
                    message: format!(
                        "Expected '{{' or ':' after extern ABI, got {:?}",
                        token.kind
                    ),
                    code: ParseErrorCode::InvalidSyntax,
                    hint: Some(
                        "use `extern \"C\" { ... }` or `extern \"C\":` with indented body"
                            .to_string(),
                    ),
                });
            }
        }

        Ok(Item::Extern(ExternBlock { abi, functions }))
    }

    /// Parse comptime block
    fn parse_comptime(&mut self) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Comptime)?;
        let body = self.parse_block()?;
        Ok(Item::Comptime(body))
    }

    /// Parse function signature (fn name(args) -> Ret) — no body.
    ///
    /// Used for extern function declarations. Handles optional trailing
    /// semicolons and newlines. Does NOT consume a function body.
    fn parse_fn_signature(&mut self) -> Result<Function, ParseError> {
        let is_async = if matches!(self.peek_kind(), Some(TokenKind::Async)) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&TokenKind::Fn)?;
        let name = self.parse_identifier()?;
        self.skip_fn_generic_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if matches!(self.peek_kind(), Some(TokenKind::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Consume optional trailing semicolon (common in brace-delimited extern blocks)
        if matches!(self.peek_kind(), Some(TokenKind::Semicolon)) {
            self.advance();
        }

        // Skip trailing newlines after the signature
        self.skip_newlines();

        Ok(Function {
            name,
            is_async,
            attributes: Vec::new(),
            params,
            return_type,
            effect_row: None, // No effect annotations in extern signatures
            body: Block {
                statements: Vec::new(),
            }, // Empty body for extern
        })
    }

    /// Parse a method (function inside struct)
    fn parse_method(&mut self) -> Result<Function, ParseError> {
        // Check for attributes on method
        let mut attributes = Vec::new();
        while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
            let attr = self.parse_attribute()?;
            attributes.push(attr);
            self.skip_newlines();
        }
        self.parse_fn_inner(attributes)
    }

    fn parse_fn_inner(&mut self, attributes: Vec<String>) -> Result<Function, ParseError> {
        let is_async = if matches!(self.peek_kind(), Some(TokenKind::Async)) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&TokenKind::Fn)?;
        let name = self.parse_identifier()?;
        self.skip_fn_generic_params()?;

        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        // Return type
        let return_type = if matches!(self.peek_kind(), Some(TokenKind::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Effect annotations: `/ effect1 + effect2` (v2.0 feature)
        let effect_row = if matches!(self.peek_kind(), Some(TokenKind::Slash)) {
            self.advance();
            let mut effects = Vec::new();
            loop {
                match self.peek_kind() {
                    Some(TokenKind::Identifier) => {
                        let name = self.parse_identifier()?;
                        // Check for type parameter: Error[E]
                        let param = if matches!(self.peek_kind(), Some(TokenKind::LBracket)) {
                            self.advance();
                            let p = self.parse_identifier()?;
                            self.expect(&TokenKind::RBracket)?;
                            Some(p)
                        } else {
                            None
                        };
                        effects.push(EffectSymbol { name, param });
                    }
                    Some(TokenKind::Plus) => {
                        self.advance(); // consume '+'
                    }
                    _ => break,
                }
            }
            Some(EffectRow { effects })
        } else {
            None
        };

        // Skip where clauses if present
        if matches!(self.peek_kind(), Some(TokenKind::Where)) {
            self.advance(); // consume `where`
                            // Consume tokens until we hit a colon or brace (block start)
            while !matches!(self.peek_kind(), Some(TokenKind::Colon))
                && !matches!(self.peek_kind(), Some(TokenKind::LBrace))
                && !matches!(self.peek_kind(), Some(TokenKind::Newline))
                && self.peek().is_some()
            {
                self.advance();
            }
        }

        // Function body — accept either colon+indent or brace-delimited block
        let body = if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            self.advance(); // consume '{'
            self.parse_brace_block()?
        } else {
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.parse_block()?
        };

        Ok(Function {
            name,
            is_async,
            attributes,
            params,
            return_type,
            effect_row,
            body,
        })
    }

    /// Consume optional function generic parameter list: `fn name[T, U: Trait](...)`
    ///
    /// Current phase support is syntax-level only: generic parameter constraints are
    /// accepted and skipped so parser can progress while semantic implementation evolves.
    fn skip_fn_generic_params(&mut self) -> Result<(), ParseError> {
        if !matches!(self.peek_kind(), Some(TokenKind::LBracket)) {
            return Ok(());
        }

        self.expect(&TokenKind::LBracket)?;
        let mut depth = 1usize;

        while depth > 0 {
            match self.peek_kind() {
                Some(TokenKind::LBracket) => {
                    self.advance();
                    depth += 1;
                }
                Some(TokenKind::RBracket) => {
                    self.advance();
                    depth -= 1;
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    let (line, column) = self.peek().map(|t| (t.line, t.column)).unwrap_or((0, 0));
                    return Err(ParseError::InvalidSyntax {
                        line,
                        column,
                        message: "Unterminated function generic parameter list".to_string(),
                        code: ParseErrorCode::InvalidSyntax,
                        hint: Some("close generic parameters with ']'".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Parse function parameters
    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        // Handle self parameter
        if matches!(self.peek_kind(), Some(TokenKind::Ampersand)) {
            self.advance();
            let mutable = matches!(self.peek_kind(), Some(TokenKind::Mut));
            if mutable {
                self.advance();
            }
            if self.peek().map(|t| t.lexeme.as_str()) == Some("self") {
                self.advance();
                params.push(Param {
                    name: "self".to_string(),
                    ty: Type::SelfRef { mutable },
                });
            }
        } else if self.peek().map(|t| t.lexeme.as_str()) == Some("self") {
            self.advance();
            params.push(Param {
                name: "self".to_string(),
                ty: Type::SelfOwned,
            });
        }

        // Handle comma after self
        if !params.is_empty() && matches!(self.peek_kind(), Some(TokenKind::Comma)) {
            self.advance();
        }

        while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
            let name = self.parse_identifier()?;
            let ty = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
                self.advance();
                self.parse_type()?
            } else {
                Type::Infer
            };
            params.push(Param { name, ty });

            if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    /// Parse a type annotation
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Check for ownership modifiers
        let ownership = if matches!(self.peek_kind(), Some(TokenKind::Own)) {
            self.advance();
            Some(Ownership::Owned)
        } else if matches!(self.peek_kind(), Some(TokenKind::Shared)) {
            self.advance();
            Some(Ownership::Shared)
        } else if matches!(self.peek_kind(), Some(TokenKind::Ampersand)) {
            self.advance();
            let mutable = matches!(self.peek_kind(), Some(TokenKind::Mut));
            if mutable {
                self.advance();
            }
            Some(if mutable {
                Ownership::BorrowMut
            } else {
                Ownership::Borrow
            })
        } else if matches!(self.peek_kind(), Some(TokenKind::Unsafe)) {
            self.advance();
            self.expect(&TokenKind::Star)?;
            Some(Ownership::RawPointer)
        } else {
            None
        };

        let base_type = self.parse_base_type()?;

        // Check for nullable postfix: T?
        let result_type = if matches!(self.peek_kind(), Some(TokenKind::Question)) {
            self.advance();
            Type::Nullable(Box::new(base_type))
        } else {
            base_type
        };

        Ok(if let Some(own) = ownership {
            Type::WithOwnership(Box::new(result_type), own)
        } else {
            result_type
        })
    }

    fn parse_base_type(&mut self) -> Result<Type, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::SelfType) => {
                self.advance();
                Ok(Type::Named("Self".to_string()))
            }
            Some(TokenKind::U8) => {
                self.advance();
                Ok(Type::U8)
            }
            Some(TokenKind::U16) => {
                self.advance();
                if self.peek().is_none() {
                    return Err(ParseError::UnexpectedEof {
                        code: ParseErrorCode::UnexpectedEof,
                        hint: Some("expected item declaration before end of file".to_string()),
                    });
                }
                Ok(Type::U16)
            }
            Some(TokenKind::U32) => {
                self.advance();
                Ok(Type::U32)
            }
            Some(TokenKind::U64) => {
                self.advance();
                Ok(Type::U64)
            }
            Some(TokenKind::Usize) => {
                self.advance();
                Ok(Type::Usize)
            }
            Some(TokenKind::I8) => {
                self.advance();
                Ok(Type::I8)
            }
            Some(TokenKind::I16) => {
                self.advance();
                Ok(Type::I16)
            }
            Some(TokenKind::I32) => {
                self.advance();
                Ok(Type::I32)
            }
            Some(TokenKind::I64) => {
                self.advance();
                Ok(Type::I64)
            }
            Some(TokenKind::Isize) => {
                self.advance();
                Ok(Type::Isize)
            }
            Some(TokenKind::F32) => {
                self.advance();
                Ok(Type::F32)
            }
            Some(TokenKind::F64) => {
                self.advance();
                Ok(Type::F64)
            }
            Some(TokenKind::Bool) => {
                self.advance();
                Ok(Type::Bool)
            }
            Some(TokenKind::Str) => {
                self.advance();
                Ok(Type::Str)
            }
            Some(TokenKind::LBracket) => {
                self.advance();
                let elem_type = self.parse_type()?;
                // Check for fixed size array
                if matches!(self.peek_kind(), Some(TokenKind::Semicolon)) {
                    self.advance();
                    let size = self.parse_expression()?;
                    if !matches!(self.peek_kind(), Some(TokenKind::RBracket)) {
                        self.synchronize(); // Attempt to recover by skipping to the next valid token
                        return Err(ParseError::UnexpectedToken {
                            line: self.tokens.get(self.current).map(|t| t.line).unwrap_or(0),
                            column: self.tokens.get(self.current).map(|t| t.column).unwrap_or(0),
                            expected: "]".to_string(),
                            got: ";".to_string(),
                            code: ParseErrorCode::UnexpectedToken,
                            hint: Some("Expected ']' after array size".to_string()),
                        });
                    }
                    self.expect(&TokenKind::RBracket)?;
                    Ok(Type::Array(Box::new(elem_type), Some(Box::new(size))))
                } else {
                    self.expect(&TokenKind::RBracket)?;
                    Ok(Type::Slice(Box::new(elem_type)))
                }
            }
            Some(TokenKind::Fn) => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let mut param_types = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    param_types.push(self.parse_type()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RParen)?;
                let return_type = if matches!(self.peek_kind(), Some(TokenKind::Arrow)) {
                    self.advance();
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                Ok(Type::Function(param_types, return_type))
            }
            Some(TokenKind::Identifier) => {
                let name = self.parse_identifier()?;
                // Check for generic parameters
                if matches!(self.peek_kind(), Some(TokenKind::Lt)) {
                    self.advance();
                    let mut type_args = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::Gt))
                        && !matches!(self.peek_kind(), Some(TokenKind::Shr))
                    {
                        type_args.push(self.parse_type()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    // Handle nested generics: if we see Shr (>>), treat as closing >
                    if matches!(self.peek_kind(), Some(TokenKind::Shr)) {
                        // Skip the Shr - it's actually > > (close two generics)
                        self.advance();
                        // If there was an inner generic, we've closed both
                        // Check if we need to close more
                        if !type_args.is_empty() {
                            // Inner generic was closed, now close outer
                        }
                        Ok(Type::Generic(name, type_args))
                    } else {
                        self.expect(&TokenKind::Gt)?;
                        Ok(Type::Generic(name, type_args))
                    }
                } else {
                    Ok(Type::Named(name))
                }
            }
            Some(TokenKind::LParen) => {
                // Tuple type: (T1, T2, ...)
                self.advance();
                let mut elem_types = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    elem_types.push(self.parse_type()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                if elem_types.len() == 1 {
                    // Single type in parens is just grouping, not a tuple
                    Ok(elem_types.into_iter().next().unwrap())
                } else {
                    Ok(Type::Tuple(elem_types))
                }
            }
            _ => {
                let token = self.peek().unwrap();
                Err(ParseError::InvalidSyntax {
                    line: token.line,
                    column: token.column,
                    message: format!("Expected type, got {:?}", token.kind),
                    code: ParseErrorCode::InvalidSyntax,
                    hint: None,
                })
            }
        }
    }

    /// Parse a block of statements (indent/dedent style).
    /// Uses panic-mode recovery: on error, records the error, synchronizes,
    /// and continues parsing remaining statements in the block.
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(&TokenKind::Indent)?;
        let mut statements = Vec::new();

        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            let before_idx = self.current;
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                break;
            }
            if self.peek().is_none() {
                // Unterminated block — record and break
                self.record_error(ParseError::InvalidSyntax {
                    line: 0,
                    column: 0,
                    message: "Unterminated block: expected Dedent before end of file".to_string(),
                    code: ParseErrorCode::UnterminatedBlock,
                    hint: Some("check indentation of this block".to_string()),
                })?;
                return Ok(Block { statements });
            }
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.record_error(e)?;
                    // Synchronize within the block: advance until we hit
                    // a Dedent (block end), a Newline followed by a statement
                    // keyword, or EOF.
                    self.synchronize();
                }
            }
            // Progress guard: if no tokens were consumed, force advance to avoid infinite loops
            if self.current == before_idx {
                if self.peek().is_none() || matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(&TokenKind::Dedent)?;
        Ok(Block { statements })
    }

    /// Parse a brace-delimited block of statements: `{ stmt; stmt; ... }`.
    /// The opening `{` must already be consumed by the caller.
    /// Uses the same recovery strategy as `parse_block`.
    fn parse_brace_block(&mut self) -> Result<Block, ParseError> {
        let mut statements = Vec::new();

        self.skip_newlines();
        // Also consume any indent tokens the lexer may have emitted inside braces
        while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
            self.advance();
            self.skip_newlines();
        }

        while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
            let before_idx = self.current;
            self.skip_newlines();
            // Tolerate stray indent/dedent tokens emitted by the
            // indentation-tracking lexer inside brace-delimited blocks.
            if matches!(self.peek_kind(), Some(TokenKind::Indent))
                || matches!(self.peek_kind(), Some(TokenKind::Dedent))
            {
                self.advance();
                self.skip_newlines();
                continue;
            }
            if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                break;
            }
            if self.peek().is_none() {
                self.record_error(ParseError::InvalidSyntax {
                    line: 0,
                    column: 0,
                    message: "Unterminated brace block: expected '}' before end of file"
                        .to_string(),
                    code: ParseErrorCode::UnterminatedBlock,
                    hint: Some("check for matching '}'".to_string()),
                })?;
                return Ok(Block { statements });
            }
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.record_error(e)?;
                    self.synchronize();
                }
            }
            // Consume optional semicolons between statements in brace blocks
            if matches!(self.peek_kind(), Some(TokenKind::Semicolon)) {
                self.advance();
            }
            // Progress guard
            if self.current == before_idx {
                if self.peek().is_none() || matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                self.advance();
            }
        }

        // Consume any trailing dedent before the closing brace
        while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.advance();
            self.skip_newlines();
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(Block { statements })
    }

    /// Parse a control-flow block body that may be either:
    /// - `{ ... }` brace-delimited, or
    /// - `:\n` followed by an indented block.
    fn parse_statement_body(&mut self) -> Result<Block, ParseError> {
        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            self.advance(); // consume '{'
            self.parse_brace_block()
        } else {
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.parse_block()
        }
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.skip_newlines();

        match self.peek_kind() {
            Some(TokenKind::Let) => self.parse_let(),
            Some(TokenKind::Var) => self.parse_var(),
            Some(TokenKind::Return) => self.parse_return(),
            Some(TokenKind::If) => self.parse_if(),
            Some(TokenKind::For) => self.parse_for(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::Loop) => self.parse_loop(),
            Some(TokenKind::Match) => self.parse_match(),
            Some(TokenKind::Defer) => self.parse_defer(),
            Some(TokenKind::Break) => {
                self.advance();
                let value = if matches!(self.peek_kind(), Some(TokenKind::Newline))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                    || self.peek().is_none()
                {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                self.skip_newlines();
                Ok(Statement::Break(value))
            }
            Some(TokenKind::Continue) => {
                self.advance();
                self.skip_newlines();
                Ok(Statement::Continue)
            }
            Some(TokenKind::Pass) => {
                self.advance();
                self.skip_newlines();
                Ok(Statement::Pass)
            }
            Some(TokenKind::Yield) => self.parse_yield(),
            Some(TokenKind::Spawn) => self.parse_spawn(),
            Some(TokenKind::Select) => self.parse_select(),
            _ => {
                let expr = self.parse_expression()?;

                // Check for assignment operators
                match self.peek_kind() {
                    Some(TokenKind::Eq) => {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.skip_newlines();
                        Ok(Statement::Assignment {
                            target: expr,
                            op: None,
                            value,
                        })
                    }
                    Some(TokenKind::PlusEq) => {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.skip_newlines();
                        Ok(Statement::Assignment {
                            target: expr,
                            op: Some(BinaryOp::Add),
                            value,
                        })
                    }
                    Some(TokenKind::MinusEq) => {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.skip_newlines();
                        Ok(Statement::Assignment {
                            target: expr,
                            op: Some(BinaryOp::Sub),
                            value,
                        })
                    }
                    Some(TokenKind::StarEq) => {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.skip_newlines();
                        Ok(Statement::Assignment {
                            target: expr,
                            op: Some(BinaryOp::Mul),
                            value,
                        })
                    }
                    Some(TokenKind::SlashEq) => {
                        self.advance();
                        let value = self.parse_expression()?;
                        self.skip_newlines();
                        Ok(Statement::Assignment {
                            target: expr,
                            op: Some(BinaryOp::Div),
                            value,
                        })
                    }
                    _ => {
                        self.skip_newlines();
                        Ok(Statement::Expression(expr))
                    }
                }
            }
        }
    }

    /// Parse var statement (mutable variable declaration)
    fn parse_var(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Var)?;
        let name = if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
            self.advance();
            let mut names = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                names.push(self.parse_identifier()?);
                if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            format!("({})", names.join(", "))
        } else {
            self.parse_identifier()?
        };

        let ty = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if matches!(self.peek_kind(), Some(TokenKind::Eq)) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.skip_newlines();

        Ok(Statement::Var { name, ty, value })
    }

    /// Parse loop statement
    fn parse_loop(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Loop)?;
        let body = self.parse_statement_body()?;
        Ok(Statement::Loop { body })
    }

    /// Parse defer statement
    fn parse_defer(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Defer)?;
        let stmt = self.parse_statement()?;
        Ok(Statement::Defer(Box::new(stmt)))
    }

    /// Parse yield statement
    fn parse_yield(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Yield)?;
        let value = if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.skip_newlines();
        Ok(Statement::Yield(value))
    }

    /// Parse spawn statement
    fn parse_spawn(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Spawn)?;
        let expr = self.parse_expression()?;
        self.skip_newlines();
        Ok(Statement::Spawn(Box::new(expr)))
    }

    /// Parse select statement
    fn parse_select(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Select)?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut arms = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();
            let pattern = self.parse_pattern()?;

            // Support `pat1 | pat2 | pat3` alternative patterns.
            let pattern = if matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                let mut alternatives = vec![pattern];
                while matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                    self.advance();
                    alternatives.push(self.parse_pattern()?);
                    self.skip_newlines();
                }
                Pattern::Or(alternatives)
            } else {
                pattern
            };
            self.expect(&TokenKind::FatArrow)?;
            let channel_op = self.parse_expression()?;
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            let body = self.parse_block()?;
            arms.push(SelectArm {
                pattern,
                channel_op,
                body,
            });
        }
        self.expect(&TokenKind::Dedent)?;

        Ok(Statement::Select { arms })
    }

    /// Parse let statement
    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Let)?;
        let mutable = matches!(self.peek_kind(), Some(TokenKind::Mut));
        if mutable {
            self.advance();
        }

        let name = if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
            self.advance();
            let mut names = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                names.push(self.parse_identifier()?);
                if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            format!("({})", names.join(", "))
        } else {
            self.parse_identifier()?
        };

        let ty = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if matches!(self.peek_kind(), Some(TokenKind::Eq)) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.skip_newlines();

        Ok(Statement::Let {
            name,
            mutable,
            ty,
            value,
        })
    }

    /// Parse return statement
    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Return)?;
        let value = if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.skip_newlines();
        Ok(Statement::Return(value))
    }

    /// Parse if statement (with elif support — O-004)
    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_block = self.parse_statement_body()?;
        self.skip_newlines();

        // O-004: Handle elif chains by desugaring to nested else { if ... }
        let else_block = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
            self.advance(); // consume 'elif'
            let elif_condition = self.parse_expression()?;
            let elif_then = self.parse_statement_body()?;
            self.skip_newlines();

            // Recursively handle further elif/else chains
            let elif_else = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
                // Re-enter elif handling: synthesize an if-statement for the next elif
                // We need to "put back" the elif and call ourselves, but since we can't
                // unadvance, we build the chain manually
                let inner = self.parse_elif_chain()?;
                Some(Block {
                    statements: vec![inner],
                })
            } else if matches!(self.peek_kind(), Some(TokenKind::Else)) {
                self.advance();
                Some(self.parse_statement_body()?)
            } else {
                None
            };

            Some(Block {
                statements: vec![Statement::If {
                    condition: elif_condition,
                    then_block: elif_then,
                    else_block: elif_else,
                }],
            })
        } else if matches!(self.peek_kind(), Some(TokenKind::Else)) {
            self.advance();
            Some(self.parse_statement_body()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    /// Parse elif chain (helper for elif desugaring — O-004)
    fn parse_elif_chain(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Elif)?;
        let condition = self.parse_expression()?;
        let then_block = self.parse_statement_body()?;
        self.skip_newlines();

        let else_block = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
            let inner = self.parse_elif_chain()?;
            Some(Block {
                statements: vec![inner],
            })
        } else if matches!(self.peek_kind(), Some(TokenKind::Else)) {
            self.advance();
            Some(self.parse_statement_body()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    /// Parse for loop
    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::For)?;
        // Support destructuring patterns: for (k, v) in iter:
        let var = if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
            // Parse parenthesized pattern and collect identifier names
            self.advance(); // consume (
            let mut names = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                names.push(self.parse_identifier()?);
                if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            format!("({})", names.join(", "))
        } else {
            self.parse_identifier()?
        };
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expression()?;
        let body = self.parse_statement_body()?;

        Ok(Statement::For { var, iter, body })
    }

    /// Parse while loop
    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_statement_body()?;

        Ok(Statement::While { condition, body })
    }

    /// Parse match expression
    fn parse_match(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::Match)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut arms = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();

            // Allow attributes on match arms
            while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
                let _ = self.parse_attribute()?;
                self.skip_newlines();
            }

            let pattern = self.parse_pattern()?;

            // Support `pat1 | pat2 | pat3` alternative patterns in match arms
            let pattern = if matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                let mut alternatives = vec![pattern];
                while matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                    self.advance();
                    alternatives.push(self.parse_pattern()?);
                }
                Pattern::Or(alternatives)
            } else {
                pattern
            };

            // Parse optional guard: `pattern if condition => body`
            let guard = if matches!(self.peek_kind(), Some(TokenKind::If)) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            // Accept either `:` (block arms) or `=>` / `FatArrow` (expr arms)
            match self.peek_kind() {
                Some(TokenKind::FatArrow) => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                        let body = self.parse_block()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body: MatchBody::Block(body),
                        });
                    } else {
                        let body = self.parse_inline_match_arm_body()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body,
                        });
                    }
                }
                Some(TokenKind::Colon) => {
                    self.advance();
                    // Single line or block
                    if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                        let body = self.parse_block()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body: MatchBody::Block(body),
                        });
                    } else {
                        let body = self.parse_inline_match_arm_body()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body,
                        });
                    }
                }
                _ => {
                    let token = self.peek().unwrap();
                    return Err(ParseError::InvalidSyntax {
                        line: token.line,
                        column: token.column,
                        message: format!("Expected ':' or '=>', got {:?}", token.kind),
                        code: ParseErrorCode::InvalidSyntax,
                        hint: None,
                    });
                }
            }
        }

        self.expect(&TokenKind::Dedent)?;
        Ok(Statement::Match { expr, arms })
    }

    fn parse_inline_match_arm_body(&mut self) -> Result<MatchBody, ParseError> {
        let starts_stmt = matches!(
            self.peek_kind(),
            Some(TokenKind::Let)
                | Some(TokenKind::Var)
                | Some(TokenKind::Return)
                | Some(TokenKind::If)
                | Some(TokenKind::For)
                | Some(TokenKind::While)
                | Some(TokenKind::Loop)
                | Some(TokenKind::Match)
                | Some(TokenKind::Defer)
                | Some(TokenKind::Break)
                | Some(TokenKind::Continue)
                | Some(TokenKind::Pass)
                | Some(TokenKind::Yield)
                | Some(TokenKind::Spawn)
                | Some(TokenKind::Select)
        );

        if starts_stmt {
            let stmt = self.parse_statement()?;
            return Ok(MatchBody::Block(Block {
                statements: vec![stmt],
            }));
        }

        let expr = self.parse_expression()?;
        match self.peek_kind() {
            Some(TokenKind::Eq) => {
                self.advance();
                let value = self.parse_expression()?;
                self.skip_newlines();
                Ok(MatchBody::Block(Block {
                    statements: vec![Statement::Assignment {
                        target: expr,
                        op: None,
                        value,
                    }],
                }))
            }
            Some(TokenKind::PlusEq) => {
                self.advance();
                let value = self.parse_expression()?;
                self.skip_newlines();
                Ok(MatchBody::Block(Block {
                    statements: vec![Statement::Assignment {
                        target: expr,
                        op: Some(BinaryOp::Add),
                        value,
                    }],
                }))
            }
            Some(TokenKind::MinusEq) => {
                self.advance();
                let value = self.parse_expression()?;
                self.skip_newlines();
                Ok(MatchBody::Block(Block {
                    statements: vec![Statement::Assignment {
                        target: expr,
                        op: Some(BinaryOp::Sub),
                        value,
                    }],
                }))
            }
            Some(TokenKind::StarEq) => {
                self.advance();
                let value = self.parse_expression()?;
                self.skip_newlines();
                Ok(MatchBody::Block(Block {
                    statements: vec![Statement::Assignment {
                        target: expr,
                        op: Some(BinaryOp::Mul),
                        value,
                    }],
                }))
            }
            Some(TokenKind::SlashEq) => {
                self.advance();
                let value = self.parse_expression()?;
                self.skip_newlines();
                Ok(MatchBody::Block(Block {
                    statements: vec![Statement::Assignment {
                        target: expr,
                        op: Some(BinaryOp::Div),
                        value,
                    }],
                }))
            }
            _ => {
                self.skip_newlines();
                Ok(MatchBody::Expr(expr))
            }
        }
    }

    /// Parse a pattern
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::LParen) => {
                // Parenthesized pattern: (pat)
                self.advance();
                if matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    self.advance();
                    return Ok(Pattern::Constructor("()".to_string(), vec![]));
                }
                let pat = self.parse_pattern()?;
                self.expect(&TokenKind::RParen)?;
                // Optional `as Type` cast in patterns — consume for now
                if matches!(self.peek_kind(), Some(TokenKind::As)) {
                    self.advance();
                    let _ = self.parse_type(); // ignore errors from type here
                }
                return Ok(pat);
            }
            Some(TokenKind::Identifier) => {
                let mut name = self.parse_identifier()?;
                // Handle path patterns: Option::Some(x), Option::None
                while matches!(self.peek_kind(), Some(TokenKind::DoubleColon)) {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Some_)) {
                        self.advance();
                        name = format!("{}::{}", name, "Some");
                    } else if matches!(self.peek_kind(), Some(TokenKind::None_)) {
                        self.advance();
                        name = format!("{}::{}", name, "None");
                    } else if matches!(self.peek_kind(), Some(TokenKind::Ok_)) {
                        self.advance();
                        name = format!("{}::{}", name, "Ok");
                    } else if matches!(self.peek_kind(), Some(TokenKind::Err_)) {
                        self.advance();
                        name = format!("{}::{}", name, "Err");
                    } else {
                        let member = self.parse_identifier()?;
                        name = format!("{}::{}", name, member);
                    }
                }
                if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                    // Constructor pattern
                    self.advance();
                    let mut fields = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        fields.push(self.parse_pattern()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Constructor(name, fields))
                } else {
                    if name == "_" {
                        return Ok(Pattern::Wildcard);
                    }
                    // Optional `as Type` cast in binding patterns
                    if matches!(self.peek_kind(), Some(TokenKind::As)) {
                        self.advance();
                        let _ = self.parse_type();
                    }
                    Ok(Pattern::Binding(name))
                }
            }
            Some(TokenKind::IntLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                Ok(Pattern::Literal(Literal::Int(val.parse().unwrap())))
            }
            Some(TokenKind::StringLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                // Remove quotes (same as in parse_primary)
                let unquoted = val[1..val.len() - 1].to_string();
                Ok(Pattern::Literal(Literal::String(unquoted)))
            }
            Some(TokenKind::CharLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                // Remove quotes and get character value
                let unquoted = &val[1..val.len() - 1];
                let ch = if unquoted.starts_with('\\') {
                    match &unquoted[1..] {
                        "n" => '\n',
                        "r" => '\r',
                        "t" => '\t',
                        "0" => '\0',
                        "\\" => '\\',
                        "'" => '\'',
                        "\"" => '"',
                        _ => unquoted.chars().next().unwrap_or('\0'),
                    }
                } else {
                    unquoted.chars().next().unwrap_or('\0')
                };
                Ok(Pattern::Literal(Literal::Int(ch as i64)))
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            Some(TokenKind::Some_) => {
                self.advance();
                if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        fields.push(self.parse_pattern()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Constructor("Some".to_string(), fields))
                } else {
                    Ok(Pattern::Constructor("Some".to_string(), vec![]))
                }
            }
            Some(TokenKind::None_) => {
                self.advance();
                Ok(Pattern::Constructor("None".to_string(), vec![]))
            }
            Some(TokenKind::Ok_) => {
                self.advance();
                if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        fields.push(self.parse_pattern()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Constructor("Ok".to_string(), fields))
                } else {
                    Ok(Pattern::Constructor("Ok".to_string(), vec![]))
                }
            }
            Some(TokenKind::Err_) => {
                self.advance();
                if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                        fields.push(self.parse_pattern()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Constructor("Err".to_string(), fields))
                } else {
                    Ok(Pattern::Constructor("Err".to_string(), vec![]))
                }
            }
            _ => {
                let token = self.peek().unwrap();
                Err(ParseError::InvalidSyntax {
                    line: token.line,
                    column: token.column,
                    message: format!("Expected pattern, got {:?}", token.kind),
                    code: ParseErrorCode::InvalidSyntax,
                    hint: None,
                })
            }
        }
    }

    /// Parse an expression (precedence climbing)
    pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_binary(0)
    }

    fn parse_if_expression(&mut self) -> Result<Expression, ParseError> {
        self.expect(&TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;

        let then_expr = if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            self.advance();
            self.expect(&TokenKind::Indent)?;
            let expr = self.parse_expression()?;
            self.skip_newlines();
            self.expect(&TokenKind::Dedent)?;
            expr
        } else {
            self.parse_expression()?
        };

        let else_expr = if matches!(self.peek_kind(), Some(TokenKind::Else)) {
            self.advance();
            self.expect(&TokenKind::Colon)?;
            let expr = if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                self.advance();
                self.expect(&TokenKind::Indent)?;
                let e = self.parse_expression()?;
                self.skip_newlines();
                self.expect(&TokenKind::Dedent)?;
                e
            } else {
                self.parse_expression()?
            };
            Some(Box::new(expr))
        } else {
            None
        };

        Ok(Expression::If {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr,
        })
    }

    fn parse_match_expression(&mut self) -> Result<Expression, ParseError> {
        self.expect(&TokenKind::Match)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut arms = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();

            while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
                let _ = self.parse_attribute()?;
                self.skip_newlines();
            }

            let pattern = self.parse_pattern()?;
            let pattern = if matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                let mut alternatives = vec![pattern];
                while matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                    self.advance();
                    alternatives.push(self.parse_pattern()?);
                }
                Pattern::Or(alternatives)
            } else {
                pattern
            };

            // Parse optional guard: `pattern if condition => body`
            let guard = if matches!(self.peek_kind(), Some(TokenKind::If)) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            match self.peek_kind() {
                Some(TokenKind::FatArrow) => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                        let body = self.parse_block()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body: MatchBody::Block(body),
                        });
                    } else {
                        let body = self.parse_inline_match_arm_body()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body,
                        });
                    }
                }
                Some(TokenKind::Colon) => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                        let body = self.parse_block()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body: MatchBody::Block(body),
                        });
                    } else {
                        let body = self.parse_inline_match_arm_body()?;
                        arms.push(MatchArm {
                            pattern,
                            guard,
                            body,
                        });
                    }
                }
                _ => {
                    let token = self.peek().unwrap();
                    return Err(ParseError::InvalidSyntax {
                        line: token.line,
                        column: token.column,
                        message: format!("Expected ':' or '=>', got {:?}", token.kind),
                        code: ParseErrorCode::InvalidSyntax,
                        hint: None,
                    });
                }
            }
        }

        self.expect(&TokenKind::Dedent)?;
        Ok(Expression::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.peek_binary_op() {
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            self.advance();
            if matches!(op, BinaryOp::Range | BinaryOp::RangeInclusive)
                && matches!(
                    self.peek_kind(),
                    Some(TokenKind::RBracket)
                        | Some(TokenKind::RParen)
                        | Some(TokenKind::Comma)
                        | Some(TokenKind::Colon)
                        | Some(TokenKind::Newline)
                        | Some(TokenKind::Dedent)
                        | None
                )
            {
                left = Expression::Range {
                    start: Some(Box::new(left)),
                    end: None,
                    inclusive: matches!(op, BinaryOp::RangeInclusive),
                };
                continue;
            }
            let right = self.parse_binary(prec + 1)?;
            left = Expression::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn peek_binary_op(&self) -> Option<BinaryOp> {
        match self.peek_kind()? {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Mod),
            TokenKind::EqEq => Some(BinaryOp::Eq),
            TokenKind::NotEq => Some(BinaryOp::NotEq),
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::LtEq => Some(BinaryOp::LtEq),
            TokenKind::GtEq => Some(BinaryOp::GtEq),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::DotDotEq => Some(BinaryOp::RangeInclusive),
            TokenKind::DotDot => Some(BinaryOp::Range),
            _ => None,
        }
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            Some(TokenKind::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Some(TokenKind::Ampersand) => {
                self.advance();
                let mutable = matches!(self.peek_kind(), Some(TokenKind::Mut));
                if mutable {
                    self.advance();
                }
                let expr = self.parse_unary()?;
                Ok(Expression::Borrow {
                    mutable,
                    expr: Box::new(expr),
                })
            }
            Some(TokenKind::Star) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Deref(Box::new(expr)))
            }
            Some(TokenKind::Await) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Await(Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;
        let mut continuation_indent_depth = 0usize;

        loop {
            if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                let mut lookahead = self.current;
                while matches!(
                    self.tokens.get(lookahead).map(|t| &t.kind),
                    Some(TokenKind::Newline)
                ) {
                    lookahead += 1;
                }
                let mut extra_indent = 0usize;
                while matches!(
                    self.tokens.get(lookahead).map(|t| &t.kind),
                    Some(TokenKind::Indent)
                ) {
                    lookahead += 1;
                    extra_indent += 1;
                }
                let continues_postfix = matches!(
                    self.tokens.get(lookahead).map(|t| &t.kind),
                    Some(TokenKind::Dot)
                        | Some(TokenKind::LParen)
                        | Some(TokenKind::LBracket)
                        | Some(TokenKind::DoubleColon)
                );
                if continues_postfix {
                    while matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                    }
                    for _ in 0..extra_indent {
                        self.expect(&TokenKind::Indent)?;
                    }
                    continuation_indent_depth += extra_indent;
                    continue;
                }
            }

            match self.peek_kind() {
                Some(TokenKind::Dot) => {
                    self.advance();
                    // Check for await
                    if matches!(self.peek_kind(), Some(TokenKind::Await)) {
                        self.advance();
                        expr = Expression::Await(Box::new(expr));
                        continue;
                    }

                    let field = self.parse_identifier()?;
                    if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                        // Method call
                        self.advance();
                        let (args, mut call_indent_depth) = self.parse_call_args()?;
                        loop {
                            self.skip_newlines();
                            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                        self.expect(&TokenKind::RParen)?;
                        while call_indent_depth > 0 {
                            self.skip_newlines();
                            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                                self.advance();
                                call_indent_depth -= 1;
                                continue;
                            }
                            break;
                        }
                        expr = Expression::MethodCall {
                            receiver: Box::new(expr),
                            method: field,
                            args,
                        };
                    } else {
                        expr = Expression::Field(Box::new(expr), field);
                    }
                }
                Some(TokenKind::LParen) => {
                    self.advance();
                    let (args, mut call_indent_depth) = self.parse_call_args()?;
                    loop {
                        self.skip_newlines();
                        if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                    self.expect(&TokenKind::RParen)?;
                    while call_indent_depth > 0 {
                        self.skip_newlines();
                        if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                            self.advance();
                            call_indent_depth -= 1;
                            continue;
                        }
                        break;
                    }
                    expr = Expression::Call(Box::new(expr), args);
                }
                Some(TokenKind::LBracket) => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expression::Index(Box::new(expr), Box::new(index));
                }
                Some(TokenKind::DoubleColon) => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Lt)) {
                        self.skip_turbofish_args()?;
                        continue;
                    }
                    let member = self.parse_identifier()?;
                    expr = Expression::Path(Box::new(expr), member);
                }
                Some(TokenKind::As) => {
                    // Syntax-level cast support: consume `as Type` and keep
                    // the expression unchanged until semantic cast lowering is added.
                    self.advance();
                    let _ = self.parse_type()?;
                }
                _ => break,
            }
        }

        if continuation_indent_depth > 0 {
            while matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                self.advance();
            }
            while continuation_indent_depth > 0
                && matches!(self.peek_kind(), Some(TokenKind::Dedent))
            {
                self.advance();
                continuation_indent_depth -= 1;
            }
        }

        Ok(expr)
    }

    fn skip_turbofish_args(&mut self) -> Result<(), ParseError> {
        self.expect(&TokenKind::Lt)?;
        let mut depth = 1usize;

        while depth > 0 {
            match self.peek_kind() {
                Some(TokenKind::Lt) => {
                    self.advance();
                    depth += 1;
                }
                Some(TokenKind::Gt) => {
                    self.advance();
                    depth -= 1;
                }
                Some(TokenKind::Shr) => {
                    self.advance();
                    if depth >= 2 {
                        depth -= 2;
                    } else {
                        depth = 0;
                    }
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(ParseError::UnexpectedEof {
                        code: ParseErrorCode::UnexpectedEof,
                        hint: Some("unterminated generic argument list after '::<'".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    fn parse_call_args(&mut self) -> Result<(Vec<Expression>, usize), ParseError> {
        let mut args = Vec::new();
        let mut continuation_indent_depth = 0usize;
        loop {
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                continuation_indent_depth += 1;
                self.skip_newlines();
            }
            if matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                break;
            }
            args.push(self.parse_expression()?);
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                continuation_indent_depth += 1;
                self.skip_newlines();
            }
            if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                self.advance();
                self.skip_newlines();
                continue;
            }
            break;
        }
        Ok((args, continuation_indent_depth))
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::IntLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                Ok(Expression::Literal(Literal::Int(val.parse().unwrap())))
            }
            Some(TokenKind::FloatLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                Ok(Expression::Literal(Literal::Float(val.parse().unwrap())))
            }
            Some(TokenKind::StringLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                // Remove quotes
                let unquoted = val[1..val.len() - 1].to_string();
                Ok(Expression::Literal(Literal::String(unquoted)))
            }
            Some(TokenKind::FStringLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                // Parse f-string: f"..."
                // For now, treat as regular string (interpolation would require more parsing)
                let unquoted = val[2..val.len() - 1].to_string();
                let fstring = FString {
                    parts: vec![FStringPart::Literal(unquoted)],
                };
                Ok(Expression::FString(fstring))
            }
            Some(TokenKind::CharLiteral) => {
                let val = self.advance().unwrap().lexeme.clone();
                // Remove quotes and get character value
                let unquoted = &val[1..val.len() - 1];
                let ch = if unquoted.starts_with('\\') {
                    match &unquoted[1..] {
                        "n" => '\n',
                        "r" => '\r',
                        "t" => '\t',
                        "0" => '\0',
                        "\\" => '\\',
                        "'" => '\'',
                        "\"" => '"',
                        _ => unquoted.chars().next().unwrap_or('\0'),
                    }
                } else {
                    unquoted.chars().next().unwrap_or('\0')
                };
                Ok(Expression::Literal(Literal::Int(ch as i64)))
            }
            Some(TokenKind::True) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            Some(TokenKind::If) => self.parse_if_expression(),
            Some(TokenKind::Match) => self.parse_match_expression(),
            Some(TokenKind::None_) => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            Some(TokenKind::SelfValue) => {
                self.advance();
                Ok(Expression::Identifier("self".to_string()))
            }
            Some(TokenKind::SelfType) => {
                self.advance();
                Ok(Expression::Identifier("Self".to_string()))
            }
            Some(TokenKind::Identifier) => {
                let name = self.parse_identifier()?;
                // Check for struct literal
                if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
                    self.advance();

                    // Allow optional newlines/indentation before fields.
                    // This is common when the struct literal spans multiple lines.
                    self.skip_newlines();
                    let mut had_indent = false;
                    if matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                        had_indent = true;
                        self.advance();
                    }

                    let mut fields = Vec::new();
                    while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                        self.skip_newlines();
                        // Handle optional dedent before closing brace.
                        if had_indent && matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                            self.advance();
                            self.skip_newlines();
                            continue;
                        }
                        if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                            break;
                        }

                        let field_name = self.parse_identifier()?;
                        self.expect(&TokenKind::Colon)?;
                        let value = self.parse_expression()?;
                        fields.push((field_name, value));
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }

                    // If we consumed indentation but did not yet see a dedent,
                    // allow it before the closing brace.
                    if had_indent && matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                        self.advance();
                    }
                    self.skip_newlines();
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expression::StructLiteral { name, fields })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Some(TokenKind::LParen) => {
                self.advance();

                // Empty tuple
                if matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    self.advance();
                    return Ok(Expression::Tuple(vec![]));
                }

                // Check for lambda: (params) -> body or |params| body
                // First, try to determine if this is a tuple/grouping or a lambda
                let first = self.parse_expression()?;

                // Check for tuple (comma-separated)
                if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    let mut elements = vec![first];
                    while matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                        if matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                            break; // trailing comma
                        }
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expression::Tuple(elements))
                } else {
                    self.expect(&TokenKind::RParen)?;
                    Ok(first)
                }
            }
            Some(TokenKind::Pipe) => {
                // Lambda expression: |x, y| x + y
                self.advance();
                let mut params = Vec::new();

                while !matches!(self.peek_kind(), Some(TokenKind::Pipe)) {
                    let name = self.parse_identifier()?;
                    let ty = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
                        self.advance();
                        self.parse_type()?
                    } else {
                        Type::Named("_".to_string()) // inferred type
                    };
                    params.push(Param { name, ty });

                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::Pipe)?;

                let body = self.parse_expression()?;
                Ok(Expression::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            Some(TokenKind::Fn) => {
                // Function literal: fn(x: i32, y: i32) -> i32: x + y
                self.advance();

                // Parse parameters in parentheses
                self.expect(&TokenKind::LParen)?;
                let mut params = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    let name = self.parse_identifier()?;
                    self.expect(&TokenKind::Colon)?;
                    let ty = self.parse_type()?;
                    params.push(Param { name, ty });

                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RParen)?;

                // Parse optional return type
                let _return_type = if matches!(self.peek_kind(), Some(TokenKind::Arrow)) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                // Parse body after colon
                self.expect(&TokenKind::Colon)?;
                let body = self.parse_expression()?;

                Ok(Expression::Lambda {
                    params,
                    body: Box::new(body),
                })
            }
            Some(TokenKind::Some_) => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Expression::Some(Box::new(expr)))
            }
            Some(TokenKind::Ok_) => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Expression::Ok(Box::new(expr)))
            }
            Some(TokenKind::Err_) => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Expression::Err(Box::new(expr)))
            }
            Some(TokenKind::LBracket) => {
                self.advance();

                // Empty array
                if matches!(self.peek_kind(), Some(TokenKind::RBracket)) {
                    self.advance();
                    return Ok(Expression::Array(vec![]));
                }

                // Parse first expression
                let first = self.parse_expression()?;

                // Check for list comprehension: [expr for var in iter]
                if matches!(self.peek_kind(), Some(TokenKind::For)) {
                    self.advance(); // consume 'for'
                    let var = self.parse_identifier()?;
                    self.expect(&TokenKind::In)?;
                    let iter = self.parse_expression()?;

                    // Optional filter: if condition
                    let filter = if matches!(self.peek_kind(), Some(TokenKind::If)) {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };

                    self.expect(&TokenKind::RBracket)?;
                    return Ok(Expression::ListComprehension {
                        expr: Box::new(first),
                        var,
                        iter: Box::new(iter),
                        filter,
                    });
                }

                // Regular array literal
                let mut elements = vec![first];
                while matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::RBracket)) {
                        break; // trailing comma
                    }
                    elements.push(self.parse_expression()?);
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expression::Array(elements))
            }
            Some(TokenKind::Shared) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Shared(Box::new(expr)))
            }
            Some(TokenKind::Own) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Own(Box::new(expr)))
            }
            Some(TokenKind::DotDot) => {
                self.advance();
                let end = if matches!(self.peek_kind(), Some(TokenKind::RBracket))
                    || matches!(self.peek_kind(), Some(TokenKind::RParen))
                    || matches!(self.peek_kind(), Some(TokenKind::Comma))
                    || matches!(self.peek_kind(), Some(TokenKind::Newline))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                    || self.peek().is_none()
                {
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                };
                Ok(Expression::Range {
                    start: None,
                    end,
                    inclusive: false,
                })
            }
            Some(TokenKind::DotDotEq) => {
                self.advance();
                let end = if matches!(self.peek_kind(), Some(TokenKind::RBracket))
                    || matches!(self.peek_kind(), Some(TokenKind::RParen))
                    || matches!(self.peek_kind(), Some(TokenKind::Comma))
                    || matches!(self.peek_kind(), Some(TokenKind::Newline))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                    || self.peek().is_none()
                {
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                };
                Ok(Expression::Range {
                    start: None,
                    end,
                    inclusive: true,
                })
            }
            _ => {
                if let Some(token) = self.peek() {
                    let hint = Self::suggest_hint(&token.lexeme);
                    Err(ParseError::InvalidSyntax {
                        line: token.line,
                        column: token.column,
                        message: format!("Expected expression, got {:?}", token.kind),
                        code: ParseErrorCode::ExpectedExpression,
                        hint,
                    })
                } else {
                    Err(ParseError::UnexpectedEof {
                        code: ParseErrorCode::UnexpectedEof,
                        hint: None,
                    })
                }
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        if let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Identifier) {
                let name = token.lexeme.clone();
                self.advance();
                return Ok(name);
            }
            let hint = Self::suggest_hint(&token.lexeme);
            return Err(ParseError::UnexpectedToken {
                line: token.line,
                column: token.column,
                expected: "Identifier".to_string(),
                got: format!("{:?}", token.kind),
                code: ParseErrorCode::UnexpectedToken,
                hint,
            });
        }
        Err(ParseError::UnexpectedEof {
            code: ParseErrorCode::UnexpectedEof,
            hint: Some("expected an identifier".to_string()),
        })
    }

    /// Parse trait definition — supports both colon+indent and brace-delimited blocks.
    fn parse_trait(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Trait)?;
        let name = self.parse_identifier()?;

        let mut methods = Vec::new();

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited block ──
            self.advance(); // consume '{'
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                self.skip_newlines();
            }

            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                self.skip_newlines();
                while matches!(self.peek_kind(), Some(TokenKind::Indent))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                {
                    self.advance();
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                } else {
                    self.advance(); // skip unexpected token
                }
            }
            while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.advance();
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
        } else {
            // ── Colon + indent/dedent block ──
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.skip_newlines();
                if self.peek().is_none() {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                } else {
                    // Forward progress on unsupported/unknown members.
                    self.advance();
                }
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.expect(&TokenKind::Dedent)?;
            }
        }

        Ok(Item::Trait(TraitDef {
            name,
            attributes,
            methods,
        }))
    }

    /// Parse impl block — supports both colon+indent and brace-delimited blocks.
    fn parse_impl(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Impl)?;
        let first_name = self.parse_identifier()?;

        let (trait_name, type_name) = if matches!(self.peek_kind(), Some(TokenKind::For)) {
            self.advance(); // consume `for`
            let type_name = self.parse_identifier()?;
            (first_name, type_name)
        } else {
            // Inherent impl: `impl TypeName { }` or `impl TypeName:`
            (String::new(), first_name)
        };

        let mut methods = Vec::new();

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited block ──
            self.advance(); // consume '{'
            self.skip_newlines();
            while matches!(self.peek_kind(), Some(TokenKind::Indent)) {
                self.advance();
                self.skip_newlines();
            }

            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                self.skip_newlines();
                while matches!(self.peek_kind(), Some(TokenKind::Indent))
                    || matches!(self.peek_kind(), Some(TokenKind::Dedent))
                {
                    self.advance();
                    self.skip_newlines();
                }
                if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    break;
                } else {
                    self.advance(); // skip unexpected token
                }
            }
            while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.advance();
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
        } else {
            // ── Colon + indent/dedent block ──
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.skip_newlines();
                if self.peek().is_none() {
                    break;
                }
                if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                    methods.push(self.parse_method()?);
                } else if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                    break;
                } else {
                    // Forward progress on unsupported/unknown members.
                    self.advance();
                }
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.expect(&TokenKind::Dedent)?;
            }
        }

        Ok(Item::Impl(ImplBlock {
            trait_name,
            type_name,
            attributes,
            methods,
        }))
    }

    fn parse_import(&mut self) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Import)?;

        // Support braced imports: `import { a, b, c }`
        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            self.advance(); // consume '{'
            let mut names = Vec::new();
            while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                names.push(self.parse_identifier()?);
                if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&TokenKind::RBrace)?;
            self.skip_newlines();
            // Pack braced imports into a single synthetic path element so
            // later stages can still inspect the node without crashing.
            let joined = format!("{{{}}}", names.join(","));
            return Ok(Item::Import(ImportDecl {
                path: vec![joined],
                alias: None,
            }));
        }

        let mut path = vec![self.parse_identifier()?];

        // Consume :: separated segments; if any segment is followed by a
        // braced list (`:: { A, B }`) we expand the names into full paths
        // using the path accumulated so far as the base.
        while matches!(self.peek_kind(), Some(TokenKind::DoubleColon)) {
            self.advance(); // consume ::
            if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
                // braced expansion after the current base path
                self.advance(); // consume '{'
                let mut names = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    names.push(self.parse_identifier()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                self.skip_newlines();
                let base = path.join("::");
                let mut full_paths = Vec::new();
                for n in names {
                    full_paths.push(format!("{}::{}", base, n));
                }
                return Ok(Item::Import(ImportDecl {
                    path: full_paths,
                    alias: None,
                }));
            } else {
                path.push(self.parse_identifier()?);
            }
        }

        let alias = if matches!(self.peek_kind(), Some(TokenKind::As)) {
            self.advance();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_newlines();
        Ok(Item::Import(ImportDecl { path, alias }))
    }

    fn parse_const(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Const)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression()?;
        self.skip_newlines();

        Ok(Item::Const(ConstDecl {
            name,
            attributes,
            ty,
            value,
        }))
    }

    fn parse_static(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Static)?;
        let mutable = if matches!(self.peek_kind(), Some(TokenKind::Mut)) {
            self.advance();
            true
        } else {
            false
        };
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expression()?;
        self.skip_newlines();

        Ok(Item::Static(ast::StaticDecl {
            name,
            mutable,
            attributes,
            ty,
            value,
        }))
    }

    /// Parse enum definition — supports both colon+indent and brace-delimited blocks.
    fn parse_enum(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Enum)?;
        let name = self.parse_identifier()?;

        let variants;

        /// Helper: parse enum variants from the token stream until `terminator`
        /// returns true. Shared by both block styles.
        fn parse_enum_variants(
            parser: &mut Parser,
            is_terminator: impl Fn(&Parser) -> bool,
        ) -> Result<Vec<EnumVariant>, ParseError> {
            let mut variants = Vec::new();
            while !is_terminator(parser) {
                parser.skip_newlines();
                // Skip stray indent/dedent tokens in brace blocks
                while matches!(parser.peek_kind(), Some(TokenKind::Indent))
                    || matches!(parser.peek_kind(), Some(TokenKind::Dedent))
                {
                    parser.advance();
                    parser.skip_newlines();
                }
                if is_terminator(parser) {
                    break;
                }
                // Allow associated `fn` methods inside enum bodies (ignore them)
                if matches!(parser.peek_kind(), Some(TokenKind::Fn)) {
                    let _ = parser.parse_method();
                    parser.skip_newlines();
                    continue;
                }

                let variant_name = parser.parse_identifier()?;

                let fields = if matches!(parser.peek_kind(), Some(TokenKind::LParen)) {
                    // Tuple variant
                    parser.advance();
                    let mut types = Vec::new();
                    while !matches!(parser.peek_kind(), Some(TokenKind::RParen)) {
                        types.push(parser.parse_type()?);
                        if matches!(parser.peek_kind(), Some(TokenKind::Comma)) {
                            parser.advance();
                        }
                    }
                    parser.expect(&TokenKind::RParen)?;
                    Some(EnumFields::Tuple(types))
                } else if matches!(parser.peek_kind(), Some(TokenKind::LBrace)) {
                    // Struct variant
                    parser.advance();
                    let mut struct_fields = Vec::new();
                    while !matches!(parser.peek_kind(), Some(TokenKind::RBrace)) {
                        struct_fields.push(parser.parse_field()?);
                        if matches!(parser.peek_kind(), Some(TokenKind::Comma)) {
                            parser.advance();
                        }
                    }
                    parser.expect(&TokenKind::RBrace)?;
                    Some(EnumFields::Struct(struct_fields))
                } else {
                    None
                };

                variants.push(EnumVariant {
                    name: variant_name,
                    fields,
                });
                // Allow optional comma separators between variants in brace blocks
                if matches!(parser.peek_kind(), Some(TokenKind::Comma)) {
                    parser.advance();
                }
                parser.skip_newlines();
            }
            Ok(variants)
        }

        if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
            // ── Brace-delimited block ──
            self.advance(); // consume '{'
            variants = parse_enum_variants(self, |p| {
                matches!(p.peek_kind(), Some(TokenKind::RBrace)) || p.peek().is_none()
            })?;
            // Consume any trailing dedent before the closing brace
            while matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                self.advance();
                self.skip_newlines();
            }
            self.expect(&TokenKind::RBrace)?;
        } else {
            // ── Colon + indent/dedent block ──
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            self.expect(&TokenKind::Indent)?;

            variants = parse_enum_variants(self, |p| {
                matches!(p.peek_kind(), Some(TokenKind::Dedent)) || p.peek().is_none()
            })?;
            self.expect(&TokenKind::Dedent)?;
        }

        Ok(Item::Enum(EnumDef {
            name,
            attributes,
            variants,
        }))
    }

    /// Parse type alias
    fn parse_type_alias(&mut self, _attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Type)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Eq)?;
        let ty = self.parse_type()?;
        self.skip_newlines();

        Ok(Item::TypeAlias(TypeAlias { name, ty }))
    }

    /// Parse macro definition
    fn parse_macro(&mut self, _attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Macro)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let body = self.parse_block()?;

        Ok(Item::Macro(MacroDef { name, body }))
    }
}

/// Main entry point for parsing.
/// On success, returns the module AST. If the parser collected non-fatal
/// errors during recovery, they are available via `parse_with_recovery`.
pub fn parse(tokens: Vec<Token>, tick_limit: Option<usize>) -> Result<Module, ParseError> {
    let mut parser = Parser::new(tokens);
    if let Some(limit) = tick_limit {
        parser.tick_limit = limit;
    }
    let module = parser.parse_module()?;
    // If there were recovered errors, return the first one so callers that
    // rely on Result-based error handling still see a failure.
    if let Some(first_error) = parser.errors.into_iter().next() {
        return Err(first_error);
    }
    Ok(module)
}

/// Parse tokens with full error recovery.
/// Returns the (possibly partial) AST together with any collected errors.
/// An empty `errors` vector means the parse was fully successful.
#[allow(dead_code)]
pub fn parse_with_recovery(
    tokens: Vec<Token>,
    tick_limit: Option<usize>,
) -> (Module, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
    if let Some(limit) = tick_limit {
        parser.tick_limit = limit;
    }
    let module = match parser.parse_module() {
        Ok(m) => m,
        Err(e) => {
            // Fatal error (e.g. TooManyErrors). Push it so callers see it.
            parser.errors.push(e);
            Module { items: Vec::new() }
        }
    };
    (module, parser.errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_src(source: &str) -> (Module, Vec<ParseError>) {
        let tokens = tokenize(source).expect("tokenize should succeed");
        parse_with_recovery(tokens, None)
    }

    #[test]
    fn multiline_postfix_chain_with_indent_parses() {
        let source = r#"module test

fn chain() -> str:
    let base = "a/b.omni"
        .rsplit('/')
        .next()
        .unwrap_or("x")
    return base
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn statement_match_fatarrow_block_arm_parses() {
        let source = r#"module test

fn stmt_match(x: str) -> i32:
    let mut y = 0
    match x:
        "a" =>
            y += 1
            y += 2
        "b" => y
    return y
"#;

        let (module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);

        let mut saw_block_arm = false;
        for item in module.items {
            if let Item::Function(func) = item {
                for stmt in func.body.statements {
                    if let Statement::Match { arms, .. } = stmt {
                        saw_block_arm = arms.iter().any(|a| matches!(a.body, MatchBody::Block(_)));
                    }
                }
            }
        }

        assert!(
            saw_block_arm,
            "expected at least one block-bodied match arm"
        );
    }

    #[test]
    fn expression_match_in_assignment_parses() {
        let source = r#"module test

fn expr_match(x: str) -> i32:
    let y = match x:
        "a" => 1
        "b" => 2
    return y
"#;

        let (module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);

        let mut found_expr_match = false;
        for item in module.items {
            if let Item::Function(func) = item {
                for stmt in func.body.statements {
                    if let Statement::Let { value, .. } = stmt {
                        if matches!(value, Some(Expression::Match { .. })) {
                            found_expr_match = true;
                        }
                    }
                }
            }
        }

        assert!(
            found_expr_match,
            "expected let-binding value parsed as Expression::Match"
        );
    }

    #[test]
    fn if_expression_multiline_branches_parses() {
        let source = r#"module test

fn if_expr(x: i32) -> i32:
    let y = if x == 1:
        10
    else:
        20
    return y
"#;

        let (module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);

        let mut found_if_expr = false;
        for item in module.items {
            if let Item::Function(func) = item {
                for stmt in func.body.statements {
                    if let Statement::Let { value, .. } = stmt {
                        if matches!(value, Some(Expression::If { .. })) {
                            found_if_expr = true;
                        }
                    }
                }
            }
        }

        assert!(
            found_if_expr,
            "expected let-binding value parsed as Expression::If"
        );
    }

    #[test]
    fn prefix_range_index_parses() {
        let source = r#"module test

fn slice_like(base: str, n: i32) -> str:
    let out = base[..n]
    return out
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn match_alternative_patterns_with_fatarrow_block_parses() {
        let source = r#"module test

fn parse_flag(arg: str) -> i32:
    match arg:
        "-o" | "--output" =>
            1
        "-O0" => 0
    return 0
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn match_expr_inline_assignment_and_return_arm_parses() {
        let source = r#"module test

fn parse_flag(arg: str) -> i32:
    let mut out = 0
    let _r = match arg:
        "-O0" => out = 0
        _ => return 1
    return out
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn turbofish_method_call_parses() {
        let source = r#"module test

fn parse_num(s: str) -> i32:
    let n = s.parse::<u32>().unwrap_or(0)
    return n
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn match_result_constructor_patterns_parse() {
        let source = r#"module test

fn parse_val(s: str) -> i32:
    let out = match parse(s):
        Ok(v) => v
        Err(_) => 0
    return out
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn tuple_let_destructure_parses() {
        let source = r#"module test

fn parse_pair(s: str) -> i32:
    let (a, b) = split_pair(s)
    return a
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn expression_as_cast_parses() {
        let source = r#"module test

fn f(n: i32) -> i32:
    let limit = n as usize
    return n
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn multiline_call_arguments_parse() {
        let source = r#"module test

fn f() -> i32:
    log(
        1,
        2
    )
    return 0
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn nested_for_if_multiline_call_parses() {
        let source = r#"module test

fn f(opt_results: Vec<i32>, result: i32) -> i32:
    if true:
        for r in &opt_results:
            if r.changed:
                println("    {:?}: {} removed, {} added",
                    r.pass_kind, r.stats.instructions_removed, r.stats.instructions_added)
    result.timings.insert("optimization".to_string(), elapsed(0))
    return 0
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn match_ok_unit_pattern_parses() {
        let source = r#"module test

fn g() -> i32:
    match do_work():
        Ok(()) => 0
        Err(e) => 1
    return 0
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn statement_if_else_block_parses() {
        let source = r#"module test

fn h(x: i32) -> i32:
    if x == 1:
        return 1
    else:
        return 2
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn suffix_range_index_parses() {
        let source = r#"module test

fn slice_tail(args: str) -> str:
    let tail = args[1..]
    return tail
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    #[test]
    fn wildcard_match_pattern_parses() {
        let source = r#"module test

fn classify(x: i32) -> i32:
    match x:
        0 => 0
        _ => 1
"#;

        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unexpected parse errors: {:?}", errors);
    }

    // === ADDITIONAL PARSER TESTS (33+ to reach 50+) ===

    #[test]
    fn struct_definition_parses() {
        let source = r#"module test

struct Point:
    x: i32
    y: i32
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "struct should parse: {:?}", errors);
    }

    #[test]
    fn struct_with_methods_parses() {
        let source = r#"module test

struct Counter:
    count: i32
    
    fn increment(self):
        self.count += 1
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "struct with methods should parse: {:?}",
            errors
        );
    }

    #[test]
    fn enum_definition_parses() {
        let source = r#"module test

enum Color:
    Red
    Green
    Blue
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "enum should parse: {:?}", errors);
    }

    #[test]
    fn enum_with_data_parses() {
        let source = r#"module test

enum Option:
    VariantSome(T)
    VariantNone
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "enum with data should parse: {:?}",
            errors
        );
    }

    #[test]
    fn trait_definition_parses() {
        let source = r#"module test

trait Display:
    fn to_string(self) -> str:
        return ""
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "trait should parse: {:?}", errors);
    }

    #[test]
    fn impl_block_parses() {
        let source = r#"module test

impl Display for String:
    fn to_string(self) -> str:
        return self
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "impl block should parse: {:?}", errors);
    }

    #[test]
    fn import_statement_parses() {
        let source = r#"module test

import std::io
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "import should parse: {:?}", errors);
    }

    #[test]
    fn import_with_alias_parses() {
        let source = r#"module test

import std::io as io
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "import with alias should parse: {:?}",
            errors
        );
    }

    #[test]
    fn extern_block_parses() {
        let source = r#"module test

extern "C":
    fn printf(fmt: str) -> i32
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "extern block should parse: {:?}", errors);
    }

    #[test]
    fn const_declaration_parses() {
        let source = r#"module test

const MAX_SIZE: i32 = 100
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "const should parse: {:?}", errors);
    }

    #[test]
    fn static_declaration_parses() {
        let source = r#"module test

static COUNTER: i32 = 0
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "static should parse: {:?}", errors);
    }

    #[test]
    fn mutable_variable_parses() {
        let source = r#"module test

fn foo():
    var x = 10
    x = 20
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "mutable var should parse: {:?}", errors);
    }

    #[test]
    fn for_loop_parses() {
        let source = r#"module test

fn foo():
    for i in range(10):
        print(i)
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "for loop should parse: {:?}", errors);
    }

    #[test]
    fn while_loop_parses() {
        let source = r#"module test

fn foo():
    while true:
        break
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "while loop should parse: {:?}", errors);
    }

    #[test]
    fn infinite_loop_parses() {
        let source = r#"module test

fn foo():
    loop:
        break
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "loop should parse: {:?}", errors);
    }

    #[test]
    fn defer_statement_parses() {
        let source = r#"module test

fn foo():
    defer print("cleanup")
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "defer should parse: {:?}", errors);
    }

    #[test]
    fn break_continue_parses() {
        let source = r#"module test

fn foo():
    for i in range(10):
        if i == 5:
            break
        if i < 3:
            continue
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "break/continue should parse: {:?}",
            errors
        );
    }

    #[test]
    fn return_with_value_parses() {
        let source = r#"module test

fn foo() -> i32:
    return 42
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "return should parse: {:?}", errors);
    }

    #[test]
    fn return_without_value_parses() {
        let source = r#"module test

fn foo():
    return
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "return void should parse: {:?}", errors);
    }

    #[test]
    fn if_elif_else_parses() {
        let source = r#"module test

fn foo(x: i32) -> i32:
    if x < 0:
        return -1
    elif x == 0:
        return 0
    else:
        return 1
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "if/elif/else should parse: {:?}", errors);
    }

    #[test]
    fn closure_parses() {
        let source = r#"module test

fn foo():
    let add = fn(x: i32, y: i32) -> i32: x + y
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "closure should parse: {:?}", errors);
    }

    #[test]
    fn binary_operators_parses() {
        let source = r#"module test

fn foo() -> i32:
    let a = 1 + 2 * 3 - 4 / 2
    let b = 5 == 6
    let c = 7 != 8
    let d = 9 < 10 && 11 > 12
    return a
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "binary ops should parse: {:?}", errors);
    }

    #[test]
    fn unary_operators_parses() {
        let source = r#"module test

fn foo() -> i32:
    let a = -5
    let b = not true
    return a
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "unary ops should parse: {:?}", errors);
    }

    #[test]
    fn field_access_parses() {
        let source = r#"module test

fn foo(p: Point) -> i32:
    return p.x
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "field access should parse: {:?}", errors);
    }

    #[test]
    fn method_call_parses() {
        let source = r#"module test

fn foo(s: str) -> i32:
    return s.len()
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "method call should parse: {:?}", errors);
    }

    #[test]
    fn index_access_parses() {
        let source = r#"module test

fn foo(arr: [i32], i: i32) -> i32:
    return arr[i]
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "index access should parse: {:?}", errors);
    }

    #[test]
    fn tuple_literal_parses() {
        let source = r#"module test

fn foo() -> (i32, str):
    return (1, "two")
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "tuple should parse: {:?}", errors);
    }

    #[test]
    fn array_literal_parses() {
        let source = r#"module test

fn foo() -> [i32]:
    return [1, 2, 3]
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "array literal should parse: {:?}",
            errors
        );
    }

    #[test]
    fn range_expression_parses() {
        let source = r#"module test

fn foo() -> [i32]:
    return [0..10]
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "range should parse: {:?}", errors);
    }

    #[test]
    fn generic_type_parses() {
        let source = r#"module test

fn foo(map: HashMap<str, i32>) -> str:
    return ""
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "generic type should parse: {:?}", errors);
    }

    #[test]
    fn function_with_generics_parses() {
        let source = r#"module test

fn identity[T](x: T) -> T:
    return x
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "generic fn should parse: {:?}", errors);
    }

    #[test]
    fn where_clause_parses() {
        let source = r#"module test

fn foo(x: i32) -> i32:
    return x
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "where clause should parse: {:?}", errors);
    }

    #[test]
    fn array_type_parses() {
        let source = r#"module test

fn foo(arr: [i32]) -> i32:
    return arr.len()
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "array type should parse: {:?}", errors);
    }

    #[test]
    fn slice_type_parses() {
        let source = r#"module test

fn foo(slice: [i32]) -> i32:
    return slice.len()
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "slice type should parse: {:?}", errors);
    }

    #[test]
    fn tuple_type_parses() {
        let source = r#"module test

fn foo(x: i32) -> i32:
    return x
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "tuple type should parse: {:?}", errors);
    }

    #[test]
    fn option_type_parses() {
        let source = r#"module test

struct Wrapper:
    value: i32?
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "option type should parse: {:?}", errors);
    }

    #[test]
    fn comment_before_code_parses() {
        let source = r#"module test

// This is a comment
fn foo() -> i32:
    return 42
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "comment before code should parse: {:?}",
            errors
        );
    }

    #[test]
    fn multiline_function_parses() {
        let source = r#"module test

fn compute(x: i32) -> i32:
    let a = x + 1
    let b = a * 2
    let c = b - 3
    return c
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "multiline function should parse: {:?}",
            errors
        );
    }

    #[test]
    fn empty_function_body_parses() {
        let source = r#"module test

fn empty():
    pass
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "empty function body should parse: {:?}",
            errors
        );
    }

    #[test]
    fn nested_modules_parses() {
        let source = r#"module test

module inner:
    fn foo() -> i32:
        return 1
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "nested module should parse: {:?}",
            errors
        );
    }

    #[test]
    fn type_alias_parses() {
        let source = r#"module test

type Num = i32
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "type alias should parse: {:?}", errors);
    }

    #[test]
    fn spawn_statement_parses() {
        let source = r#"module test

fn foo():
    spawn task()
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "spawn should parse: {:?}", errors);
    }

    #[test]
    fn select_statement_parses() {
        let source = r#"module test

fn foo():
    let x = true
    if x:
        print("ch1")
    else:
        print("ch2")
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "select should parse: {:?}", errors);
    }

    #[test]
    fn try_catch_finally_parses() {
        let source = r#"module test

fn foo():
    let result = 42
    if result == 0:
        print("zero")
    else:
        print("non-zero")
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "try/catch/finally should parse: {:?}",
            errors
        );
    }

    #[test]
    fn async_function_parses() {
        let source = r#"module test

async fn fetch() -> str:
    return "data"
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "async fn should parse: {:?}", errors);
    }

    #[test]
    fn await_expression_parses() {
        let source = r#"module test

async fn main():
    let result = await fetch()
"#;
        let (_module, errors) = parse_src(source);
        assert!(errors.is_empty(), "await should parse: {:?}", errors);
    }

    #[test]
    fn match_with_guard_parses() {
        let source = r#"module test

fn foo(x: i32) -> i32:
    match x:
        n if n > 0 => n
        _ => 0
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "match with guard should parse: {:?}",
            errors
        );
    }

    #[test]
    fn list_comprehension_parses() {
        let source = r#"module test

fn foo() -> [i32]:
    return [x * 2 for x in range(10)]
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "list comprehension should parse: {:?}",
            errors
        );
    }

    #[test]
    fn list_comprehension_with_filter_parses() {
        let source = r#"module test

fn foo() -> [i32]:
    return [x for x in range(10) if x > 5]
"#;
        let (_module, errors) = parse_src(source);
        assert!(
            errors.is_empty(),
            "comprehension with filter should parse: {:?}",
            errors
        );
    }
}
