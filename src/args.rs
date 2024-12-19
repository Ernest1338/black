use std::{path::PathBuf, process::exit};

const HELP: &str = "\
Black Lang

USAGE:
  black [OPTIONS] <FILE(s)>

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --output PATH         Sets an output path (default: out.app)
";

#[derive(Debug)]
pub struct AppArgs {
    pub input: PathBuf,
    pub output: PathBuf,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        // Parses an optional value from `&OsStr` using a specified function.
        output: pargs
            .opt_value_from_os_str("--output", parse_path)?
            .unwrap_or(PathBuf::from("out.app")),
        // Parses a required free-standing/positional argument.
        input: pargs.free_from_str()?,
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

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

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}
