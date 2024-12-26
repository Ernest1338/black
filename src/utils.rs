use std::{
    env,
    fmt::Debug,
    io::{stdout, Write},
    time::Instant,
};

/// ANSI color codes for CLI output
pub enum Color {
    Gray,
    Reset,
}

impl Color {
    /// Returns the ANSI escape code for the color
    fn str(&self) -> &str {
        match self {
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[00m",
        }
    }
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
    // and usage in both `dbg` and `dbg_pretty`
    // NOTE: maybe a debug printer object?
    if env::var("DEBUG").is_ok() {
        println!("{} {label}: {value:?}", color("[DEBUG]", Color::Gray));
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
        let duration = start.elapsed();
        dbg(&format!("⏱  {label} took"), &duration);
        result
    } else {
        f()
    }
}
