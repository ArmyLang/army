use std::fmt;

/// 所有词法单元类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // ── keywords ──
    Package,
    Import,
    Struct,
    Func,
    Static,
    Final,
    Permits,
    If,
    Else,
    Return,
    True,
    False,
    Null,

    // ── types ──
    IntType,
    FloatType,
    StringType,
    BoolType,
    VoidType,

    // ── literals ──
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    Ident,

    // ── operators / delimiters ──
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Assign,
    ColonDefine, // :=
    Dot,
    Comma,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,

    // ── special ──
    Newline,
    Eof,
}

impl TokenType {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenType::IntLiteral
                | TokenType::FloatLiteral
                | TokenType::StringLiteral
                | TokenType::True
                | TokenType::False
                | TokenType::Null
        )
    }

    pub fn is_type(&self) -> bool {
        matches!(
            self,
            TokenType::IntType
                | TokenType::FloatType
                | TokenType::StringType
                | TokenType::BoolType
                | TokenType::VoidType
        )
    }

    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            TokenType::Eq | TokenType::Neq | TokenType::Lt | TokenType::Gt | TokenType::Le | TokenType::Ge
        )
    }
}

/// 词法单元：类型 + 词素 + 位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn eof(line: usize, col: usize) -> Self {
        Token {
            ty: TokenType::Eof,
            lexeme: String::new(),
            line,
            col,
        }
    }

    pub fn newline(line: usize, col: usize) -> Self {
        Token {
            ty: TokenType::Newline,
            lexeme: "\\n".to_string(),
            line,
            col,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] {:?} '{}'", self.line, self.col, self.ty, self.lexeme)
    }
}
