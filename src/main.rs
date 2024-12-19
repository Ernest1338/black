// TODO: remove those
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use crate::{compiler::Compiler, interpreter::Interpreter};
use std::fs::read_to_string;

mod args;
use args::get_args;

mod compiler;

mod interpreter;

mod parser;
use parser::{lexer, parser, ASTNode};

pub struct State {
    data_section: Vec<(String, String)>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            data_section: Vec::new(),
        }
    }
}

fn main() {
    let args = get_args();

    let source_code = read_to_string(args.input).unwrap();

    // Lexical Analysis
    let tokens = lexer(&source_code).expect("Lexer failed");
    println!("Tokens: {:?}", tokens);

    // Parsing
    // let ast = parser(&tokens).expect("Parser failed");
    // println!("AST: {:?}", &ast);

    // Interpreter
    // let ast = parser(&tokens).expect("Parser failed");
    // let interpreter = Interpreter::from_ast(ast);
    // interpreter.run();

    // Compiler
    // let ast = parser(&tokens).expect("Parser failed");
    // let compiler = Compiler::from_ast(ast);
    // let ir = compiler.compile(args.output);
    // println!("compiled:\n{}", ir);
}
