use std::{
    env,
    fmt::{Debug, Display},
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

/// Display error to the user in a pretty way
pub fn display_error(err: ErrorType, line: Option<usize>) {
    let line = match line {
        Some(line) => &format!(" on line {line}:"),
        None => "",
    };
    match err {
        ErrorType::SyntaxError(s) => {
            eprintln!("{}{line} {s}", color("[Syntax Error]", Color::LightRed),);
        }
        ErrorType::Generic(s) => {
            eprintln!("{}{line} {s}", color("[Error]", Color::LightRed))
        }
    };
}

/// Display error to the user in a pretty way
pub fn display_error_stdout(err: ErrorType) {
    match err {
        ErrorType::SyntaxError(s) => {
            println!("{} {s}", color("[Syntax Error]", Color::LightRed))
        }
        ErrorType::Generic(s) => {
            println!("{} {s}", color("[Error]", Color::LightRed))
        }
    };
}

/// Escapes backslashes and double quotes in a string for safe inclusion in string literals
pub fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\").replace("\"", "\\\"")
}
