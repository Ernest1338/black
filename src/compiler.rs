#![allow(dead_code)]

use crate::{
    args::AppArgs,
    parser::{type_check, Ast, BinExpr, FuncCall, Variable, VariableDeclaration},
    utils::{
        dbg, dbg_file_if_env, dbg_plain, escape_string, get_tmp_fname, measure_time, ErrorType,
    },
    Expr,
};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
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
    pub pk: usize,
    pub variables: HashMap<String, Variable>,
}

impl Compiler {
    /// Creates a new instance of the `Compiler` struct, initializing its fields to default values
    pub fn new() -> Self {
        Self {
            ast: Vec::new(),
            ir: String::new(),
            data: String::new(),
            pk: 0,
            variables: HashMap::new(),
        }
    }

    /// Loads the provided abstract syntax tree (AST) into the compiler, replacing any existing AST
    pub fn load_ast(&mut self, ast: Ast) {
        self.ast = ast;
    }

    /// Creates a new `Compiler` instance from the given AST, initializing necessary fields
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            ir: String::new(),
            data: String::new(),
            pk: 0,
            variables: HashMap::new(),
        }
    }

    /// Retrieves a variable by its identifier, returning it or exiting with an error if not found
    fn get_var(&self, ident: &str) -> Result<Variable, String> {
        self.variables
            .get(ident)
            .cloned()
            .ok_or(format!("Variable doesn't exist: `{ident}`"))
    }

    /// Increments and returns the primary key, used for generating unique variable labels
    fn next_pk(&mut self) -> usize {
        self.pk += 1;
        self.pk
    }

    /// Increments the primary key without returning it
    fn inc_pk(&mut self) {
        self.pk += 1;
    }

    /// Handles a function call by dispatching to the appropriate handler
    fn handle_func_call(&mut self, func_call: &FuncCall) -> Result<(), String> {
        match func_call.name.as_ref() {
            "print" => self.handle_print(func_call)?,
            _ => return Err(format!("Function `{}` is not implemented", func_call.name)),
        }

        Ok(())
    }

    /// Handles the `print` function call by generating IR to print its arguments
    fn handle_print(&mut self, func_call: &FuncCall) -> Result<(), String> {
        let args = func_call.arguments.iter();
        let args_count = args.len();
        for (i, arg) in args.enumerate() {
            let pk = self.next_pk();

            match arg {
                Expr::StringLiteral(message) => {
                    let pk = self.emit_str(message);
                    self.ir.push_str(&format!("  call $printf(l $v{pk})\n"));
                }

                Expr::Number(num) => {
                    let pk = self.emit_str(&num.to_string());
                    self.ir.push_str(&format!("  call $printf(l $v{pk})\n"));
                }

                Expr::BinExpr(bin_expr) => {
                    let res_var = self.handle_bin_expr(bin_expr)?;
                    self.ir
                        .push_str(&format!("  call $printf(l $fmt_int, w {res_var})\n"));
                }

                Expr::Identifier(id) => {
                    let var = self.get_var(id)?;
                    match var {
                        Variable::Number(_) => {
                            // NOTE: here we could grab the number, save it to data section
                            // as a string and print it using puts instead
                            self.ir.push_str(&format!("  %v{pk} =w loadw ${id}\n"));
                            self.ir
                                .push_str(&format!("  call $printf(l $fmt_int, w %v{pk})\n"));
                        }
                        Variable::StringLiteral(_) => {
                            self.ir.push_str(&format!("  call $printf(l ${id})\n"))
                        }
                    }
                }

                _ => {
                    return Err("Invalid argument to print".to_string());
                }
            }

            // Add space between arguments if not the last one
            if i != args_count - 1 {
                self.ir.push_str("  call $printf(l $space)\n");
            }
        }

        self.ir.push_str("  call $printf(l $endl)\n");

        Ok(())
    }

    /// Emits given string to IR data section and returns pk for the variable
    fn emit_str(&mut self, s: &str) -> usize {
        let escaped = escape_string(s);
        let pk = self.next_pk();
        self.data
            .push_str(&format!("data $v{pk} = {{ b \"{escaped}\", b 0 }}\n"));
        pk
    }

    /// Evaluates an operand expression and returns its result temporary variable
    fn eval_operand(&mut self, operand: &Expr) -> Result<String, String> {
        let pk = self.next_pk();
        match operand {
            Expr::Number(n) => Ok(n.to_string()),

            Expr::Identifier(id) => {
                self.ir.push_str(&format!("  %op{pk} =w loadw ${id}\n"));
                Ok(format!("%op{pk}"))
            }

            Expr::BinExpr(bin_expr) => self.handle_bin_expr(bin_expr),

            _ => Err("Cannot add variable which is not a number".to_string()),
        }
    }

    /// Handles a binary expression and generates corresponding IR. Returns temporary variable
    /// containing the equation result
    fn handle_bin_expr(&mut self, bin_expr: &BinExpr) -> Result<String, String> {
        let lhs = self.eval_operand(&bin_expr.lhs)?;
        let rhs = self.eval_operand(&bin_expr.rhs)?;
        let pk = self.next_pk();

        self.ir.push_str(&format!(
            "  %v{pk} =w {} {lhs}, {rhs}\n",
            bin_expr.kind.to_str()
        ));

        Ok(format!("%v{pk}"))
    }

    /// Handles a variable declaration, storing the variable in the `variables` map and generating
    /// corresponding data and IR
    fn handle_var_decl(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), String> {
        let var_label = format!("${}", variable_declaration.identifier);

        if let Some(var_type) = &variable_declaration.typ {
            if !type_check(var_type, &variable_declaration.value) {
                return Err(format!(
                    "Variable type `{var_type}` does not match value type",
                ));
            }
        }

        let value = match &variable_declaration.value {
            Expr::Number(n) => {
                self.data
                    .push_str(&format!("data {var_label} = {{ w {} }}\n", *n));

                Variable::Number(*n)
            }

            Expr::StringLiteral(s) => {
                self.data.push_str(&format!(
                    "data {var_label} = {{ b \"{}\", b 0 }}\n",
                    Variable::StringLiteral(s.to_owned())
                ));

                Variable::StringLiteral(s.to_owned())
            }

            Expr::BinExpr(bin_expr) => {
                let res_var = self.handle_bin_expr(bin_expr)?;
                self.data
                    .push_str(&format!("data {var_label} = {{ w 0 }}\n"));
                self.ir
                    .push_str(&format!("  storew {res_var}, {var_label}\n"));

                Variable::Number(0)
            }

            _ => {
                return Err("Can only store strings and numbers in variables".to_string());
            }
        };

        let id = variable_declaration.identifier.clone();
        self.variables.insert(id, value);

        Ok(())
    }

    /// Generates the intermediate representation (IR) for the AST and returns it as a string
    pub fn generate_ir(&mut self) -> Result<String, ErrorType> {
        self.ir.push_str("export function w $main() {\n@start\n");

        let ast = self.ast.clone();

        for node in &ast {
            match node {
                Expr::FuncCall(func_call) => self.handle_func_call(func_call)?,

                Expr::VariableDeclaration(variable_declaration) => {
                    self.handle_var_decl(variable_declaration)?
                }

                _ => {
                    return Err(ErrorType::Generic(format!(
                        "Expression `{node:?}` in this context is not yet implemented"
                    )));
                }
            }
        }

        self.ir.push_str("  ret 0\n}");

        Ok(format!("{}\n{}", self.data, self.ir))
    }

    /// Compiles the AST by generating IR, running it through the `qbe` compiler, and then
    /// assembling and linking the output with `cc` to produce the final executable
    pub fn compile(&mut self, args: &AppArgs) -> Result<(), ErrorType> {
        let ir = format!("{}{}", include_str!("ext.ssa"), self.generate_ir()?);

        dbg("Variables", &self.variables);
        dbg_plain("Compiled IR", &ir);
        dbg_file_if_env(&ir, "debug.ir", "SAVE_IR");

        let out_file_str = args.output.to_str().expect("invalid output file");

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
        dbg_file_if_env(&qbe_output, "debug.asm", "SAVE_ASM");

        let mut cc_output = String::new();

        let cc_args = if args.static_link {
            vec!["-x", "assembler", "-static", "-o", out_file_str, "-"]
        } else {
            vec!["-x", "assembler", "-o", out_file_str, "-"]
        };

        measure_time("CC execution", || {
            let mut cc = Command::new("cc")
                .args(cc_args)
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

        if !cc_output.is_empty() {
            dbg("WARNING non 0 exit code: CC output", &cc_output);
        }

        // Clean up temporary qbe
        // std::fs::remove_file(&qbe_path).unwrap();

        Ok(())
    }
}
