use std::collections::HashMap;

use crate::token::{Token, TokenType};

pub struct Lexer {
    src: Vec<char>,
    tokens: Vec<Token>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer {
            src: src.chars().collect(),
            tokens: Vec::new(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, String> {
        while self.pos < self.src.len() {
            let c = self.peek();

            if c == '\n' {
                self.add_token(TokenType::Newline, "\\n");
                self.advance();
                self.line += 1;
                self.col = 1;
                continue;
            }
            if c.is_whitespace() {
                self.advance();
                continue;
            }
            if c == '/' && self.peek_ahead(1) == '/' {
                self.skip_line();
                continue;
            }
            if c == '/' && self.peek_ahead(1) == '*' {
                self.skip_block()?;
                continue;
            }

            if c == '"' {
                self.string()?;
                continue;
            }
            if c.is_ascii_digit() {
                self.number();
                continue;
            }
            if c.is_alphabetic() || c == '_' {
                self.ident_or_keyword();
                continue;
            }

            self.scan_operator(c)?;
        }

        self.tokens.push(Token::eof(self.line, self.col));
        Ok(std::mem::take(&mut self.tokens))
    }

    // ── helpers ──

    fn peek(&self) -> char {
        self.src.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek_ahead(&self, ahead: usize) -> char {
        self.src.get(self.pos + ahead).copied().unwrap_or('\0')
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    fn advance_n(&mut self, n: usize) {
        self.pos += n;
        self.col += n;
    }

    fn add_token(&mut self, ty: TokenType, lexeme: &str) {
        self.tokens.push(Token {
            ty,
            lexeme: lexeme.to_string(),
            line: self.line,
            col: self.col,
        });
    }

    fn add_token_char(&mut self, ty: TokenType) {
        let c = self.peek();
        self.add_token(ty, &c.to_string());
    }

    fn skip_line(&mut self) {
        while self.pos < self.src.len() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_block(&mut self) -> Result<(), String> {
        self.advance_n(2);
        while self.pos < self.src.len() && !(self.peek() == '*' && self.peek_ahead(1) == '/') {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 0;
            }
            self.advance();
        }
        if self.pos < self.src.len() {
            self.advance_n(2);
        }
        Ok(())
    }

    fn string(&mut self) -> Result<(), String> {
        let start_col = self.col;
        self.advance(); // skip opening "
        let mut s = String::new();
        while self.pos < self.src.len() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance();
            }
            if self.peek() == '\n' {
                self.line += 1;
            }
            s.push(self.peek());
            self.advance();
        }
        if self.pos >= self.src.len() {
            return Err(self.error("unterminated string"));
        }
        self.advance(); // skip closing "
        self.tokens.push(Token {
            ty: TokenType::StringLiteral,
            lexeme: s,
            line: self.line,
            col: start_col,
        });
        Ok(())
    }

    fn number(&mut self) {
        let start_col = self.col;
        let mut s = String::new();
        let mut is_float = false;
        while self.pos < self.src.len()
            && (self.peek().is_ascii_digit() || self.peek() == '.')
        {
            if self.peek() == '.' {
                if is_float {
                    break;
                }
                is_float = true;
            }
            s.push(self.peek());
            self.advance();
        }
        let ty = if is_float {
            TokenType::FloatLiteral
        } else {
            TokenType::IntLiteral
        };
        self.tokens.push(Token {
            ty,
            lexeme: s,
            line: self.line,
            col: start_col,
        });
    }

    fn ident_or_keyword(&mut self) {
        let start_col = self.col;
        let mut s = String::new();
        while self.pos < self.src.len()
            && (self.peek().is_alphanumeric() || self.peek() == '_')
        {
            s.push(self.peek());
            self.advance();
        }
        let ty = Self::keyword_map().get(s.as_str()).copied().unwrap_or(TokenType::Ident);
        self.tokens.push(Token {
            ty,
            lexeme: s,
            line: self.line,
            col: start_col,
        });
    }

    fn scan_operator(&mut self, c: char) -> Result<(), String> {
        match c {
            '+' => {
                self.add_token_char(TokenType::Plus);
                self.advance();
            }
            '-' => {
                self.add_token_char(TokenType::Minus);
                self.advance();
            }
            '*' => {
                self.add_token_char(TokenType::Star);
                self.advance();
            }
            '/' => {
                self.add_token_char(TokenType::Slash);
                self.advance();
            }
            '%' => {
                self.add_token_char(TokenType::Percent);
                self.advance();
            }
            '(' => {
                self.add_token_char(TokenType::LParen);
                self.advance();
            }
            ')' => {
                self.add_token_char(TokenType::RParen);
                self.advance();
            }
            '{' => {
                self.add_token_char(TokenType::LBrace);
                self.advance();
            }
            '}' => {
                self.add_token_char(TokenType::RBrace);
                self.advance();
            }
            ',' => {
                self.add_token_char(TokenType::Comma);
                self.advance();
            }
            ';' => {
                self.add_token_char(TokenType::Semi);
                self.advance();
            }
            '.' => {
                if self.peek_ahead(1).is_ascii_digit() {
                    self.number();
                } else {
                    self.add_token_char(TokenType::Dot);
                    self.advance();
                }
            }
            ':' => {
                if self.peek_ahead(1) == '=' {
                    self.add_token(TokenType::ColonDefine, ":=");
                    self.advance_n(2);
                } else {
                    return Err(self.error("expected ':='"));
                }
            }
            '=' => {
                if self.peek_ahead(1) == '=' {
                    self.add_token(TokenType::Eq, "==");
                    self.advance_n(2);
                } else {
                    self.add_token_char(TokenType::Assign);
                    self.advance();
                }
            }
            '!' => {
                if self.peek_ahead(1) == '=' {
                    self.add_token(TokenType::Neq, "!=");
                    self.advance_n(2);
                } else {
                    return Err(self.error("expected '!='"));
                }
            }
            '<' => {
                if self.peek_ahead(1) == '=' {
                    self.add_token(TokenType::Le, "<=");
                    self.advance_n(2);
                } else {
                    self.add_token_char(TokenType::Lt);
                    self.advance();
                }
            }
            '>' => {
                if self.peek_ahead(1) == '=' {
                    self.add_token(TokenType::Ge, ">=");
                    self.advance_n(2);
                } else {
                    self.add_token_char(TokenType::Gt);
                    self.advance();
                }
            }
            _ => {
                return Err(self.error(&format!("unexpected character '{}'", c)));
            }
        }
        Ok(())
    }

    fn keyword_map() -> &'static HashMap<String, TokenType> {
        use std::sync::OnceLock;
        static KEYWORDS: OnceLock<HashMap<String, TokenType>> = OnceLock::new();
        KEYWORDS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("package".into(), TokenType::Package);
            m.insert("import".into(), TokenType::Import);
            m.insert("struct".into(), TokenType::Struct);
            m.insert("func".into(), TokenType::Func);
            m.insert("static".into(), TokenType::Static);
            m.insert("final".into(), TokenType::Final);
            m.insert("permits".into(), TokenType::Permits);
            m.insert("if".into(), TokenType::If);
            m.insert("else".into(), TokenType::Else);
            m.insert("return".into(), TokenType::Return);
            m.insert("true".into(), TokenType::True);
            m.insert("false".into(), TokenType::False);
            m.insert("null".into(), TokenType::Null);
            m.insert("int".into(), TokenType::IntType);
            m.insert("float".into(), TokenType::FloatType);
            m.insert("string".into(), TokenType::StringType);
            m.insert("bool".into(), TokenType::BoolType);
            m.insert("void".into(), TokenType::VoidType);
            m
        })
    }

    fn error(&self, msg: &str) -> String {
        format!("[{}:{}] {}", self.line, self.col, msg)
    }
}
