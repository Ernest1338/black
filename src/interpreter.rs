use crate::{parser::Ast, ASTNode};

pub struct Interpreter {
    ast: Ast,
}

impl Interpreter {
    pub fn from_ast(ast: Ast) -> Self {
        Self { ast }
    }

    pub fn run(&self) {
        for node in &self.ast {
            match node {
                ASTNode::Print(message) => println!("{}", message),
            }
        }
    }
}
