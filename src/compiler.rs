#![allow(dead_code)]

use crate::{
    parser::{Ast, BinExpr, FuncCall, Type, Variable, VariableDeclaration},
    utils::{dbg, dbg_plain, get_tmp_fname, measure_time, ErrorType},
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
    pub pk: usize,
    pub variables: HashMap<String, Variable>,
}

impl Compiler {
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
    #[allow(dead_code)]
    fn get_var(&self, ident: &str) -> Result<Variable, ErrorType> {
        if self.variables.contains_key(ident) {
            if let Some(s) = self.variables.get(ident) {
                return Ok(s.clone());
            }
        }
        Err(ErrorType::SyntaxError(format!(
            "Variable doesn't exist: `{ident}`"
        )))
    }

    /// Increments and returns the primary key, used for generating unique variable labels
    fn get_pk(&mut self) -> usize {
        self.pk += 1;
        self.pk
    }

    /// Increments the primary key without returning it
    fn inc_pk(&mut self) {
        self.pk += 1;
    }

    /// Handles a function call
    fn handle_func_call(&mut self, func_call: &FuncCall) -> Result<(), ErrorType> {
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
                            self.ir.push_str(&format!("  call $printf(l $v{pk})\n"));
                        }
                        Expr::Number(num) => {
                            let escaped =
                                num.to_string().replace("\\", "\\\\").replace("\"", "\\\"");
                            self.data
                                .push_str(&format!("data $v{pk} = {{ b \"{escaped}\", b 0 }}\n"));
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
                                    self.ir.push_str(&format!(
                                        "  call $printf(l $fmt_int, w %v{pk})\n"
                                    ));
                                }
                                Variable::StringLiteral(_) => {
                                    self.ir.push_str(&format!("  call $printf(l ${id})\n"))
                                }
                            }
                        }
                        _ => {
                            return Err(ErrorType::Generic(
                                "Argument type is not supported".to_string(),
                            ));
                        }
                    }
                    if i != args_count - 1 {
                        self.ir.push_str("  call $printf(l $space)\n");
                    }
                }
                self.ir.push_str("  call $printf(l $endl)\n");
            }
            _ => {
                return Err(ErrorType::Generic(format!(
                    "Function `{}` is not implemented",
                    func_call.name
                )))
            }
        }

        Ok(())
    }

    /// Evaluates an operand expression and returns its result temporary variable
    fn eval_operand(&mut self, operand: &Expr) -> Result<String, ErrorType> {
        let pk = self.get_pk();
        match operand {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::Identifier(id) => {
                self.ir.push_str(&format!("  %op{pk} =w loadw ${id}\n"));
                Ok(format!("%op{pk}"))
            }
            Expr::BinExpr(bin_expr) => self.handle_bin_expr(bin_expr),
            _ => Err(ErrorType::Generic(
                "Cannot add variable which is not a number".to_string(),
            )),
        }
    }

    /// Handles a binary expression and generates corresponding IR. Returns temporary variable
    /// containing the equation result
    fn handle_bin_expr(&mut self, bin_expr: &BinExpr) -> Result<String, ErrorType> {
        let lhs = self.eval_operand(&bin_expr.lhs)?;
        let rhs = self.eval_operand(&bin_expr.rhs)?;
        let pk = self.get_pk();
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
    ) -> Result<(), ErrorType> {
        let var_label = format!("${}", variable_declaration.identifier);
        let value = match &variable_declaration.value {
            Expr::Number(n) => {
                self.data
                    .push_str(&format!("data {var_label} = {{ w {} }}\n", *n));
                Variable::Number(*n)
            }
            Expr::StringLiteral(s) => {
                if variable_declaration.typ.is_some() && variable_declaration.typ != Some(Type::Str)
                {
                    return Err(ErrorType::Generic(
                        "Variable type `str` but value is not a string".to_string(),
                    ));
                }
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
                return Err(ErrorType::Generic(
                    "Can only store strings and numbers in variables".to_string(),
                ));
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
                    return Err(ErrorType::Generic(
                        "This expression type is not yet implemented".to_string(),
                    ));
                }
            }
        }

        self.ir.push_str("  ret 0\n}");

        Ok(format!("{}\n{}", self.data, self.ir))
    }

    /// Compiles the AST by generating IR, running it through the `qbe` compiler, and then
    /// assembling and linking the output with `cc` to produce the final executable
    pub fn compile(&mut self, output_file: PathBuf) -> Result<(), ErrorType> {
        let ir = format!("{}{}", include_str!("ext.ssa"), self.generate_ir()?);

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

        if !cc_output.is_empty() {
            dbg("WARNING non 0 exit code: CC output", &cc_output);
        }

        // Clean up temporary qbe
        // std::fs::remove_file(&qbe_path).unwrap();

        Ok(())
    }
}
