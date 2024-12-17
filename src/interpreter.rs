use crate::{parser::AST, ASTNode};

pub trait Interpreter {
    fn interpret(&self);
}

impl Interpreter for AST {
    fn interpret(&self) {
        for node in self {
            match node {
                ASTNode::Print(message) => println!("{}", message),
            }
        }
    }
}
