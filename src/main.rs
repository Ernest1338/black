use crate::{compiler::Compiler, interpreter::Interpreter};
use std::fs::read_to_string;

mod args;
use args::get_args;

mod compiler;

mod interpreter;

mod parser;
use parser::{lexer, preprocess, Expr, Parser};

fn main() {
    let args = get_args();

    // Reading source code
    let source_code = read_to_string(args.input).expect("Error: can not read source code file");

    // Preprocessing
    let source_code = preprocess(&source_code);

    // Lexical Analysis
    let tokens = lexer(&source_code).expect("Lexer failed");
    // println!("Tokens: {:?}", tokens);

    // Parsing
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().expect("Parser failed");
    // println!("AST: {:#?}", &ast);

    if args.interpreter {
        // Interpreter
        let mut interpreter = Interpreter::from_ast(ast);
        interpreter.run();
    } else {
        // Compiler
        let compiler = Compiler::from_ast(ast);
        compiler.compile(args.output);
    }
}
