use crate::{compiler::Compiler, interpreter::Interpreter};
use std::{
    fs::{canonicalize, read_to_string},
    io::stdin,
    process::{exit, Command, Stdio},
};

// TODO:
// - test cases for compiler cli
// - good compiler errors with line numbers
// - handling errors in rust (+ make interpreter not crash on error)
// - color error messages, general cli output
// - if, else expr
// - fn expr
// - type checker
// - static build, static qbe in release gh
// - linter
// - formatter
// - build system (toml? github repos as packages. line eg. Ernest1338/package = "0.1")
// - more test cases

mod args;
use args::get_args;

mod compiler;

mod interpreter;

mod parser;
use parser::{lexer, preprocess, Expr, Parser};

mod utils;
use utils::{dbg, dbg_pretty, measure_time, print_and_flush};

mod tests;

/// Interactive mode banner
const INTERACTIVE_BANNER: &str = "\
╭──────────────────────╮
│   ☠︎︎  \x1b[1mBlack Lang\x1b[00m  ☠︎︎   │
│                      │
│ ⚓ \x1b[4mInteractive mode\x1b[00m  │
╰──────────────────────╯
";

/// Entry point of the language CLI
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
    let source_code = measure_time("Preprocessing", || preprocess(&source_code));

    // Lexical Analysis
    let tokens = measure_time("Lexical Analysis", || {
        lexer(&source_code).expect("Lexer failed")
    });
    dbg("Tokens", &tokens);

    // Parsing
    let mut parser = Parser::new(&tokens);
    let ast = measure_time("Parsing", || parser.parse().expect("Parser failed"));
    dbg_pretty("AST", &ast);

    if args.interpreter {
        // Interpreter
        let mut interpreter = Interpreter::from_ast(ast);
        measure_time("Interpreter Execution", || interpreter.run());
    } else if args.build_and_run {
        // Compile
        let mut compiler = Compiler::from_ast(ast);
        measure_time("Full Compiler Execution", || {
            compiler.compile(args.output.clone())
        });
        // Run
        let absolute_path =
            canonicalize(args.output).expect("Error: Failed to get binary absolute path");
        Command::new(absolute_path)
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute binary")
            .wait()
            .expect("Failed to wait for the binary to finish execution");
    } else {
        // Compiler
        let mut compiler = Compiler::from_ast(ast);
        measure_time("Full Compiler Execution", || compiler.compile(args.output));
    }
}
