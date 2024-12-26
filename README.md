# <p align="center">⚫️ Black ☠︎︎ ⚫️</p>
Black is a custom programming language implemented in Rust. The project aims to provide a simple yet powerful language with an interactive interpreter mode and a performant compiler based on the QBE backend.

## Features 🌟
- Interpreter: Run Black code without any compilation steps.
- Interactive Mode: Run Black code interactively in a REPL (Read-Eval-Print Loop) environment.
- Compiler: Compile Black source code files to a native binary.

## Installation 🛠️
To install Black, follow these steps:

Clone the repository:
```sh
git clone https://github.com/Ernest1338/black.git
```

Navigate to the project directory:
```sh
cd black
```

Build the project using Cargo:
```sh
cargo build --release
```

## Usage 🚀
### Interactive Mode
To start the interactive mode, run the following command:
```sh
./black -i
```

You will see a prompt where you can enter Black code line by line. Type exit or quit to leave the interactive mode.

### Compiling Source Files
To compile a Black source file, use:
```sh
./black <path_to_source_file>
```
Replace <path_to_source_file> with the path to your Black source file.

## Example 📘
Here's a simple example of using Black in interactive mode:

```rust
>>> let x = 10
  … let y = 20
  … let sum = x + y
  … print("The sum of", x, "and", y, "is", sum)
30
```

## License 📄
This project is licensed under the MIT License. See the LICENSE file for details.
