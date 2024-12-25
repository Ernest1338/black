use crate::{compiler::Compiler, interpreter::Interpreter};
use std::{
    fs::read_to_string,
    io::{stdin, stdout, Write},
    process::exit,
};

// TODO:
// - good compiler errors with line numbers
// - handling errors in rust (+ make interpreter not crash on error)

mod args;
use args::get_args;

mod compiler;

mod interpreter;

mod parser;
use parser::{lexer, preprocess, Expr, Parser};

const INTERACTIVE_BANNER: &str = "\
╭──────────────────────╮
│   ☠︎︎  Black Lang  ☠︎︎   │
│                      │
│ ⚓Interactive mode   │
╰──────────────────────╯
";

fn print_and_flush(m: &str) {
    print!("{m}");
    stdout().flush().unwrap();
}

fn main() {
    let args = get_args();

    if args.input.is_none() {
        // Interactive mode
        print_and_flush(INTERACTIVE_BANNER);
        let mut interpreter = Interpreter::default();
        loop {
            print_and_flush(">>> ");
            let mut input = String::new();
            loop {
                let mut tmp = String::new();
                stdin()
                    .read_line(&mut tmp)
                    .expect("Error: reading user input");

                // Short circuit exit on "exit" or "quit"
                if ["exit", "quit"].contains(&tmp.trim()) {
                    exit(0);
                }

                input.push_str(&tmp);
                if input.ends_with("\n\n") {
                    break;
                }

                print_and_flush("  … ");
            }
            input = input.trim().to_string();

            let code = preprocess(&input);
            let tokens = lexer(&code).expect("Lexer failed");
            let mut parser = Parser::new(&tokens);
            let ast = parser.parse().expect("Parser failed");
            interpreter.ast = ast;

            // Clear last line
            print!("\x1b[1A\x1b[2K");

            interpreter.run();
        }
    }

    // Reading source code
    let source_code = match args.input {
        Some(input) => read_to_string(input).expect("Error: can not read source code file"),
        None => panic!("Input argument unexpectedly None. This is a bug."),
    };

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
