use std::fmt;

// ── Type ──

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub name: String,
}

impl Type {
    pub const INT: Type = Type {
        name: String::new(), // placeholder, see INT_STR below
    };
    pub const INT_STR: &str = "int";
    pub const FLOAT_STR: &str = "float";
    pub const STRING_STR: &str = "string";
    pub const BOOL_STR: &str = "bool";
    pub const VOID_STR: &str = "void";

    pub fn new(name: impl Into<String>) -> Self {
        Type { name: name.into() }
    }

    pub fn int() -> Self {
        Type::new(Self::INT_STR)
    }
    pub fn float() -> Self {
        Type::new(Self::FLOAT_STR)
    }
    pub fn string() -> Self {
        Type::new(Self::STRING_STR)
    }
    pub fn bool_type() -> Self {
        Type::new(Self::BOOL_STR)
    }
    pub fn void() -> Self {
        Type::new(Self::VOID_STR)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// ── Enum (Modifier) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Static,
    Final,
}

// ── Expressions ──

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(LiteralValue, Type),
    Ident(String),
    Binary {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
    Unary {
        op: String,
        expr: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    /// Struct literal construction: `Rectangle{width: 10.0, height: 20.0}`
    Constructor {
        struct_name: String,
        fields: Vec<FieldInit>,
    },
}

/// Represents literal values in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Int(v) => write!(f, "{}", v),
            LiteralValue::Float(v) => write!(f, "{}", v),
            LiteralValue::String(v) => write!(f, "\"{}\"", v),
            LiteralValue::Bool(v) => write!(f, "{}", v),
            LiteralValue::Null => write!(f, "null"),
        }
    }
}

// ── Statements ──

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ExprStmt(Expression),
    VarDecl {
        name: String,
        explicit_type: Option<Type>,
        init: Option<Box<Expression>>,
    },
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Block(Vec<Statement>),
    If {
        cond: Box<Expression>,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    Return(Option<Box<Expression>>),
}

// ── Method / Function declarations ──

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Statement>,
}

// ── Field declarations ──

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub ty: Type,
    pub init: Option<Box<Expression>>,
}

// ── Struct declarations ──

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    /// Sealed struct 的允许子类型，空表示不密封
    pub permits: Vec<String>,
    pub modifiers: Vec<Modifier>,
    pub fields: Vec<FieldDecl>,
    /// 匿名组合
    pub anonymous_fields: Vec<StructDecl>,
    pub methods: Vec<MethodDecl>,
}

impl StructDecl {
    pub fn is_sealed(&self) -> bool {
        !self.permits.is_empty()
    }
}

// ── Top-level program ──

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub package_name: String,
    pub structs: Vec<StructDecl>,
    pub funcs: Vec<MethodDecl>,
}

// ── AstNode trait for shared behavior ──

/// Everything is an AST node.
pub trait AstNode: fmt::Debug {
    fn node_type(&self) -> &'static str;
}

impl AstNode for Type {
    fn node_type(&self) -> &'static str {
        "Type"
    }
}

impl AstNode for Expression {
    fn node_type(&self) -> &'static str {
        "Expression"
    }
}

impl AstNode for Statement {
    fn node_type(&self) -> &'static str {
        "Statement"
    }
}

impl AstNode for Program {
    fn node_type(&self) -> &'static str {
        "Program"
    }
}

impl AstNode for StructDecl {
    fn node_type(&self) -> &'static str {
        "StructDecl"
    }
}

impl AstNode for FieldDecl {
    fn node_type(&self) -> &'static str {
        "FieldDecl"
    }
}

impl AstNode for MethodDecl {
    fn node_type(&self) -> &'static str {
        "MethodDecl"
    }
}
