use std::{path::PathBuf, process::exit};

const HELP: &str = "\
Black Lang

\x1b[92mUSAGE\x1b[00m:
  \x1b[33mblack [OPTIONS] <FILE(s)>\x1b[00m

\x1b[92mFLAGS\x1b[00m:
  -i, --interpreter     \x1b[90mUse interpreter instead of compiling to a binary\x1b[00m
  -r, --run             \x1b[90mBuild and run a file\x1b[00m
  -s, --static          \x1b[90mStaticaly link output binary\x1b[00m
  -h, --help            \x1b[90mPrints help information\x1b[00m
  -V, --version         \x1b[90mPrints black version\x1b[00m

\x1b[92mOPTIONS\x1b[00m:
  -o, --output PATH     \x1b[90mSets an output path (default: out.app)\x1b[00m
";

const VERSION: &str = "Black version: \x1b[92mv0.0.1\x1b[00m";

#[derive(Debug, PartialEq)]
pub struct AppArgs {
    // Flags
    pub static_link: bool,
    pub interpreter: bool,
    pub build_and_run: bool,

    // Options
    pub input: Option<PathBuf>,
    pub output: PathBuf,
}

pub fn get_args(args: Vec<String>) -> AppArgs {
    let mut args = args.iter().skip(1); // Skip the program name

    let mut input = None;
    let mut output = PathBuf::from("out.app");
    let mut interpreter = false;
    let mut build_and_run = false;
    let mut static_link = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", HELP);
                exit(0);
            }
            "-V" | "--version" => {
                println!("{}", VERSION);
                exit(0);
            }
            "-i" | "--interpreter" => interpreter = true,
            "-r" | "--run" => build_and_run = true,
            "-s" | "--static" => static_link = true,
            "-o" | "--output" => {
                output = args.next().map(PathBuf::from).unwrap_or_else(|| {
                    eprintln!("Error: Missing output path after -o/--output");
                    exit(1);
                });
            }
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => {
                eprintln!("Error: Unexpected argument '{}'", arg);
                exit(1);
            }
        }
    }

    AppArgs {
        input,
        interpreter,
        build_and_run,
        static_link,
        output,
    }
}
