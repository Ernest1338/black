use crate::{
    parser::{Ast, BinExpr, BinOpKind, FuncCall, Variable, VariableDeclaration},
    Expr,
};
use std::{collections::HashMap, fmt, process::exit};

/// Implements the `Display` trait for the `Variable` enum, allowing formatted output for eg.
/// numbers and string literals
impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variable::Number(n) => write!(f, "{}", n),
            Variable::StringLiteral(s) => write!(f, "{}", s),
        }
    }
}

/// Represents an interpreter that processes an abstract syntax tree (AST) and evaluates expressions
pub struct Interpreter {
    pub ast: Ast,
    pub variables: HashMap<String, Variable>,
}

impl Interpreter {
    /// Creates a new `Interpreter` instance from the provided AST
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            variables: HashMap::new(),
        }
    }

    /// Creates a default `Interpreter` instance with an empty AST and no variables
    pub fn default() -> Self {
        Self {
            ast: Ast::default(),
            variables: HashMap::new(),
        }
    }

    /// Runs the interpreter, processing each expression in the AST
    pub fn run(&mut self) {
        let ast = self.ast.clone();

        for node in &ast {
            match node {
                Expr::FuncCall(func_call) => self.handle_func_call(func_call),
                Expr::VariableDeclaration(variable_declaration) => {
                    self.handle_var_decl(variable_declaration)
                }
                _ => todo!(),
            }
        }
    }

    /// Retrieves the value of a variable, or exits with an error if it doesn't exist
    fn get_var(&self, ident: &str) -> Variable {
        if self.variables.contains_key(ident) {
            if let Some(s) = self.variables.get(ident) {
                return s.clone();
            }
        }
        eprintln!("Error: variable doesn't exist: {ident}");
        exit(1); // FIXME
    }

    /// Evaluates an operand
    fn eval_operand(&self, operand: &Expr) -> i64 {
        match operand {
            Expr::BinExpr(bin_expr) => self.handle_bin_expr(bin_expr),
            Expr::Number(n) => *n,
            Expr::Identifier(id) => match self.get_var(id) {
                Variable::Number(n) => n,
                _ => {
                    eprintln!("Error: cannot add variable which is not a number");
                    exit(1); // FIXME
                }
            },
            _ => todo!(),
        }
    }

    /// Handles the evaluation of a binary expression, returning the result of the operation
    fn handle_bin_expr(&self, bin_expr: &BinExpr) -> i64 {
        let lhs = self.eval_operand(&bin_expr.lhs);
        let rhs = self.eval_operand(&bin_expr.rhs);

        match bin_expr.kind {
            BinOpKind::Plus => lhs + rhs,
            BinOpKind::Minus => lhs - rhs,
            BinOpKind::Multiply => lhs * rhs,
            BinOpKind::Divide => lhs / rhs,
        }
    }

    /// Handles function calls
    fn handle_func_call(&self, func_call: &FuncCall) {
        match func_call.name.as_ref() {
            "print" => {
                for arg in &func_call.arguments {
                    match arg {
                        Expr::FuncCall(func_call) => self.handle_func_call(func_call),
                        Expr::BinExpr(bin_expr) => print!("{}", self.handle_bin_expr(bin_expr)),
                        Expr::Number(n) => print!("{n}"),
                        Expr::Identifier(id) => print!("{}", self.get_var(id)),
                        Expr::StringLiteral(s) => print!("{s}"),
                        _ => eprintln!("Invalid argument to print"),
                    }
                    print!(" ");
                }

                println!();
            }
            _ => {
                // TODO: handle user defined functions
                eprintln!("Error: unknown function: {}", &func_call.name);
                exit(1); // FIXME
            }
        }
    }

    /// Handles variable declarations by storing the variable in the `variables` map and
    /// evaluating its value
    fn handle_var_decl(&mut self, variable_declaration: &VariableDeclaration) {
        self.variables.insert(
            variable_declaration.identifier.clone(),
            match &variable_declaration.value {
                Expr::Number(n) => Variable::Number(*n),
                Expr::StringLiteral(s) => Variable::StringLiteral(s.to_owned()),
                Expr::BinExpr(bin_expr) => Variable::Number(self.handle_bin_expr(bin_expr)),
                _ => {
                    eprintln!("Error: Can only store strings and numbers in variables");
                    exit(1); // FIXME
                }
            },
        );
    }
}
