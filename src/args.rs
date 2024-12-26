use std::{path::PathBuf, process::exit};

/// Help message displayed when the user requests help
const HELP: &str = "\
Black Lang

USAGE:
  black [OPTIONS] <FILE(s)>

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --output PATH         Sets an output path (default: out.app)
  -i, --interpreter     Use interpreter instead of compiling to a binary
";

/// Represents the parsed application arguments
#[derive(Debug)]
pub struct AppArgs {
    pub input: Option<PathBuf>,
    pub interpreter: bool,
    pub output: PathBuf,
}

/// Parses command-line arguments and returns an `AppArgs` struct with the parsed values
fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        exit(0);
    }

    let interpreter = pargs.contains(["-i", "--interpreter"]);

    let args = AppArgs {
        // Parses an optional value from `&OsStr` using a specified function.
        output: pargs
            .opt_value_from_os_str("--output", parse_path)?
            .unwrap_or(PathBuf::from("out.app")),
        interpreter,
        // Parses free-standing/positional argument.
        input: pargs.opt_free_from_str()?,
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

/// Retrieves the application arguments, parsing and handling any errors
pub fn get_args() -> AppArgs {
    match parse_args() {
        Ok(v) => v,
        Err(e) => {
            match e {
                pico_args::Error::MissingArgument => eprintln!("Error: No input files"),
                _ => eprintln!("Error: {}", e),
            }
            exit(0);
        }
    }
}

/// Converts a given `OsStr` to a `PathBuf`
fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}
