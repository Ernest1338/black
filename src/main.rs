mod args;
use crate::compiler::Compiler;
use crate::interpreter::Interpreter;
use args::get_args;

mod compiler;
// use compiler::call_qbe;

mod interpreter;

mod parser;
use parser::{lexer, parser, ASTNode};

fn main() {
    let args = get_args();

    let sourcecode = std::fs::read_to_string(args.input).unwrap();

    // Lexical Analysis
    let tokens = lexer(&sourcecode).expect("Lexer failed");
    // println!("Tokens: {:?}", tokens);

    // Parsing
    let ast = parser(&tokens).expect("Parser failed");
    // println!("AST: {:?}", &ast);

    ast.interpret();
    ast.compile();

    // call_qbe();
}
