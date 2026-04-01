// Parser for Omni Language
// Converts token stream into an Abstract Syntax Tree

use crate::lexer::{Token, TokenType};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            let item = self.parse_item()?;
            items.push(item);
            self.skip_newlines();
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match &self.current().token_type {
            TokenType::Identifier(name) if name == "module" => {
                self.parse_module_declaration()?;
                self.parse_item()
            }
            TokenType::Async => self.parse_function(),
            TokenType::Fn => self.parse_function(),
            TokenType::Struct => self.parse_struct(),
            TokenType::Let | TokenType::Pub => self.parse_variable(),
            _ => Err(format!(
                "Expected function, struct, or variable at {}",
                self.current().line
            )),
        }
    }

    fn parse_function(&mut self) -> Result<Item, String> {
        // Accept optional async prefix for phase-3 convergence.
        self.match_token(&TokenType::Async);
        self.consume(TokenType::Fn)?;
        let name = self.expect_identifier()?;
        self.skip_optional_generic_params()?;
        self.consume(TokenType::LeftParen)?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = self.expect_identifier()?;
                self.consume(TokenType::Colon)?;
                let param_type = self.parse_type()?;
                params.push((param_name, param_type));

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen)?;

        let return_type = if self.match_token(&TokenType::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = if self.match_token(&TokenType::LeftBrace) {
            let block = self.parse_block()?;
            self.consume(TokenType::RightBrace)?;
            block
        } else if self.match_token(&TokenType::Colon) {
            self.parse_colon_block()?
        } else {
            return Err(format!(
                "Expected '{{' or ':' after function signature at line {}",
                self.current().line
            ));
        };

        Ok(Item::FunctionDef(FunctionDef {
            name,
            params,
            return_type,
            body,
        }))
    }

    fn parse_struct(&mut self) -> Result<Item, String> {
        self.consume(TokenType::Struct)?;
        let name = self.expect_identifier()?;
        self.consume(TokenType::LeftBrace)?;

        let mut fields = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            let field_name = self.expect_identifier()?;
            self.consume(TokenType::Colon)?;
            let field_type = self.parse_type()?;
            fields.push((field_name, field_type));

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }
        self.consume(TokenType::RightBrace)?;

        Ok(Item::StructDef(StructDef { name, fields }))
    }

    fn parse_variable(&mut self) -> Result<Item, String> {
        if self.match_token(&TokenType::Pub) {
            self.consume(TokenType::Let)?;
        } else {
            self.consume(TokenType::Let)?;
        }

        let is_mutable = self.match_token(&TokenType::Mut);
        let name = self.expect_identifier()?;

        let var_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenType::Equal)?;
        let value = self.parse_expression()?;
        self.consume(TokenType::Semicolon)?;

        Ok(Item::Variable(VariableDecl {
            name,
            var_type,
            is_mutable,
            value,
        }))
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenType::RightBrace) {
                break;
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }

        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.current().token_type {
            TokenType::Let => {
                self.consume(TokenType::Let)?;
                let is_mutable = self.match_token(&TokenType::Mut);
                let name = self.expect_identifier()?;
                let var_type = if self.match_token(&TokenType::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.consume(TokenType::Equal)?;
                let value = self.parse_expression()?;
                self.consume_stmt_terminator()?;
                Ok(Statement::Let(name, var_type, is_mutable, value))
            }
            TokenType::If => {
                self.consume(TokenType::If)?;
                let condition = self.parse_expression()?;
                self.consume(TokenType::LeftBrace)?;
                let if_block = self.parse_block()?;
                self.consume(TokenType::RightBrace)?;
                let else_block = if self.match_token(&TokenType::Else) {
                    self.consume(TokenType::LeftBrace)?;
                    let block = self.parse_block()?;
                    self.consume(TokenType::RightBrace)?;
                    Some(block)
                } else {
                    None
                };
                Ok(Statement::If(condition, if_block, else_block))
            }
            TokenType::While => {
                self.consume(TokenType::While)?;
                let condition = self.parse_expression()?;
                self.consume(TokenType::LeftBrace)?;
                let body = self.parse_block()?;
                self.consume(TokenType::RightBrace)?;
                Ok(Statement::While(condition, body))
            }
            TokenType::For => {
                self.consume(TokenType::For)?;
                let var = self.expect_identifier()?;
                self.consume(TokenType::In)?;
                let iter = self.parse_expression()?;
                self.consume(TokenType::LeftBrace)?;
                let body = self.parse_block()?;
                self.consume(TokenType::RightBrace)?;
                Ok(Statement::For(var, iter, body))
            }
            TokenType::Return => {
                self.consume(TokenType::Return)?;
                let expr = if self.check(&TokenType::Semicolon) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                self.consume_stmt_terminator()?;
                Ok(Statement::Return(expr))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.consume_stmt_terminator()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_colon_block(&mut self) -> Result<Block, String> {
        self.skip_newlines();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if matches!(
                self.current().token_type,
                TokenType::Fn | TokenType::Async | TokenType::Struct
            ) {
                break;
            }
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }
            if matches!(
                self.current().token_type,
                TokenType::Fn | TokenType::Async | TokenType::Struct
            ) {
                break;
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }

        Ok(Block { statements })
    }

    fn parse_module_declaration(&mut self) -> Result<(), String> {
        self.expect_identifier()?; // module
        while !self.is_at_end() && !self.check(&TokenType::Newline) {
            self.advance();
        }
        self.skip_newlines();
        Ok(())
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;

        while self.match_token(&TokenType::Or) {
            let right = self.parse_and()?;
            left = Expression::BinaryOp(Box::new(left), BinaryOp::Or, Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;

        while self.match_token(&TokenType::And) {
            let right = self.parse_equality()?;
            left = Expression::BinaryOp(Box::new(left), BinaryOp::And, Box::new(right));
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        loop {
            if self.match_token(&TokenType::EqualEqual) {
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Equal, Box::new(right));
            } else if self.match_token(&TokenType::NotEqual) {
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::NotEqual, Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_additive()?;

        loop {
            if self.match_token(&TokenType::Less) {
                let right = self.parse_additive()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Less, Box::new(right));
            } else if self.match_token(&TokenType::LessEqual) {
                let right = self.parse_additive()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::LessEqual, Box::new(right));
            } else if self.match_token(&TokenType::Greater) {
                let right = self.parse_additive()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Greater, Box::new(right));
            } else if self.match_token(&TokenType::GreaterEqual) {
                let right = self.parse_additive()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::GreaterEqual, Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            if self.match_token(&TokenType::Plus) {
                let right = self.parse_multiplicative()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Add, Box::new(right));
            } else if self.match_token(&TokenType::Minus) {
                let right = self.parse_multiplicative()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Subtract, Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;

        loop {
            if self.match_token(&TokenType::Star) {
                let right = self.parse_unary()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Multiply, Box::new(right));
            } else if self.match_token(&TokenType::Slash) {
                let right = self.parse_unary()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Divide, Box::new(right));
            } else if self.match_token(&TokenType::Percent) {
                let right = self.parse_unary()?;
                left = Expression::BinaryOp(Box::new(left), BinaryOp::Modulo, Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_token(&TokenType::Minus) {
            let expr = self.parse_unary()?;
            Ok(Expression::UnaryOp(UnaryOp::Negate, Box::new(expr)))
        } else if self.match_token(&TokenType::Not) {
            let expr = self.parse_unary()?;
            Ok(Expression::UnaryOp(UnaryOp::Not, Box::new(expr)))
        } else if self.match_token(&TokenType::Await) {
            // Stage1 parser support: treat `await expr` as expression passthrough
            // until dedicated await AST node is introduced in this lightweight path.
            self.parse_unary()
        } else {
            self.parse_primary()
        }
    }

    fn skip_optional_generic_params(&mut self) -> Result<(), String> {
        if !self.match_token(&TokenType::LeftBracket) {
            return Ok(());
        }

        let mut depth = 1usize;
        while depth > 0 {
            if self.is_at_end() {
                return Err("Unterminated generic parameter list in function declaration".to_string());
            }

            if self.match_token(&TokenType::LeftBracket) {
                depth += 1;
                continue;
            }
            if self.match_token(&TokenType::RightBracket) {
                depth -= 1;
                continue;
            }

            self.advance();
        }

        Ok(())
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match &self.current().token_type {
            TokenType::Number(num_str) => {
                let num: f64 = num_str.parse().map_err(|_| "Invalid number".to_string())?;
                self.advance();
                Ok(Expression::Literal(Literal::Number(num)))
            }
            TokenType::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.match_token(&TokenType::LeftParen) {
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenType::RightParen)?;
                    Ok(Expression::Call(name, args))
                } else if self.match_token(&TokenType::Equal) {
                    let value = self.parse_expression()?;
                    Ok(Expression::Assignment(name, Box::new(value)))
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen)?;
                Ok(expr)
            }
            _ => Err(format!(
                "Unexpected token in expression: {:?} at line {}",
                self.current().token_type,
                self.current().line
            )),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let type_str = self.expect_identifier()?;
        Ok(match type_str.as_str() {
            "i32" => Type::I32,
            "i64" => Type::I64,
            "f64" => Type::F64,
            "bool" => Type::Bool,
            "string" => Type::String,
            "void" => Type::Void,
            name => Type::Custom(name.to_string()),
        })
    }

    // Helper methods
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().token_type, TokenType::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.current().token_type)
            == std::mem::discriminant(token_type)
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: TokenType) -> Result<(), String> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, got {:?} at line {}",
                token_type, self.current().token_type, self.current().line
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        match &self.current().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(format!(
                "Expected identifier, got {:?} at line {}",
                self.current().token_type,
                self.current().line
            )),
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_token(&TokenType::Newline) {}
    }

    fn consume_stmt_terminator(&mut self) -> Result<(), String> {
        if self.match_token(&TokenType::Semicolon) || self.match_token(&TokenType::Newline) {
            return Ok(());
        }
        if self.check(&TokenType::RightBrace) || self.check(&TokenType::Eof) {
            return Ok(());
        }
        Err(format!(
            "Expected statement terminator at line {}",
            self.current().line
        ))
    }
}
