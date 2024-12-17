// pub fn call_qbe() {
//     let s = std::process::Command::new("qbe")
//         .args(["-h"])
//         .output()
//         .unwrap();
//     println!("{}", String::from_utf8(s.stdout).unwrap());
// }

use crate::{parser::AST, ASTNode};

pub trait Compiler {
    fn compile(&self);
}

impl Compiler for AST {
    fn compile(&self) {
        for node in self {
            match node {
                ASTNode::Print(message) => println!("{}", message),
            }
        }
    }
}
