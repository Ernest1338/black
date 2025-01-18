use crate::{
    parser::{Ast, BinExpr, BinOpKind, FuncCall, Variable, VariableDeclaration},
    utils::ErrorType,
    Expr,
};
use std::{collections::HashMap, fmt};

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
    pub fn run(&mut self) -> Result<(), ErrorType> {
        let ast = self.ast.clone();

        for node in &ast {
            match node {
                Expr::FuncCall(func_call) => self.handle_func_call(func_call)?,
                Expr::VariableDeclaration(variable_declaration) => {
                    self.handle_var_decl(variable_declaration)?
                }
                Expr::Identifier(id) => {
                    // If it's a valid variable, print it
                    // Probably only useful in the interactive mode
                    // Should we only restrict this code to such condition?
                    let var = self.get_var(id)?;
                    println!("{var}");
                }
                _ => return Err(ErrorType::Generic(format!("Not implemented: {node:?}"))),
            }
        }

        Ok(())
    }

    /// Retrieves the value of a variable, or exits with an error if it doesn't exist
    fn get_var(&self, ident: &str) -> Result<Variable, ErrorType> {
        if self.variables.contains_key(ident) {
            if let Some(s) = self.variables.get(ident) {
                return Ok(s.clone());
            }
        }
        Err(ErrorType::SyntaxError(format!(
            "Variable doesn't exist: `{ident}`"
        )))
    }

    /// Evaluates an operand
    fn eval_operand(&self, operand: &Expr) -> Result<i64, ErrorType> {
        match operand {
            Expr::BinExpr(bin_expr) => Ok(self.handle_bin_expr(bin_expr)?),
            Expr::Number(n) => Ok(*n),
            Expr::Identifier(id) => match self.get_var(id)? {
                Variable::Number(n) => Ok(n),
                _ => Err(ErrorType::Generic(
                    "Cannot add variable which is not a number".to_string(),
                )),
            },
            _ => Err(ErrorType::Generic(
                "Cannot add variable which is not a number".to_string(),
            )),
        }
    }

    /// Handles the evaluation of a binary expression, returning the result of the operation
    fn handle_bin_expr(&self, bin_expr: &BinExpr) -> Result<i64, ErrorType> {
        let lhs = self.eval_operand(&bin_expr.lhs)?;
        let rhs = self.eval_operand(&bin_expr.rhs)?;

        match bin_expr.kind {
            BinOpKind::Plus => Ok(lhs + rhs),
            BinOpKind::Minus => Ok(lhs - rhs),
            BinOpKind::Multiply => Ok(lhs * rhs),
            BinOpKind::Divide => Ok(lhs / rhs),
        }
    }

    /// Handles function calls
    fn handle_func_call(&self, func_call: &FuncCall) -> Result<(), ErrorType> {
        match func_call.name.as_ref() {
            "print" => {
                let args = func_call.arguments.iter();
                let args_count = args.len();
                for (i, arg) in args.enumerate() {
                    match arg {
                        Expr::FuncCall(func_call) => self.handle_func_call(func_call)?,
                        Expr::BinExpr(bin_expr) => print!("{}", self.handle_bin_expr(bin_expr)?),
                        Expr::Number(n) => print!("{n}"),
                        Expr::Identifier(id) => print!("{}", self.get_var(id)?),
                        Expr::StringLiteral(s) => print!("{s}"),
                        _ => {
                            return Err(ErrorType::Generic("Invalid argument to print".to_string()))
                        }
                    }
                    if i != args_count - 1 {
                        print!(" ");
                    }
                }

                println!();
            }
            _ => {
                // TODO: handle user defined functions
                return Err(ErrorType::Generic(format!(
                    "Unknown function: {}",
                    &func_call.name
                )));
            }
        }

        Ok(())
    }

    /// Handles variable declarations by storing the variable in the `variables` map and
    /// evaluating its value
    fn handle_var_decl(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), ErrorType> {
        self.variables.insert(
            variable_declaration.identifier.clone(),
            match &variable_declaration.value {
                Expr::Number(n) => Variable::Number(*n),
                Expr::StringLiteral(s) => Variable::StringLiteral(s.to_owned()),
                Expr::BinExpr(bin_expr) => Variable::Number(self.handle_bin_expr(bin_expr)?),
                _ => {
                    return Err(ErrorType::Generic(
                        "Can only store strings and number in variables".to_string(),
                    ));
                }
            },
        );

        Ok(())
    }
}
