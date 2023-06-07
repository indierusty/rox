use crate::ast::{BinaryOperator, Expr, LogicalOperator, Stmt, UnaryOperator};
use crate::environment::Environment;
use crate::parser::parse;
use crate::value::Value;

pub struct Interpreter {
    envs: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            envs: Environment::new(),
        }
    }

    pub fn interpret(&mut self, src: &str) {
        let ast = parse(src);
        for stmt in ast {
            println!("Stmt => {:?}", stmt); // DEBUG:
            match self.run(stmt) {
                Ok(_) => {}
                Err(err) => println!("RuntimeErr: {}", err),
            };
        }
    }

    fn run(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => self.block(stmts),
            Stmt::Expr(expr) => self.expr_stmt(expr),
            Stmt::If(expr, then_stmt, else_stmt) => self.if_stmt(expr, then_stmt, else_stmt),
            Stmt::Print(expr) => self.print_stmt(expr),
            Stmt::Let(i, e) => self.let_stmt(i, e),
        }
    }
}

/// Statement
impl Interpreter {
    fn block(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {
        self.envs.begin_scope();

        for stmt in stmts {
            self.run(stmt)?
        }

        self.envs.end_scope();
        Ok(())
    }

    fn expr_stmt(&mut self, expr: Expr) -> Result<(), String> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn if_stmt(
        &mut self,
        expr: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    ) -> Result<(), String> {
        if self.evaluate(expr)? == Value::Bool(true) {
            self.run(*then_stmt)?;
        } else {
            if let Some(stmt) = else_stmt {
                self.run(*stmt)?;
            }
        }
        Ok(())
    }

    fn print_stmt(&mut self, expr: Expr) -> Result<(), String> {
        let value = self.evaluate(expr)?;
        println!("{:?}", value);
        Ok(())
    }

    fn let_stmt(&mut self, name: String, initializer: Option<Expr>) -> Result<(), String> {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }

        self.envs.define_var(name, Some(value));
        Ok(())
    }
}

/// Expression
impl Interpreter {
    fn evaluate(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Binary(l, o, r) => self.binary(l, o, r),
            Expr::Logical(l, o, r) => self.logical(l, o, r),
            Expr::Unary(o, expr) => self.unary(o, expr),
            Expr::Number(number) => Ok(Value::Num(number)),
            Expr::Boolean(value) => Ok(Value::Bool(value)),
            Expr::Nil => Ok(Value::Nil),
            Expr::String(s) => Ok(Value::String(s)),
            Expr::Variable(var) => self.variable(var), // TODO:
            Expr::Assignment(name, expr) => self.assignment(name, expr),
        }
    }

    fn assignment(&mut self, name: String, expr: Box<Expr>) -> Result<Value, String> {
        let value = self.evaluate(*expr)?;
        self.envs.assign_var(name, value)
    }

    fn variable(&mut self, identifier: String) -> Result<Value, String> {
        self.envs.get_var(identifier)
    }

    fn logical(
        &mut self,
        left: Box<Expr>,
        op: LogicalOperator,
        right: Box<Expr>,
    ) -> Result<Value, String> {
        let left = self.evaluate(*left)?;

        match op {
            LogicalOperator::And => {
                if left == Value::Bool(false) {
                    return Ok(left);
                }
            }
            LogicalOperator::Or => {
                if left == Value::Bool(true) {
                    return Ok(left);
                }
            }
        }

        self.evaluate(*right)
    }

    fn binary(
        &mut self,
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    ) -> Result<Value, String> {
        let left = self.evaluate(*left)?;
        let right = self.evaluate(*right)?;
        match op {
            BinaryOperator::Slash => left / right,
            BinaryOperator::Star => left * right,
            BinaryOperator::Plus => left + right,
            BinaryOperator::Minus => left - right,
            BinaryOperator::Greater => Ok(Value::Bool(left > right)),
            BinaryOperator::GreaterEqual => Ok(Value::Bool(left >= right)),
            BinaryOperator::Less => Ok(Value::Bool(left < right)),
            BinaryOperator::LessEqual => Ok(Value::Bool(left <= right)),
            BinaryOperator::EqualEqual => Ok(Value::Bool(left == right)),
            BinaryOperator::NotEqual => Ok(Value::Bool(left != right)),
        }
    }

    fn unary(&mut self, op: UnaryOperator, expr: Box<Expr>) -> Result<Value, String> {
        let expr = self.evaluate(*expr)?;
        match op {
            UnaryOperator::Not => !Value::from(expr),
            UnaryOperator::Minus => !Value::from(expr),
        }
    }
}
