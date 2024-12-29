#![allow(dead_code)]

use std::{
    fs::{remove_file, OpenOptions},
    io::Write,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn get_tmp_fname(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos(); // Use nanoseconds for more uniqueness
    format!("/tmp/{prefix}_{}.blk", timestamp)
}

fn compile(code: &str) -> String {
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

#[test]
fn print_str() {
    let code = r#"print("hello")"#;
    let expected = "hello";
    assert!(interpret(code) == expected);
    assert!(compile(code) == expected);
}

#[test]
fn print_int() {
    let code = r#"print(1)"#;
    let expected = "1";
    assert!(interpret(code) == expected);
    assert!(compile(code) == expected);
}

#[test]
fn print_var_str() {
    let code = r#"
let a = "hello"
print(a)
"#;
    let expected = "hello";
    assert!(interpret(code) == expected);
    assert!(compile(code) == expected);
}

#[test]
fn print_var_int() {
    let code = r#"
let a = 1
print(a)
"#;
    let expected = "1";
    assert!(interpret(code) == expected);
    assert!(compile(code) == expected);
}

#[test]
fn print_multiple_args() {
    let code = r#"print("hello", 1)"#;
    let expected = "hello 1";
    assert!(interpret(code) == expected);
    // FIXME
    // assert!(compile(code) == expected);
}

#[test]
fn print_bin_expr() {
    let code = r#"
print(1+1)
"#;
    let expected = "2";
    assert!(interpret(code) == expected);
    // FIXME
    // assert!(compile(code) == expected);
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
    // FIXME
    // assert!(compile(code) == expected);
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
    // FIXME
    // assert!(compile(code) == expected);
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
    assert!(compile(code) == expected);
}

#[test]
fn inline_comments() {
    let code = r#"
print("a") // print("b")
"#;
    let expected = "a";
    assert!(interpret(code) == expected);
    assert!(compile(code) == expected);
}
