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
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            error_limit: 50,
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
                let hint = Self::suggest_hint(&token.lexeme);
                Err(ParseError::UnexpectedToken {
                    line: token.line,
                    column: token.column,
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
            Some(TokenKind::Trait) => self.parse_trait(attributes),
            Some(TokenKind::Impl) => self.parse_impl(attributes),
            Some(TokenKind::Import) => self.parse_import(),
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

    /// Parse module declaration
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

        // Check for colon (block module) vs newline (file-scope declaration)
        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
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

    /// Parse struct definition
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

        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                methods.push(self.parse_method()?);
            } else if matches!(self.peek_kind(), Some(TokenKind::Identifier)) {
                fields.push(self.parse_field()?);
            } else if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                break;
            }
        }
        self.expect(&TokenKind::Dedent)?;

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

    /// Parse extern block (e.g. extern "C++")
    fn parse_extern(&mut self, _attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Extern)?;

        let abi = if matches!(self.peek_kind(), Some(TokenKind::StringLiteral)) {
            let s = self.advance().unwrap().lexeme.clone();
            s[1..s.len() - 1].to_string() // Remove quotes
        } else {
            "C".to_string() // Default to C
        };

        self.expect(&TokenKind::LBrace)?;

        let mut functions = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                // Parse function signature only (no body) for extern declarations.
                // Uses parse_fn_signature which handles semicolon-terminated signatures.
                functions.push(self.parse_fn_signature()?);
            } else if matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                break;
            } else {
                // Skip unexpected tokens to avoid infinite loop
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(Item::Extern(ExternBlock { abi, functions }))
    }

    /// Parse comptime block
    fn parse_comptime(&mut self) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Comptime)?;
        let body = self.parse_block()?;
        Ok(Item::Comptime(body))
    }

    /// Parse function signature (fn name(args) -> Ret;)
    fn parse_fn_signature(&mut self) -> Result<Function, ParseError> {
        let is_async = if matches!(self.peek_kind(), Some(TokenKind::Async)) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&TokenKind::Fn)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let return_type = if matches!(self.peek_kind(), Some(TokenKind::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            // It's a definition, not a signature? But externs shouldn't have bodies.
            // We allow it to error out or we parse body.
            // For extern, we expect NO body.
        }

        // Extern functions usually end with newline or semicolon/brace depending on syntax
        // Omni syntax: `fn foo():` <- block start.
        // For extern, maybe `fn foo()` simply?
        // Let's assume externs are just signatures inside the brace block.

        Ok(Function {
            name,
            is_async,
            attributes: Vec::new(),
            params,
            return_type,
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

        // Skip where clauses if present
        if matches!(self.peek_kind(), Some(TokenKind::Where)) {
            self.advance(); // consume `where`
                            // Consume tokens until we hit a colon (block start)
            while !matches!(self.peek_kind(), Some(TokenKind::Colon))
                && !matches!(self.peek_kind(), Some(TokenKind::Newline))
                && self.peek().is_some()
            {
                self.advance();
            }
        }

        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;

        // Function body
        let body = self.parse_block()?;

        Ok(Function {
            name,
            is_async,
            attributes,
            params,
            return_type,
            body,
        })
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
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type()?;
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
                    while !matches!(self.peek_kind(), Some(TokenKind::Gt)) {
                        type_args.push(self.parse_type()?);
                        if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                            self.advance();
                        }
                    }
                    self.expect(&TokenKind::Gt)?;
                    Ok(Type::Generic(name, type_args))
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

    /// Parse a block of statements.
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
        let name = self.parse_identifier()?;

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
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let body = self.parse_block()?;
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

        let name = self.parse_identifier()?;

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
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let then_block = self.parse_block()?;

        // O-004: Handle elif chains by desugaring to nested else { if ... }
        let else_block = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
            self.advance(); // consume 'elif'
            let elif_condition = self.parse_expression()?;
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            let elif_then = self.parse_block()?;

            // Recursively handle further elif/else chains
            let elif_else = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
                // Re-enter elif handling: synthesize an if-statement for the next elif
                // We need to "put back" the elif and call ourselves, but since we can't
                // unadvance, we build the chain manually
                let mut inner = self.parse_elif_chain()?;
                Some(Block { statements: vec![inner] })
            } else if matches!(self.peek_kind(), Some(TokenKind::Else)) {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                self.expect(&TokenKind::Newline)?;
                Some(self.parse_block()?)
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
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            Some(self.parse_block()?)
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
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let then_block = self.parse_block()?;

        let else_block = if matches!(self.peek_kind(), Some(TokenKind::Elif)) {
            let inner = self.parse_elif_chain()?;
            Some(Block { statements: vec![inner] })
        } else if matches!(self.peek_kind(), Some(TokenKind::Else)) {
            self.advance();
            self.expect(&TokenKind::Colon)?;
            self.expect(&TokenKind::Newline)?;
            Some(self.parse_block()?)
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
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let body = self.parse_block()?;

        Ok(Statement::For { var, iter, body })
    }

    /// Parse while loop
    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        let body = self.parse_block()?;

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

            // Accept either `:` (block arms) or `=>` / `FatArrow` (expr arms)
            match self.peek_kind() {
                Some(TokenKind::FatArrow) => {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.skip_newlines();
                    arms.push(MatchArm {
                        pattern,
                        body: MatchBody::Expr(expr),
                    });
                }
                Some(TokenKind::Colon) => {
                    self.advance();
                    // Single line or block
                    if matches!(self.peek_kind(), Some(TokenKind::Newline)) {
                        self.advance();
                        let body = self.parse_block()?;
                        arms.push(MatchArm {
                            pattern,
                            body: MatchBody::Block(body),
                        });
                    } else {
                        let expr = self.parse_expression()?;
                        self.skip_newlines();
                        arms.push(MatchArm {
                            pattern,
                            body: MatchBody::Expr(expr),
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

    /// Parse a pattern
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::LParen) => {
                // Parenthesized pattern: (pat)
                self.advance();
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

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.peek_binary_op() {
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            self.advance();
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
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
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
                        let args = self.parse_call_args()?;
                        self.expect(&TokenKind::RParen)?;
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
                    let args = self.parse_call_args()?;
                    self.expect(&TokenKind::RParen)?;
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
                    let member = self.parse_identifier()?;
                    expr = Expression::Path(Box::new(expr), member);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
            args.push(self.parse_expression()?);
            if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                self.advance();
            }
        }
        Ok(args)
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

    fn parse_trait(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Trait)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut methods = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                methods.push(self.parse_method()?);
            }
        }
        self.expect(&TokenKind::Dedent)?;

        Ok(Item::Trait(TraitDef {
            name,
            attributes,
            methods,
        }))
    }

    fn parse_impl(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Impl)?;
        let first_name = self.parse_identifier()?;

        let (trait_name, type_name) = if matches!(self.peek_kind(), Some(TokenKind::For)) {
            self.advance(); // consume `for`
            let type_name = self.parse_identifier()?;
            (first_name, type_name)
        } else {
            // Inherent impl: `impl TypeName { }`
            (String::new(), first_name)
        };

        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut methods = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                methods.push(self.parse_method()?);
            }
        }
        self.expect(&TokenKind::Dedent)?;

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

    /// Parse enum definition
    fn parse_enum(&mut self, attributes: Vec<String>) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Enum)?;
        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut variants = Vec::new();
        while !matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
            self.skip_newlines();
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) {
                break;
            }
            // Allow associated `fn` methods inside enum bodies (ignore them)
            if matches!(self.peek_kind(), Some(TokenKind::Fn)) {
                // Parse and discard method to avoid breaking the enum body parse
                let _ = self.parse_method();
                self.skip_newlines();
                continue;
            }

            let variant_name = self.parse_identifier()?;

            let fields = if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                // Tuple variant
                self.advance();
                let mut types = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RParen)) {
                    types.push(self.parse_type()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RParen)?;
                Some(EnumFields::Tuple(types))
            } else if matches!(self.peek_kind(), Some(TokenKind::LBrace)) {
                // Struct variant
                self.advance();
                let mut struct_fields = Vec::new();
                while !matches!(self.peek_kind(), Some(TokenKind::RBrace)) {
                    struct_fields.push(self.parse_field()?);
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    }
                }
                self.expect(&TokenKind::RBrace)?;
                Some(EnumFields::Struct(struct_fields))
            } else {
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });
            self.skip_newlines();
        }
        self.expect(&TokenKind::Dedent)?;

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
pub fn parse(tokens: Vec<Token>) -> Result<Module, ParseError> {
    let mut parser = Parser::new(tokens);
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
pub fn parse_with_recovery(tokens: Vec<Token>) -> (Module, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
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
