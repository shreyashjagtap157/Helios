// Lexer for Omni Language
// Tokenizes source code into a stream of tokens

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Let,
    Mut,
    Fn,
    Async,
    If,
    Else,
    While,
    For,
    In,
    Return,
    True,
    False,
    Struct,
    Enum,
    Impl,
    Trait,
    Use,
    Pub,
    Unsafe,
    Await,
    
    // Identifiers and literals
    Identifier(String),
    Number(String),
    String(String),
    
    // Operators & Punctuation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Ampersand,
    Colon,
    DoubleColon,
    Semicolon,
    Comma,
    Dot,
    Arrow,
    FatArrow,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.current_char();

        let token_type = match ch {
            '+' => { self.advance(); TokenType::Plus }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => { self.advance(); TokenType::Star }
            '/' => { self.advance(); TokenType::Slash }
            '%' => { self.advance(); TokenType::Percent }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::EqualEqual
                } else if self.current_char() == '>' {
                    self.advance();
                    TokenType::FatArrow
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Not
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    TokenType::And
                } else {
                    TokenType::Ampersand
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::Or
                } else {
                    return Err(format!("Unexpected character '|' at {}:{}", start_line, start_column));
                }
            }
            ':' => {
                self.advance();
                if self.current_char() == ':' {
                    self.advance();
                    TokenType::DoubleColon
                } else {
                    TokenType::Colon
                }
            }
            ';' => { self.advance(); TokenType::Semicolon }
            ',' => { self.advance(); TokenType::Comma }
            '.' => { self.advance(); TokenType::Dot }
            '(' => { self.advance(); TokenType::LeftParen }
            ')' => { self.advance(); TokenType::RightParen }
            '{' => { self.advance(); TokenType::LeftBrace }
            '}' => { self.advance(); TokenType::RightBrace }
            '[' => { self.advance(); TokenType::LeftBracket }
            ']' => { self.advance(); TokenType::RightBracket }
            '"' => self.read_string()?,
            '\n' => {
                self.advance();
                self.line += 1;
                self.column = 1;
                TokenType::Newline
            }
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ => return Err(format!("Unexpected character '{}' at {}:{}", ch, start_line, start_column)),
        };

        Ok(Token {
            token_type,
            lexeme: String::new(),
            line: start_line,
            column: start_column,
        })
    }

    fn read_identifier(&mut self) -> TokenType {
        let mut ident = String::new();
        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            ident.push(self.current_char());
            self.advance();
        }

        match ident.as_str() {
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "fn" => TokenType::Fn,
            "async" => TokenType::Async,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "impl" => TokenType::Impl,
            "trait" => TokenType::Trait,
            "use" => TokenType::Use,
            "pub" => TokenType::Pub,
            "unsafe" => TokenType::Unsafe,
            "await" => TokenType::Await,
            _ => TokenType::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> TokenType {
        let mut num = String::new();
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num.push(self.current_char());
            self.advance();
        }
        if !self.is_at_end() && self.current_char() == '.' {
            num.push(self.current_char());
            self.advance();
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                num.push(self.current_char());
                self.advance();
            }
        }
        TokenType::Number(num)
    }

    fn read_string(&mut self) -> Result<TokenType, String> {
        self.advance(); // consume opening quote
        let mut string = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                let ch = match self.current_char() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => self.current_char(),
                };
                string.push(ch);
                self.advance();
            } else {
                string.push(self.current_char());
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.advance(); // consume closing quote
        Ok(TokenType::String(string))
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' => self.advance(),
                '/' if self.peek() == '/' => {
                    // Line comment
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                '/' if self.peek() == '*' => {
                    // Block comment
                    self.advance();
                    self.advance();
                    while !self.is_at_end() {
                        if self.current_char() == '*' && self.peek() == '/' {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() { '\0' } else { self.input[self.position] }
    }

    fn peek(&self) -> char {
        if self.position + 1 >= self.input.len() { '\0' } else { self.input[self.position + 1] }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
