use crate::ast::BinaryOperator;
use crate::ast::Expr;
use crate::ast::UnaryOperator;
use crate::parser::parse;

pub fn interpret(src: &str) {
    let ast = parse(src);
    println!("{:?}", ast);
    let result = evaluate(Box::new(ast));
    println!("{}", result);
}

pub fn evaluate(expr: Box<Expr>) -> f64 {
    match *expr {
        Expr::Binary(l, o, r) => binary(l, o, r),
        Expr::Unary(o, expr) => unary(o, expr),
        Expr::Number(number) => number,
        Expr::Boolean(_) => todo!(),
        Expr::Nil => todo!(),
        Expr::String(_) => todo!(),
        Expr::Variable(_) => todo!(),
    }
}

pub fn binary(left: Box<Expr>, op: BinaryOperator, right: Box<Expr>) -> f64 {
    let left = evaluate(left);
    let right = evaluate(right);
    match op {
        BinaryOperator::Slash => left / right,
        BinaryOperator::Star => left * right,
        BinaryOperator::Plus => left + right,
        BinaryOperator::Minus => left - right,
        BinaryOperator::Greater => todo!(),
        BinaryOperator::GreaterEqual => todo!(),
        BinaryOperator::Less => todo!(),
        BinaryOperator::LessEqual => todo!(),
        BinaryOperator::EqualEqual => todo!(),
        BinaryOperator::NotEqual => todo!(),
    }
}

pub fn unary(op: UnaryOperator, expr: Box<Expr>) -> f64 {
    let expr = evaluate(expr);
    match op {
        UnaryOperator::Not => todo!(),
        UnaryOperator::Minus => -expr,
    }
}
