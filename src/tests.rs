#![allow(dead_code, unused_imports)]

use crate::{
    args::{get_args, AppArgs},
    compiler::Compiler,
    interpreter::Interpreter,
    parser::{lexer, preprocess, Parser},
    utils::{get_tmp_fname, ErrorType},
};
use std::{
    fs::{remove_file, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Output},
};

fn compile_and_run(code: &str) -> String {
    let code_fname = get_tmp_fname("blkcode");
    let bin_fname = get_tmp_fname("blkbin");

    let mut tmp = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(code_fname.clone())
        .unwrap();
    tmp.write_all(code.as_bytes()).unwrap();

    Command::new("cargo")
        .args(["run", "--", "--output", &bin_fname, &code_fname])
        .output()
        .expect("Failed to execute cargo");

    let output = Command::new(&bin_fname)
        .output()
        .expect("Failed to execute test bin");

    remove_file(code_fname).unwrap();
    remove_file(bin_fname).unwrap();

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn interpret(code: &str) -> String {
    let code_fname = get_tmp_fname("blkcode");

    let mut tmp = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(code_fname.clone())
        .unwrap();
    tmp.write_all(code.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "-i", &code_fname])
        .output()
        .expect("Failed to execute cargo");

    remove_file(code_fname).unwrap();

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_compiler(compiler_args: Vec<&str>) -> Output {
    let mut args = vec!["run", "--"];
    args.extend(compiler_args);

    let tmp_fname = get_tmp_fname("blkcode");
    let should_create_tmp = args.contains(&"TMP");

    // Replace TMP with temporary file path
    if should_create_tmp {
        for element in &mut args {
            if element == &"TMP" {
                *element = &tmp_fname;
            }
        }

        // Write hello world to the temporary file
        let mut tmp = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(tmp_fname.clone())
            .unwrap();
        tmp.write_all("print(\"Hello, World!\")\n".as_bytes())
            .unwrap();
    }

    let out = Command::new("cargo")
        .args(args.clone())
        .output()
        .expect("Failed to execute cargo");

    // Remove the temporary file
    if should_create_tmp {
        remove_file(tmp_fname).unwrap();
    }

    out
}

fn get_stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone())
        .expect("Failed to get stdout")
        .trim()
        .to_string()
}

fn args(args: &[&str]) -> Vec<String> {
    args.iter().map(|e| e.to_string()).collect()
}

fn get_compiler_res(code: &str) -> Result<(), ErrorType> {
    // Preprocessor
    let code = preprocess(code);

    // Lexer
    let tokens = match lexer(&code) {
        Ok(tokens) => tokens,
        Err(_) => unreachable!(),
    };

    // Parser
    let mut parser = Parser::new(&tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(_) => unreachable!(),
    };

    // Compiler
    let mut compiler = Compiler::from_ast(ast);
    let bin_fname = get_tmp_fname("blkbin");

    compiler.compile(bin_fname.into())
}

fn get_interpreter_res(code: &str) -> Result<(), ErrorType> {
    // Preprocessor
    let code = preprocess(code);

    // Lexer
    let tokens = match lexer(&code) {
        Ok(tokens) => tokens,
        Err(_) => unreachable!(),
    };

    // Parser
    let mut parser = Parser::new(&tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(_) => unreachable!(),
    };

    // Interpreter
    let mut interpreter = Interpreter::from_ast(ast);

    interpreter.run()
}

fn assert_error(result: Result<(), ErrorType>, expected: &ErrorType) {
    match result {
        Err(err) => assert!(err == *expected),
        Ok(_) => panic!("Expected an error, but got Ok"),
    }
}

