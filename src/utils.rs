use std::{
    env,
    fmt::Debug,
    io::{stdout, Write},
    time::Instant,
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
