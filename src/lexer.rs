use crate::token::Token;
use crate::token::WithSpan;

pub struct Lexer<'a> {
    source: &'a [char],
    start_pos: usize,
    cursor: usize, // index of next char will be scanned
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [char]) -> Self {
        Self {
            source,
            start_pos: 0,
            cursor: 0,
        }
    }

    fn number(&mut self) -> Token {
        self.consume_while(|c| c >= '0' && c <= '9');
        if self.consume_if('.') {
            self.consume_while(|c| c >= '0' && c <= '9');
        }
        Token::Number
    }

    fn identifier(&mut self, ch: char) -> Token {
        let mut consumed_chars = self.consume_while(|c| c.is_alphabetic() || c == '_');
        let mut match_chars = vec![ch];
        match_chars.append(&mut consumed_chars);

        match match_chars[..] {
            ['a', 'n', 'd'] => Token::And,
            ['e', 'l', 's', 'e'] => Token::Else,
            ['f', 'a', 'l', 's', 'e'] => Token::False,
            ['f', 'o', 'r'] => Token::For,
            ['f', 'u', 'n'] => Token::Fun,
            ['i', 'f'] => Token::If,
            ['i', 'n'] => Token::In,
            ['n', 'i', 'l'] => Token::Nil,
            ['o', 'r'] => Token::Or,
            ['p', 'r', 'i', 'n', 't'] => Token::Print,
            ['r', 'e', 't', 'u', 'r', 'n'] => Token::Return,
            ['t', 'r', 'u', 'e'] => Token::True,
            ['l', 'e', 't'] => Token::Let,
            _ => Token::Identifier,
        }
    }

    fn string(&mut self) -> Token {
        self.consume_while(|c| c != '"');
        if !self.consume_if('"') {
            println!("Unterminated String.");
            // TODO: produce good error for Unterminated String.
        }
        Token::String
    }

    fn skip_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn skip_comment(&mut self) {
        if self.check_comment() {
            self.consume_while(|c| c != '\n');
            self.skip_whitespace(); // skip newline at end if present
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        self.skip_comment();

        if let Some(ch) = self.next() {
            self.start_pos = self.cursor - 1;

            match ch {
                '(' => Some(Token::LeftParen),
                ')' => Some(Token::RightParen),
                '[' => Some(Token::LeftBracket),
                ']' => Some(Token::RightBracket),
                '{' => Some(Token::LeftBrace),
                '}' => Some(Token::RightBrace),
                ',' => Some(Token::Comma),
                '.' => Some(Token::Dot),
                '-' => Some(Token::Minus),
                '+' => Some(Token::Plus),
                ';' => Some(Token::Semicolon),
                '/' => Some(Token::Slash),
                '*' => Some(Token::Star),
                '!' => Some(self.if_match('=', Token::NotEqual, Token::Not)),
                '=' => Some(self.if_match('=', Token::EqualEqual, Token::Equal)),
                '>' => Some(self.if_match('=', Token::GreaterEqual, Token::Greater)),
                '<' => Some(self.if_match('=', Token::LessEqual, Token::Less)),
                '"' => Some(self.string()),
                '0'..='9' => Some(self.number()),
                'a'..='z' | 'A'..='Z' | '_' => Some(self.identifier(ch)),
                _ => Some(Token::Error),
            }
        } else {
            None
        }
    }

    pub fn tokenize_with_context(&mut self) -> Vec<WithSpan<Token>> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token() {
            tokens.push(WithSpan::new(
                token,
                self.start_pos as u32,
                self.cursor as u32 - 1,
            ));
        }
        tokens.push(WithSpan::new(
            Token::Eof,
            self.cursor as u32 - 1,
            self.cursor as u32 - 1,
        ));
        tokens
    }
}

impl<'a> Lexer<'a> {
    fn is_at_end(&self) -> bool {
        self.cursor == self.source.len()
    }

