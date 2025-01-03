#![allow(dead_code)]

use crate::{
    parser::{Ast, FuncCall, Variable, VariableDeclaration},
    utils::{dbg, dbg_plain, get_tmp_fname, measure_time},
    Expr,
};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{exit, Command, Stdio},
};

// FIXME: not everywhere qbe is present in the /sbin/qbe path. Adjust accordingly
const QBE_BINARY: &[u8] = &[]; //  include_bytes!("/sbin/qbe");

/// Unpacks QBE from memory into a temporary file. Don't forget to remove the tmp file afterwards
// FIXME: qbe needs to be statically linked. As well as the output black binary
fn get_qbe() -> Result<String, Box<dyn std::error::Error>> {
    // Get a unique temporary file path
    let tmp_path = get_tmp_fname("qbe");

    // Write the embedded QBE binary to the temporary file
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;
        file.write_all(QBE_BINARY)?;
    }

    // Make the file executable (Unix-specific; adjust for Windows)
    #[cfg(unix)]
    {
        let mut permissions = File::open(&tmp_path)?.metadata()?.permissions();
        permissions.set_mode(0o755); // Owner-executable permissions
        File::open(&tmp_path)?.set_permissions(permissions)?;
    }

    Ok(tmp_path)
}

/// Represents a compiler that processes an abstract syntax tree (AST) and generates intermediate
/// representation (IR), as well as handles variable management and function calls
pub struct Compiler {
    pub ast: Ast,
    pub ir: String,
    pub data: String,
    pub primary_key: usize,
    pub variables: HashMap<String, Variable>,
}

