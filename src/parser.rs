use crate::{
    ast::{Ast, BinaryOperator, Expr, Stmt, UnaryOperator},
    lexer::tokenize_with_context,
    token::Token,
    token::WithSpan,
};

pub struct Parser {
    tokens: Vec<WithSpan<Token>>,
    source: Vec<char>,
    cursor: usize,
    errors: Vec<String>,
}

pub fn parse(src: &str) -> Vec<Stmt> {
    Parser::new(src).parse()
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let source: Vec<char> = source.chars().collect();
        Self {
            tokens: tokenize_with_context(&source[..]),
            source,
            cursor: 0,
            errors: vec![],
        }
    }

    // Program => statement* EOF;
    pub fn parse(mut self) -> Ast {
        // DEBUG:
        for token in &self.tokens {
            println!("{:?}", token.value);
        }
        //

        let mut statements = vec![];
        while !self.is_at_end() {
            if let Ok(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                self.synchronize();
            }
        }
        // DEBUG:
        for err in &self.errors {
            print!("{}", err);
        }
        //
        statements
    }
}

/// Statements
impl Parser {
    fn declaration(&mut self) -> Result<Stmt, ()> {
        if let Some(token) = self.peek() {
            match token {
                Token::Let => self.var_declaration(),
                _ => self.statement(),
            }
        } else {
            Err(())
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ()> {
        self.advance(); // advance Let token
        self.consume(Token::Identifier, "Expected Identifier")?;
        let identifier_lexeme = self.parse_name(self.cursor - 1);

        let mut initializer_expr: Option<Expr> = None;
        if self.match_token(Token::Equal) {
            initializer_expr = Some(self.expr()?);
        }

        self.consume(Token::Semicolon, "Expected ';' after expr.")?;
        Ok(Stmt::Let(identifier_lexeme, initializer_expr))
    }

    // Statement => ExprStmt | PrintStmt | Block;
    fn statement(&mut self) -> Result<Stmt, ()> {
        if let Some(token) = self.peek() {
            match token {
                Token::LeftBrace => self.block(),
                Token::Print => self.print_stmt(),
                _ => self.expr_stmt(),
            }
        } else {
            Err(())
        }
    }

    // Block => Declarations* ;
    fn block(&mut self) -> Result<Stmt, ()> {
        self.advance(); // advance '{' token

        let mut stmts = vec![];

        while !self.check(Token::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(Token::RightBrace, "Expected '}' after block.")?;

        Ok(Stmt::Block(stmts))
    }

    fn print_stmt(&mut self) -> Result<Stmt, ()> {
        self.advance(); // advance Print token

        match self.expr() {
            Ok(expr) => {
                self.consume(Token::Semicolon, "Expected ';' after expr")?;
                Ok(Stmt::Print(expr))
            }
            Err(_) => {
                self.error_at(self.cursor - 1, "Expected Expression after print.");
                Err(())
            }
        }
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ()> {
        // an expr_stmt is expr which end with semicolon.
        match self.expr() {
            Ok(expr) => {
                self.consume(Token::Semicolon, "Expected ';' after expr.")?;
                Ok(Stmt::Expr(expr))
            }
            Err(_) => {
                self.error_at(self.cursor, "Expected expr statement.");
                Err(())
            }
        }
    }
}

/// Expression
impl Parser {
    fn grouping(&mut self) -> Result<Expr, ()> {
        let expr = self.expr();
        self.consume(Token::RightParen, "Expected ')' after expr.")?;
        expr
    }

    // primary => IDENTIFIER | NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")"
    fn primary(&mut self) -> Result<Expr, ()> {
        if let Some(token) = self.next() {
            match token {
                Token::Nil => Ok(Expr::Nil),
                Token::False => Ok(Expr::Boolean(false)),
                Token::True => Ok(Expr::Boolean(true)),
                Token::Number => Ok(Expr::Number(self.parse_number(self.cursor - 1))),
                Token::String => Ok(Expr::String(self.parse_string(self.cursor - 1))),
                Token::Identifier => Ok(Expr::Variable(self.parse_name(self.cursor - 1))),
                Token::LeftParen => self.grouping(),
                _ => Err(()),
            }
        } else {
            self.error_at(self.cursor, "Expected Primary Token");
            Err(())
        }
    }

    // unary => ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<Expr, ()> {
        if self.match_token(Token::Not) | self.match_token(Token::Minus) {
            let operator = parser_unary_operator(self.previous_token())?;
            let right = self.unary()?;
            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }

    // factor => unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, ()> {
        let mut left = self.unary()?;
        while self.match_token(Token::Star) | self.match_token(Token::Slash) {
            let operator = parse_binary_operator(self.previous_token())?;
            let right = self.unary()?;
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    // term    => factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, ()> {
        let mut left = self.factor()?;

        while self.match_token(Token::Plus) | self.match_token(Token::Minus) {
            let operator = parse_binary_operator(self.previous_token())?;
            let right = self.factor()?;
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    // comparison => term ( ( ">" | ">=" | "<", | "<=" ) term )
    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut left = self.term()?;

        while self.match_token(Token::Greater)
            | self.match_token(Token::GreaterEqual)
            | self.match_token(Token::Less)
            | self.match_token(Token::LessEqual)
        {
            let operator = parse_binary_operator(self.previous_token())?;
            let right = self.term()?;
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    // equality => comparision ( ( "!=" | "==" ) comparision )
    fn equality(&mut self) -> Result<Expr, ()> {
        let mut left = self.comparison()?;

        while self.match_token(Token::NotEqual) || self.match_token(Token::EqualEqual) {
            let operator = parse_binary_operator(self.previous_token())?;
            let right = self.comparison()?;
            left = Expr::Binary(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    fn assignment(&mut self) -> Result<Expr, ()> {
        // e.g [ a = "hari" ]
        let left_expr = self.equality()?;

        if self.match_token(Token::Equal) {
            let equal_index = self.cursor - 1; // for err reporting

            // e.g [ a = b = "hari" ], hence parse right_hand_side as assignment itself.
            let right_expr = self.assignment()?;

            if let Expr::Variable(name) = left_expr {
                return Ok(Expr::Assignment(name, Box::new(right_expr)));
            };

            self.error_at(equal_index, "Invalid assignment target.")
        }

        Ok(left_expr)
    }

    /* expression     → assignment;
     * assignment     → IDENTIFIER '=' assignment | equality ;
     * equality       → comparison ( ( "!=" | "==" ) comparison )* ;
     * comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
     * term           → factor ( ( "-" | "+" ) factor )* ;
     * factor         → unary ( ( "/" | "*" ) unary )* ;
     * unary          → ( "!" | "-" ) unary
     *                | primary ;
     * primary        → NUMBER | STRING | "true" | "false" | "nil"
     *                  | "(" expression ")" ; */
    pub fn expr(&mut self) -> Result<Expr, ()> {
        // self.equality()
        self.assignment()
    }
}

/// Tokens
impl Parser {
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            println!("IN Sync"); //////// REMOVE:
            if self.previous_token() == Token::Semicolon {
                return;
            }

            if let Some(token) = self.peek() {
                match token {
                    Token::For
                    | Token::Fun
                    | Token::If
                    | Token::Print
                    | Token::Return
                    | Token::Let
                    | Token::Eof => return,
                    _ => self.advance(),
                }
            }
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        if Some(token) == self.peek() {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token: Token, msg: &str) -> Result<(), ()> {
        if Some(token) == self.peek() {
            self.advance();
            Ok(())
        } else {
            self.error_at(self.cursor, msg);
            Err(())
        }
    }

    fn next(&mut self) -> Option<Token> {
        if let Some(token) = self.peek() {
            self.advance();
            Some(token)
        } else {
            None
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

    fn check(&self, token: Token) -> bool {
        if Some(token) == self.peek() {
            true
        } else {
            false
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

    fn parse_name(&mut self, at: usize) -> String {
        let token = self.tokens[at];
        self.source[token.start_pos()..=token.end_pos()]
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

    fn get_line(&self, token_start_pos: usize) -> u32 {
        let mut line = 1;
        for i in 0..=token_start_pos {
            if self.source[i] == '\n' {
                line += 1;
            }
        }

        line
    }

    fn error_at(&mut self, at: usize, msg: &str) {
        // let line = "Todo"; // TODO
        let token = self.tokens[at];
        let line_number = self.get_line(token.start_pos());

        self.errors.push(format!(
            "\nParseErr: {msg}\nAtLine [{line_number}] AtToken[{token:?}]\n\n"
        ));
    }
}

fn parse_binary_operator(token: Token) -> Result<BinaryOperator, ()> {
    match token {
        Token::Minus => Ok(BinaryOperator::Minus),
        Token::Plus => Ok(BinaryOperator::Plus),
        Token::Slash => Ok(BinaryOperator::Slash),
        Token::Star => Ok(BinaryOperator::Star),
        Token::EqualEqual => Ok(BinaryOperator::EqualEqual),
        Token::Greater => Ok(BinaryOperator::Greater),
        Token::GreaterEqual => Ok(BinaryOperator::GreaterEqual),
        Token::Less => Ok(BinaryOperator::Less),
        Token::LessEqual => Ok(BinaryOperator::LessEqual),
        Token::NotEqual => Ok(BinaryOperator::NotEqual),
        _ => {
            eprintln!("Err parsing binay operator from token");
            Err(())
        }
    }
}

fn parser_unary_operator(token: Token) -> Result<UnaryOperator, ()> {
    match token {
        Token::Not => Ok(UnaryOperator::Not),
        Token::Minus => Ok(UnaryOperator::Minus),
        _ => {
            eprintln!("Err parsing unary operator from token.");
            return Err(());
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::ast::{BinaryOperator::*, Expr, Expr::*, UnaryOperator};
    use super::Parser;

    fn parse_expression(src: &str) -> Result<Expr, ()> {
        Parser::new(src).expr()
    }

    #[test]
    fn unary() {
        let left = parse_expression("-10 + 2");
        let right = Binary(
            Box::new(Unary(UnaryOperator::Minus, Box::new(Number(10.0)))),
            Plus,
            Box::new(Number(2.0)),
        );

        assert_eq!(left, Ok(right));
    }

    #[test]
    fn unary_2() {
        let left = parse_expression("-1;");
        let right = Unary(UnaryOperator::Minus, Box::new(Number(1.0)));

        assert_eq!(left, Ok(right));
    }

    #[test]
    fn binary() {
        let left = parse_expression("10 + 2");
        let right = Binary(Box::new(Number(10.0)), Plus, Box::new(Number(2.0)));

        assert_eq!(left, Ok(right));
    }

    #[test]
    fn binary_2() {
        let left = parse_expression("10 / 2 * 5");
        let right = Binary(
            Box::new(Binary(Box::new(Number(10.0)), Slash, Box::new(Number(2.0)))),
            Star,
            Box::new(Number(5.0)),
        );

        assert_eq!(left, Ok(right));
    }

    #[test]
    fn binary_grouping() {
        let left = parse_expression("10 / (2 * 5)");
        let right = Binary(
            Box::new(Number(10.0)),
            Slash,
            Box::new(Binary(Box::new(Number(2.0)), Star, Box::new(Number(5.0)))),
        );

        assert_eq!(left, Ok(right));
    }

    #[test]
    fn binary_unary() {
        let left = parse_expression("10 / -(2 * 5) + 2");
        let right = Binary(
            Box::new(Binary(
                Box::new(Number(10.0)),
                Slash,
                Box::new(Unary(
                    UnaryOperator::Minus,
                    Box::new(Binary(Box::new(Number(2.0)), Star, Box::new(Number(5.0)))),
                )),
            )),
            Plus,
            Box::new(Number(2.0)),
        );

        assert_eq!(left, Ok(right));
    }
}