    fn peek(&mut self) -> Option<char> {
        if !self.is_at_end() {
            Some(self.source[self.cursor])
        } else {
            None
        }
    }

    fn check_comment(&mut self) -> bool {
        if Some(&['/', '/'][..]) == self.source.get(self.cursor..self.cursor + 2) {
            true
        } else {
            false
        }
    }

    fn advance_cursor(&mut self) {
        self.cursor += 1;
    }

    fn next(&mut self) -> Option<char> {
        if !self.is_at_end() {
            self.advance_cursor();
            Some(self.source[self.cursor - 1])
        } else {
            None
        }
    }

    fn if_match(&mut self, ch: char, then: Token, else_: Token) -> Token {
        if Some(ch) == self.peek() {
            self.advance_cursor();
            then
        } else {
            else_
        }
    }

    fn consume_if(&mut self, ch: char) -> bool {
        if Some(ch) == self.peek() {
            self.advance_cursor();
            true
        } else {
            false
        }
    }

    fn consume_while<F>(&mut self, x: F) -> Vec<char>
    where
        F: Fn(char) -> bool,
    {
        let mut consumed = vec![];
        while let Some(c) = self.peek() {
            if x(c) {
                consumed.push(c);
                self.advance_cursor()
            } else {
                break;
            }
        }
        consumed
    }
}

pub fn tokenize_with_context(buf: &[char]) -> Vec<WithSpan<Token>> {
    let mut t = Lexer::new(buf);
    t.tokenize_with_context()
}

#[cfg(test)]
mod tests {
    use super::super::token::Token;
    use super::tokenize_with_context;

    fn tokenize(src: &str) -> Vec<Token> {
        let src: Vec<char> = src.chars().collect();
        tokenize_with_context(&src[..])
            .iter()
            .map(|t| t.value())
            .collect()
    }

    #[test]
    fn test() {
        assert_eq!(
            tokenize("()"),
            vec![Token::LeftParen, Token::RightParen, Token::Eof]
        );

        assert_eq!(
            tokenize("=(!) != ! == > < <= >=[[]]"),
            vec![
                Token::Equal,
                Token::LeftParen,
                Token::Not,
                Token::RightParen,
                Token::NotEqual,
                Token::Not,
                Token::EqualEqual,
                Token::Greater,
                Token::Less,
                Token::LessEqual,
                Token::GreaterEqual,
                Token::LeftBracket,
                Token::LeftBracket,
                Token::RightBracket,
                Token::RightBracket,
                Token::Eof,
            ]
        );

        assert_eq!(
            tokenize("fun sayhello() { print \"hello\"; }"),
            vec![
                Token::Fun,
                Token::Identifier,
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBrace,
                Token::Print,
                Token::String,
                Token::Semicolon,
                Token::RightBrace,
                Token::Eof,
            ]
        );

        assert_eq!(
            tokenize("fun _count() { for (let i = 0; i < 10; i = i + 1) { print i; } }"),
            vec![
                Token::Fun,        // fun
                Token::Identifier, // count
                Token::LeftParen,  // )
                Token::RightParen, // )
                Token::LeftBrace,  // {
                Token::For,        // for
                Token::LeftParen,  // (
                Token::Let,        // let
                Token::Identifier, // i
                Token::Equal,      // =
                Token::Number,     // 0
                Token::Semicolon,  // ;
                Token::Identifier, // i
                Token::Less,       // <
                Token::Number,     // 10
                Token::Semicolon,  // ;
                Token::Identifier, // i
                Token::Equal,      // =
                Token::Identifier, // i
                Token::Plus,       // +
                Token::Number,     // 1
                Token::RightParen, // )
                Token::LeftBrace,  // {
                Token::Print,      // print
                Token::Identifier, // i
                Token::Semicolon,  // ;
                Token::RightBrace, // }
                Token::RightBrace, // }
                Token::Eof,
            ]
        )
    }
}
