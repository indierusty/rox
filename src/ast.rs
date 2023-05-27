#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOperator {
    Slash,
    Star,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    EqualEqual,
    NotEqual,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Unary(UnaryOperator, Box<Expr>),
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Variable(String),
}