impl Compiler {
    /// Creates a new `Compiler` instance from the given AST, initializing necessary fields
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            ir: String::new(),
            data: String::new(),
            primary_key: 0,
            variables: HashMap::new(),
        }
    }

    /// Retrieves a variable by its identifier, returning it or exiting with an error if not found
    #[allow(dead_code)]
    fn get_var(&self, ident: &str) -> Variable {
        if self.variables.contains_key(ident) {
            if let Some(s) = self.variables.get(ident) {
                return s.clone();
            }
        }
        eprintln!("Error: variable doesn't exist: {ident}");
        exit(1); // FIXME
    }

    /// Increments and returns the primary key, used for generating unique variable labels
    fn get_pk(&mut self) -> usize {
        self.primary_key += 1;
        self.primary_key
    }

    /// Handles a function call
    fn handle_func_call(&mut self, func_call: &FuncCall) {
        match func_call.name.as_ref() {
            "print" => {
                let args = func_call.arguments.iter();
                let args_count = args.len();
                for (i, arg) in args.enumerate() {
                    let pk = self.get_pk();

                    match arg {
                        Expr::StringLiteral(message) => {
                            let escaped = message.replace("\\", "\\\\").replace("\"", "\\\"");

                            self.data
                                .push_str(&format!("data $v{pk} = {{ b \"{escaped}\", b 0 }}\n"));
                            self.ir.push_str(&format!("  call $printf(l $v{pk})\n",));
                        }
                        Expr::Number(num) => {
                            let escaped =
                                num.to_string().replace("\\", "\\\\").replace("\"", "\\\"");
                            self.data
                                .push_str(&format!("data $v{pk} = {{ b \"{escaped}\", b 0 }}\n",));
                            self.ir.push_str(&format!("  call $printf(l $v{pk})\n",));
                        }
                        Expr::Identifier(id) => {
                            let var = self.get_var(id);
                            match var {
                                Variable::Number(_) => {
                                    // NOTE: here we could grab the number, save it to data section
                                    // as a string and print it using puts instead
                                    self.ir.push_str(&format!("  %v{pk} =w loadw ${id}\n"));
                                    self.ir.push_str(&format!("  call $print_int(w %v{pk})\n"));
                                }
                                Variable::StringLiteral(_) => {
                                    self.ir.push_str(&format!("  call $printf(l ${id})\n"))
                                }
                            }
                        }
                        _ => unimplemented!("Argument type is not supported"),
                    }
                    if i != args_count - 1 {
                        self.ir.push_str("  call $printf(l $space)\n");
                    }
                }
                self.ir.push_str("  call $printf(l $endl)\n");
            }
            _ => unimplemented!("Function '{}' is not implemented", func_call.name),
        }
    }

    /// Handles a variable declaration, storing the variable in the `variables` map and generating
    /// corresponding data and IR
    fn handle_var_decl(&mut self, variable_declaration: &VariableDeclaration) {
        self.variables.insert(
            variable_declaration.identifier.clone(),
            match &variable_declaration.value {
                Expr::Number(n) => Variable::Number(*n),
                Expr::StringLiteral(s) => Variable::StringLiteral(s.to_owned()),
                // TODO
                // Expr::BinExpr(bin_expr) => Variable::Number(self.handle_bin_expr(bin_expr)),
                _ => {
                    eprintln!("Error: Can only store strings and numbers in variables");
                    exit(1); // FIXME
                }
            },
        );
        let var_label = format!("${}", variable_declaration.identifier);
        match &variable_declaration.value {
            Expr::Number(n) => {
                self.data.push_str(&format!(
                    "data {var_label} = {{ w {} }}\n",
                    Variable::Number(*n)
                ));
            }
            Expr::StringLiteral(s) => {
                self.data.push_str(&format!(
                    "data {var_label} = {{ b \"{}\", b 0 }}\n",
                    Variable::StringLiteral(s.to_owned())
                ));
            }
            // TODO
            // Expr::BinExpr(bin_expr) => Variable::Number(self.handle_bin_expr(bin_expr)),
            _ => {
                eprintln!("Error: Can only store strings and numbers in variables");
                exit(1); // FIXME
            }
        };
    }

    /// Generates the intermediate representation (IR) for the AST and returns it as a string
    pub fn generate_ir(&mut self) -> String {
        self.ir.push_str("export function w $main() {\n@start\n");

        let ast = self.ast.clone();

        for node in &ast {
            match node {
                Expr::FuncCall(func_call) => self.handle_func_call(func_call),
                Expr::VariableDeclaration(variable_declaration) => {
                    self.handle_var_decl(variable_declaration)
                }
                _ => unimplemented!("This expression type is not yet implemented"),
            }
        }

        self.ir.push_str("  ret 0\n}");

        format!("{}\n{}", self.data, self.ir)
    }

    /// Compiles the AST by generating IR, running it through the `qbe` compiler, and then
    /// assembling and linking the output with `cc` to produce the final executable
    pub fn compile(&mut self, output_file: PathBuf) {
        let ir = format!("{}{}", include_str!("ext.ssa"), self.generate_ir());

        dbg("Variables", &self.variables);
        dbg_plain("Compiled IR", &ir);

        let out_file_str = output_file.to_str().expect("invalid output file");

        let mut qbe_output = String::new();

        // Use system installed qbe
        let qbe_path = String::from("qbe");
        // Use qbe included with the compiler
        // let qbe_path = get_qbe().expect("Failed to dynamically unpack QBE. This is a bug.");

        measure_time("QBE execution", || {
            let mut qbe = Command::new(&qbe_path)
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start qbe");

            // Write ir to the qbe process stdin
            if let Some(mut stdin) = qbe.stdin.take() {
                stdin
                    .write_all(ir.as_bytes())
                    .expect("Failed to write to qbe stdin");
            }

            // Get the qbe output assembly
            if let Some(mut stdout) = qbe.stdout.take() {
                stdout
                    .read_to_string(&mut qbe_output)
                    .expect("Failed to read qbe stdout");
            }

            let status = qbe.wait().expect("Failed to wait for qbe process");
            if !status.success() {
                eprintln!("Error: QBE backend error. This is a bug.");
                exit(1);
            }
        });

        dbg("QBE output", &qbe_output);

        let mut cc_output = String::new();

        measure_time("CC execution", || {
            let mut cc = Command::new("cc")
                .args(["-x", "assembler", "-o", out_file_str, "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start cc");

            // Write assembly from qbe to stdin of cc compiler
            if let Some(mut stdin) = cc.stdin.take() {
                stdin
                    .write_all(qbe_output.as_bytes())
                    .expect("Failed to write to qbe stdin");
            }

            // Get the CC output
            if let Some(mut stdout) = cc.stdout.take() {
                stdout
                    .read_to_string(&mut cc_output)
                    .expect("Failed to read cc stdout");
            }

            let status = cc.wait().expect("Failed to wait for cc process");
            if !status.success() {
                eprintln!("Error: CC execution failed. This is a bug.");
                exit(1);
            }
        });

        dbg("CC output", &cc_output);

        // Clean up temporary qbe
        // std::fs::remove_file(&qbe_path).unwrap();
    }
}
