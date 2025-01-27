use crate::{
    compiler::Compiler,
    parser::{lexer, Parser},
};
use std::{
    env,
    fmt::{Debug, Display},
    fs::OpenOptions,
    io::{stdout, Write},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

/// ANSI color codes for CLI output
#[allow(dead_code)]
pub enum Color {
    Gray,
    Black,
    Red,
    Green,
    Gold,
    Blue,
    Pink,
    Cyan,
    LightRed,
    LightGreen,
    Yellow,
    Purple,
    LightPink,
    LightBlue,
    White,
    Bold,
    Faint,
    Italic,
    Underline,
    Blink,
    Invert,
    Strike,
    Reset,
}

impl Color {
    /// Returns the ANSI escape code for the color
    fn str(&self) -> &str {
        match self {
            Color::Gray => "\x1b[90m",
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Gold => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Pink => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::LightRed => "\x1b[91m",
            Color::LightGreen => "\x1b[92m",
            Color::Yellow => "\x1b[93m",
            Color::Purple => "\x1b[94m",
            Color::LightPink => "\x1b[95m",
            Color::LightBlue => "\x1b[96m",
            Color::White => "\x1b[97m",
            Color::Bold => "\x1b[1m",
            Color::Faint => "\x1b[2m",
            Color::Italic => "\x1b[3m",
            Color::Underline => "\x1b[4m",
            Color::Blink => "\x1b[5m",
            Color::Invert => "\x1b[7m",
            Color::Strike => "\x1b[9m",
            Color::Reset => "\x1b[00m",
        }
    }
}

/// Get temporary directory. Either /tmp or specified by the user using TMPDIR env var
pub fn get_tmp_dir() -> String {
    env::var("TMPDIR").unwrap_or("/tmp".to_string())
}

/// Get temporary file path. Uses get_tmp_dir for the base file path
pub fn get_tmp_fname(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos(); // Use nanoseconds for more uniqueness
    let tmp_dir = get_tmp_dir();
    format!("{tmp_dir}/{prefix}_{timestamp}")
}

/// Wraps a string with the given ANSI color code
pub fn color(string: &str, color: Color) -> String {
    // Possible to add env var to disable coloring by just adding one if statement
    // to check if that var is set and, if yes, return the original string
    format!("{}{string}{}", color.str(), Color::Reset.str())
}

/// Prints a message to the console and flushes the output
pub fn print_and_flush(m: &str) {
    print!("{m}");
    stdout().flush().unwrap();
}

/// Debug-prints a value with a label if the DEBUG environment variable is set
pub fn dbg<T: Debug>(label: &str, value: &T) {
    // TODO: debug levels (DEBUG_LEVEL env var)
    // TODO: getter function for precached values for DEBUG and DEBUG_LEVEL vars
    // and usage in both `dbg` and `dbg_pretty` and `dbg_plain`
    // NOTE: maybe a debug printer object?
    if env::var("DEBUG").is_ok() {
        println!(
            "{} {}: {value:?}",
            color("[DEBUG]", Color::Gray),
            color(label, Color::LightPink)
        );
    }
}

/// Plain-prints a value with a label if the DEBUG environment variable is set
pub fn dbg_plain<T: Display>(label: &str, value: &T) {
    if env::var("DEBUG").is_ok() {
        println!(
            "{} {}: {value}",
            color("[DEBUG]", Color::Gray),
            color(label, Color::LightPink)
        );
    }
}

/// Debug-pretty prints a value with a label if the DEBUG environment variable is set
pub fn dbg_pretty<T: Debug>(label: &str, value: &T) {
    if env::var("DEBUG").is_ok() {
        println!("{} {label}: {value:#?}", color("[DEBUG]", Color::Gray));
    }
}

/// Measures the execution time of a function and prints it if DEBUG is set
pub fn measure_time<T, F: FnOnce() -> T>(label: &str, f: F) -> T {
    if env::var("DEBUG").is_ok() {
        let start = Instant::now();
        let result = f();
        let duration = format!("{:?}", start.elapsed());
        println!(
            "{} {}  {label} took: {}",
            color("[DEBUG]", Color::Gray),
            color("⏱", Color::Gold),
            color(&duration, Color::LightGreen)
        );
        result
    } else {
        f()
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    SyntaxError(String),
    Generic(String),
}

fn get_line_nr_str(line_nr: Option<usize>) -> String {
    match line_nr {
        Some(line_nr) => format!(" on line {line_nr}:"),
        None => "".to_string(),
    }
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Stdout,
    Stderr,
}

/// Display error to the user in a pretty way
pub fn display_error(err: ErrorType, src: &str, target: Output) {
    let output_fn = match target {
        Output::Stdout => |msg| println!("{msg}"),
        Output::Stderr => |msg| eprintln!("{msg}"),
    };

    match err {
        ErrorType::SyntaxError(message) => {
            output_fn(&format!(
                "{}{} {}",
                color("[Syntax Error]", Color::LightRed),
                get_line_nr_str(find_error_line_number(src)),
                message
            ));
        }
        ErrorType::Generic(message) => {
            output_fn(&format!(
                "{}{} {}",
                color("[Error]", Color::LightRed),
                get_line_nr_str(find_error_line_number(src)),
                message
            ));
        }
    };
}

/// Escapes backslashes and double quotes in a string for safe inclusion in string literals
pub fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\").replace("\"", "\\\"")
}

impl From<String> for ErrorType {
    /// Converts a String message into an ErrorType::Generic variant
    fn from(message: String) -> Self {
        ErrorType::Generic(message)
    }
}

/// Writes data to a file if the given environment variable is set
pub fn dbg_file_if_env(data: &str, file: &str, var: &str) {
    if env::var(var).is_ok() {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file)
            .unwrap()
            .write_all(data.as_bytes())
            .unwrap();
    }
}

/// Finds the line number where a syntax error occurs in the given source code
pub fn find_error_line_number(source: &str) -> Option<usize> {
    let mut current_line = 1;
    let mut context = String::new();
    let mut compiler = Compiler::new();

    // Iterate through the source line by line
    for line in source.lines() {
        // Handle line comments and empty lines
        if line.starts_with("//") || line.is_empty() {
            current_line += 1;
            continue;
        }

        // Handle inline comments
        let line = line.split("//").collect::<Vec<&str>>()[0];

        // Append context
        context.push_str(&format!("{line}\n"));

        // Tokenize and parse the context
        let context_tokens = match lexer(&context) {
            Ok(tokens) => tokens,
            Err(_) => return Some(current_line), // Return current line if lexer fails
        };

        let mut parser = Parser::new(&context_tokens);
        let ast = match parser.parse() {
            Ok(ast) => {
                context.clear();
                ast
            }
            Err(_) => {
                // Return the current line where parsing fails
                return Some(current_line);
            }
        };

        compiler.load_ast(ast);
        if let Err(_) = compiler.generate_ir() {
            return Some(current_line);
        }

        // Increment current line counter
        current_line += 1;
    }

    None // Return None if no error line is found
}
