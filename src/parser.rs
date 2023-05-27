use crate::{
    ast::{BinaryOperator, Expr, UnaryOperator},
    lexer::tokenize_with_context,
    token::Token,
    token::WithSpan,
};

pub struct Parser {
    tokens: Vec<WithSpan<Token>>,
    source: Vec<char>,
    cursor: usize,
}

pub fn parse(src: &str) -> Expr {
    Parser::new(src).parse()
}

impl Parser {
    pub fn new(source: &str) -> Self {
        Self {
            tokens: tokenize_with_context(source),
            source: source.chars().collect(),
            cursor: 0,
        }
    }

    pub fn parse(mut self) -> Expr {
        self.expr()
    }

    // primary => NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")"
    fn primary(&mut self) -> Expr {
        if let Some(token) = self.peek() {
            self.advance();
            match token {
                Token::Nil => Expr::Nil,
                Token::False => Expr::Boolean(false),
                Token::True => Expr::Boolean(true),
                Token::Number => Expr::Number(self.parse_number(self.cursor - 1)),
                Token::String => Expr::String(self.parse_string(self.cursor - 1)),
                Token::LeftParen => self.expr(),
                _ => {
                    eprintln!("Expected Primary Token: Found {:?}", token);
                    Expr::Nil
                }
            }
        } else {
            eprintln!("Expected Primary Token");
            Expr::Nil
        }
    }

    // unary => ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Expr {
        if self.match_token(Token::Not) | self.match_token(Token::Minus) {
            let operator = match self.previous_token() {
                Token::Not => UnaryOperator::Not,
                Token::Minus => UnaryOperator::Minus,
                _ => UnaryOperator::Not,
            };
            let right = self.unary();
            return Expr::Unary(operator, Box::new(right));
        }
        self.primary()
    }

    // factor => unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Expr {
        let mut left = self.unary();
        while self.match_token(Token::Star) | self.match_token(Token::Slash) {
            let operator = parse_binary_operator(self.previous_token());
            let right = self.unary();
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        left
    }

    // term    => factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Expr {
        let mut left = self.factor();

        while self.match_token(Token::Plus) | self.match_token(Token::Minus) {
            let operator = parse_binary_operator(self.previous_token());
            let right = self.factor();
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        left
    }

    // comparison => term ( ( ">" | ">=" | "<", | "<=" ) term )
    fn comparison(&mut self) -> Expr {
        let mut left = self.term();

        while self.match_token(Token::Greater)
            | self.match_token(Token::GreaterEqual)
            | self.match_token(Token::Less)
            | self.match_token(Token::LessEqual)
        {
            let operator = parse_binary_operator(self.previous_token());
            let right = self.term();
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        left
    }

    // equality => comparision ( ( "!=" | "==" ) comparision )
    fn equality(&mut self) -> Expr {
        let mut left = self.comparison();

        while self.match_token(Token::NotEqual) || self.match_token(Token::EqualEqual) {
            let operator = parse_binary_operator(self.previous_token());
            let right = self.comparison();
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        left
    }

    /* expression     → equality ;
     * equality       → comparison ( ( "!=" | "==" ) comparison )* ;
     * comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
     * term           → factor ( ( "-" | "+" ) factor )* ;
     * factor         → unary ( ( "/" | "*" ) unary )* ;
     * unary          → ( "!" | "-" ) unary
     *                | primary ;
     * primary        → NUMBER | STRING | "true" | "false" | "nil"
     *                  | "(" expression ")" ; */
    fn expr(&mut self) -> Expr {
        return self.equality();
    }
}

impl Parser {
    fn match_token(&mut self, token: Token) -> bool {
        if Some(token) == self.peek() {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.cursor].value() == Token::Eof
    }

    fn peek(&self) -> Option<Token> {
        if !self.is_at_end() {
            Some(self.tokens[self.cursor].value())
        } else {
            None
        }
    }

    fn previous_token(&mut self) -> Token {
        self.tokens[self.cursor - 1].value()
    }

    fn parse_string(&mut self, at: usize) -> String {
        let token = self.tokens[at];
        self.source[token.start_pos() + 1..token.end_pos()]
            .iter()
            .collect::<String>()
    }

    fn parse_number(&mut self, at: usize) -> f64 {
        let token = self.tokens[at];
        self.source[token.start_pos()..=token.end_pos()]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap() // TODO: report err
    }
}

fn parse_binary_operator(token: Token) -> BinaryOperator {
    match token {
        Token::Minus => BinaryOperator::Minus,
        Token::Plus => BinaryOperator::Plus,
        Token::Slash => BinaryOperator::Slash,
        Token::Star => BinaryOperator::Star,
        Token::EqualEqual => BinaryOperator::EqualEqual,
        Token::Greater => BinaryOperator::Greater,
        Token::GreaterEqual => BinaryOperator::GreaterEqual,
        Token::Less => BinaryOperator::Less,
        Token::LessEqual => BinaryOperator::LessEqual,
        Token::NotEqual => BinaryOperator::NotEqual,
        _ => {
            eprintln!("Not a binay operator");
            BinaryOperator::EqualEqual
        }
    }
}

#[cfg(test)]
mod test {
    use super::parse;
    use crate::ast::BinaryOperator::*;
    use crate::ast::Expr::*;
    use crate::ast::UnaryOperator;

    #[test]
    fn unary() {
        let left = parse("-10 + 2");
        let right = Binary(
            Box::new(Unary(UnaryOperator::Minus, Box::new(Number(10.0)))),
            Plus,
            Box::new(Number(2.0)),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn binary() {
        let left = parse("10 + 2");
        let right = Binary(Box::new(Number(10.0)), Plus, Box::new(Number(2.0)));

        assert_eq!(left, right);
    }

    #[test]
    fn binary_2() {
        let left = parse("10 / 2 * 5");
        let right = Binary(
            Box::new(Binary(Box::new(Number(10.0)), Slash, Box::new(Number(2.0)))),
            Star,
            Box::new(Number(5.0)),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn binary_grouping() {
        let left = parse("10 / (2 * 5)");
        let right = Binary(
            Box::new(Number(10.0)),
            Slash,
            Box::new(Binary(Box::new(Number(2.0)), Star, Box::new(Number(5.0)))),
        );

        assert_eq!(left, right);
    }

    #[test]
    fn binary_unary() {
        let left = parse("10 / -(2 * 5)");
        let right = Binary(
            Box::new(Number(10.0)),
            Slash,
            Box::new(Unary(
                UnaryOperator::Minus,
                Box::new(Binary(Box::new(Number(2.0)), Star, Box::new(Number(5.0)))),
            )),
        );

        assert_eq!(left, right);
    }
}
