use crate::{
    parser::{Ast, BinExpr, BinOpKind, FuncCall},
    Expr,
};
use std::{collections::HashMap, fmt, process::exit};

#[derive(Debug, Clone)]
pub enum Variable {
    Number(i64),
    StringLiteral(String),
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variable::Number(n) => write!(f, "{}", n),
            Variable::StringLiteral(s) => write!(f, "{}", s),
        }
    }
}

pub struct Interpreter {
    pub ast: Ast,
    pub variables: HashMap<String, Variable>,
}

impl Interpreter {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            variables: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        for node in &self.ast {
            match node {
                Expr::FuncCall(func_call) => {
                    self.handle_func_call(&func_call);
                }
                Expr::VariableDeclaration(variable_declaration) => {
                    self.variables.insert(
                        variable_declaration.identifier.clone(),
                        match &variable_declaration.value {
                            Expr::Number(n) => Variable::Number(*n),
                            Expr::StringLiteral(s) => Variable::StringLiteral(s.to_owned()),
                            Expr::BinExpr(bin_expr) => Variable::Number(Self::handle_bin_expr(&bin_expr)),
                            _ => {
                                eprintln!("Error: Can only store strings and numbers in variables");
                                exit(1);
                            }
                        },
                    );
                }
                _ => todo!(),
            }
        }
    }

    fn handle_bin_expr(bin_expr: &BinExpr) -> i64 {
        let lhs = match &bin_expr.lhs {
            Expr::BinExpr(bin_expr) => &Self::handle_bin_expr(&bin_expr),
            Expr::Number(n) => n,
            _ => todo!(),
        };
        let rhs = match &bin_expr.rhs {
            Expr::BinExpr(bin_expr) => &Self::handle_bin_expr(&bin_expr),
            Expr::Number(n) => n,
            _ => todo!(),
        };
        match bin_expr.kind {
            BinOpKind::Plus => lhs + rhs,
            BinOpKind::Minus => lhs - rhs,
            BinOpKind::Multiply => lhs * rhs,
            BinOpKind::Divide => lhs / rhs,
        }
    }

    fn handle_func_call(&self, func_call: &FuncCall) {
        if func_call.name != "print" {
            println!("Error: prints only supported for now");
        }

        for arg in &func_call.arguments {
            match arg {
                Expr::FuncCall(func_call) => self.handle_func_call(&func_call),
                Expr::BinExpr(bin_expr) => print!("{}", Self::handle_bin_expr(&bin_expr)),
                Expr::Number(n) => print!("{n}"),
                Expr::Identifier(i) => {
                    if self.variables.contains_key(i) {
                        if let Some(s) = self.variables.get(i) {
                            print!("{s}");
                        } else {
                            eprintln!("Error: variable doesn't exist: {i}");
                            exit(1);
                        }
                    }
                }
                Expr::StringLiteral(s) => print!("{s}"),
                _ => eprintln!("Invalid argument to print"),
            }
        }

        println!();
    }
}
