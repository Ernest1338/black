// fn call_qbe() {
//     let s = std::process::Command::new("qbe")
//         .args(["-h"])
//         .output()
//         .unwrap();
//     println!("{}", String::from_utf8(s.stdout).unwrap());
// }

use crate::{parser::Ast, ASTNode};
use std::path::PathBuf;

pub struct Compiler {
    ast: Ast,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self { ast }
    }

    pub fn compile(&self, output_file: PathBuf) {
        println!("compiling into: {output_file:?}");
    }
}
