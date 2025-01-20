use crate::{
    compiler::Compiler,
    interpreter::Interpreter,
    utils::{display_error, display_error_stdout, ErrorType},
};
use std::{
    fs::{canonicalize, read_to_string},
    io::stdin,
    process::{exit, Command, Stdio},
};

// TODO:
// - line numbers in errors
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
use parser::{lexer, preprocess, Parser};

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
    let args = get_args(std::env::args().collect());

    if args.input.is_none() {
        // ----------------
        // Interactive mode
        // ----------------
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
            let tokens = match lexer(&code) {
                Ok(tokens) => tokens,
                Err(err) => {
                    display_error_stdout(ErrorType::Generic(err));
                    continue;
                }
            };
            let mut parser = Parser::new(&tokens);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(err) => {
                    display_error_stdout(ErrorType::Generic(err));
                    continue;
                }
            };
            interpreter.ast = ast;

            // Clear last line
            print!("\x1b[1A\x1b[2K");

            let res = interpreter.run();
            if let Err(err) = res {
                display_error_stdout(err);
            }
        }
    }

    // -------------------
    // Reading source code
    // -------------------
    let source_code = match args.input {
        Some(input) => match read_to_string(input) {
            Ok(input) => input,
            Err(_) => {
                display_error(ErrorType::Generic(
                    "Could not read source code file".to_string(),
                ));
                exit(1);
            }
        },
        None => panic!("Input argument unexpectedly None. This is a bug."),
    };

    // -------------
    // Preprocessing
    // -------------
    let source_code = measure_time("Preprocessing", || preprocess(&source_code));

    // ----------------
    // Lexical Analysis
    // ----------------
    let tokens = measure_time("Lexical Analysis", || match lexer(&source_code) {
        Ok(tokens) => tokens,
        Err(err) => {
            display_error(ErrorType::SyntaxError(err));
            exit(1);
        }
    });
    dbg("Tokens", &tokens);

    // -------
    // Parsing
    // -------
    let mut parser = Parser::new(&tokens);
    let ast = measure_time("Parsing", || match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            display_error(ErrorType::SyntaxError(err));
            exit(1);
        }
    });
    dbg_pretty("AST", &ast);

    if args.interpreter {
        // -----------
        // Interpreter
        // -----------
        let mut interpreter = Interpreter::from_ast(ast);
        measure_time("Interpreter Execution", || {
            if let Err(err) = interpreter.run() {
                display_error(err);
                exit(1);
            }
        });
    } else if args.build_and_run {
        // ---------------
        // Compile and run
        // ---------------
        let mut compiler = Compiler::from_ast(ast);
        measure_time("Full Compiler Execution", || {
            if let Err(err) = compiler.compile(args.output.clone()) {
                display_error(err);
                exit(1);
            }
        });
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
        // --------
        // Compiler
        // --------
        let mut compiler = Compiler::from_ast(ast);
        measure_time("Full Compiler Execution", || {
            if let Err(err) = compiler.compile(args.output) {
                display_error(err);
                exit(1);
            }
        });
    }
}