#[test]
fn print_str() {
    let code = r#"print("hello")"#;
    let expected = "hello";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_int() {
    let code = r#"print(1)"#;
    let expected = "1";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_var_str() {
    let code = r#"
let a = "hello"
print(a)
"#;
    let expected = "hello";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_var_int() {
    let code = r#"
let a = 1
print(a)
"#;
    let expected = "1";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_multiple_args() {
    let code = r#"print("hello", 1)"#;
    let expected = "hello 1";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_bin_expr() {
    let code = r#"
print(1+1)
"#;
    let expected = "2";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_complex_bin_expr() {
    let code = r#"
print(1*2+3)
"#;
    let expected = "5";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_complex_bin_expr_2() {
    let code = r#"
let a = 2*4
let b = a*2
print(1*b/2, a/b, a+b)
"#;
    let expected = "8 0 24";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn print_add_vars() {
    let code = r#"
let a = 1
let b = 1
print(a+b)
"#;
    let expected = "2";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn adding_vars() {
    let code = r#"
let a = 1
let b = 1
let c = a + b
print(c, a + b)
"#;
    let expected = "2 2";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn line_comments() {
    let code = r#"
print("a")
// print("b")
print("c")
"#;
    let expected = "a\nc";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn inline_comments() {
    let code = r#"
print("a") // print("b")
"#;
    let expected = "a";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn variable_redefinition() {
    let code = r#"
let a = 1
print(a)
let a = 2
print(a)
"#;
    let expected = "1\n2";
    assert!(interpret(code) == expected);
    // FIXME
    // assert!(compile_and_run(code) == expected);
}

#[test]
fn example_hello_world() {
    let code = include_str!("../examples/helloworld.blk");
    let expected = "Hello, World!";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn example_example() {
    let code = include_str!("../examples/example.blk");
    let expected = "\
hello, world
hello 123
6
hello, sailor
2
3 3";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

#[test]
fn variable_types() {
    let code = r#"
let int a = 1
let str b = "test"
print(a, b)
"#;
    let expected = "1 test";
    assert!(interpret(code) == expected);
    assert!(compile_and_run(code) == expected);
}

// #[test]
// fn variable_mutability() {
//     let code = r#"
// let a = 1
// print(a)
// a = 2
// print(a)
// "#;
//     let expected = "1\n2";
//     assert!(interpret(code) == expected);
//     assert!(compile_and_run(code) == expected);
// }

#[test]
fn cli_help() {
    let out = run_compiler(vec!["-h"]);
    let stdout = get_stdout(&out);

    assert!(out.status.success());
    assert!(stdout.contains("USAGE"));
}

#[test]
fn cli_version() {
    let out = run_compiler(vec!["-V"]);
    let stdout = get_stdout(&out);

    assert!(out.status.success());
    assert!(stdout.contains("version"));
}

#[test]
fn cli_interpreter() {
    let out = run_compiler(vec!["-i", "TMP"]);
    let stdout = get_stdout(&out);

    assert!(out.status.success());
    assert!(stdout == "Hello, World!");
}

#[test]
fn args_interpreter() {
    let app_args = get_args(args(&["binary", "-i", "input"]));
    assert!(
        app_args
            == AppArgs {
                input: Some(PathBuf::from("input")),
                interpreter: true,
                build_and_run: false,
                output: PathBuf::from("out.app")
            }
    );
}

#[test]
fn args_compiler_out() {
    let app_args = get_args(args(&["binary", "-o", "outfile"]));
    assert!(
        app_args
            == AppArgs {
                input: None,
                interpreter: false,
                build_and_run: false,
                output: PathBuf::from("outfile")
            }
    );
}

#[test]
fn args_build_and_run_out() {
    let app_args = get_args(args(&["binary", "-r", "-o", "outfile"]));
    assert!(
        app_args
            == AppArgs {
                input: None,
                interpreter: false,
                build_and_run: true,
                output: PathBuf::from("outfile")
            }
    );
    let app_args = get_args(args(&["binary", "-o", "outfile", "-r"]));
    assert!(
        app_args
            == AppArgs {
                input: None,
                interpreter: false,
                build_and_run: true,
                output: PathBuf::from("outfile")
            }
    );
}

#[test]
fn err_unknown_func() {
    let code = r#"prnt("test")"#;
    let expected = ErrorType::Generic("Function `prnt` is not implemented".to_string());

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}

#[test]
fn err_variable_doesnt_exist() {
    let code = r#"print(a)"#;
    let expected = ErrorType::SyntaxError("Variable doesn't exist: `a`".to_string());

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}

#[test]
fn err_invalid_print_arg() {
    let code = r#"print(let a = 2)"#;
    let expected = ErrorType::Generic("Invalid argument to print".to_string());

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}

#[test]
fn err_add_not_num() {
    let code = r#"print(1+"")"#;
    let expected = ErrorType::Generic("Cannot add variable which is not a number".to_string());

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}

#[test]
fn err_invalid_expr_type() {
    let code = r#"1"#;
    let expected = ErrorType::Generic(
        "Expression `Number(1)` in this context is not yet implemented".to_string(),
    );

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}

#[test]
fn err_var_type_str_but_not_str() {
    let code = r#"let str a = 1"#;
    let expected = ErrorType::Generic("Variable type `str` does not match value type".to_string());

    assert_error(get_compiler_res(code), &expected);
    assert_error(get_interpreter_res(code), &expected);
}
