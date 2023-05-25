#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Else,
    False,
    For,
    Fun,
    If,
    In,
    Nil,
    Or,
    Print,
    Return,
    True,
    Let,

    Error,
    Eof,
}

#[derive(Debug, Clone, Copy)]
pub struct WithSpan<T> {
    value: T,
    start_pos: u32,
    end_pos: u32,
}

impl<T> WithSpan<T>
where
    T: Copy + Clone,
{
    pub fn new(value: T, start_pos: u32, end_pos: u32) -> Self {
        Self {
            value,
            start_pos,
            end_pos,
        }
    }

    pub fn value(&self) -> T {
        self.value
    }
}
