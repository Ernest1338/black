use crate::{
    parser::{Ast, FuncCall, Variable, VariableDeclaration},
    Expr,
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    process::{exit, Command, Stdio},
};

pub struct Compiler {
    pub ast: Ast,
    pub ir: String,
    pub data_sections: String,
    pub primary_key: usize,
    pub variables: HashMap<String, Variable>,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            ir: String::new(),
            data_sections: String::new(),
            primary_key: 0,
            variables: HashMap::new(),
        }
    }

    fn get_var(&self, ident: &str) -> Variable {
        if self.variables.contains_key(ident) {
            if let Some(s) = self.variables.get(ident) {
                return s.clone();
            }
        }
        eprintln!("Error: variable doesn't exist: {ident}");
        exit(1); // FIXME
    }

    fn handle_func_call(&mut self, func_call: &FuncCall) {
        match func_call.name.as_ref() {
            "print" => {
                let mut concat_msg = String::new();
                for arg in func_call.arguments.iter() {
                    match arg {
                        Expr::StringLiteral(message) => {
                            concat_msg.push_str(message);
                        }
                        Expr::Number(num) => {
                            concat_msg.push_str(&num.to_string());
                        }
                        _ => unimplemented!("Argument type is not supported"),
                    }
                    concat_msg.push(' ');
                }
                concat_msg = concat_msg.trim().to_string();

                let data_label = format!("$str{}", self.primary_key);
                self.data_sections.push_str(&format!(
                    "data {data_label} = {{ b \"{}\", b 0 }}\n",
                    concat_msg.replace("\\", "\\\\").replace("\"", "\\\"")
                ));

                self.ir.push_str(&format!(
                    "  %r{} =w call $puts(l {})\n",
                    self.primary_key, data_label
                ));
            }
            _ => unimplemented!("Function '{}' is not implemented", func_call.name),
        }
    }

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
                self.data_sections.push_str(&format!(
                    "data {var_label} = {{ w {} }}",
                    Variable::Number(*n)
                ));
            }
            Expr::StringLiteral(s) => {
                self.data_sections.push_str(&format!(
                    "data {var_label} = {{ b \"{}\", b 0 }}",
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
            self.primary_key += 1;
        }

        self.ir.push_str("  ret 0\n}");

        format!("{}\n\n{}", self.ir, self.data_sections)
    }

    pub fn compile(&mut self, output_file: PathBuf) {
        let ir = self.generate_ir();
        dbg!(&self.variables);
        println!("compiled:\n{}", ir);

        let out_file_str = output_file.to_str().expect("invalid output file");

        let mut qbe = Command::new("qbe")
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
        let mut qbe_output = String::new();
        if let Some(mut stdout) = qbe.stdout.take() {
            stdout
                .read_to_string(&mut qbe_output)
                .expect("Failed to read qbe stdout");
        }

        qbe.wait().expect("Failed to wait for qbe process");

        // println!("QBE output: {qbe_output}");

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

        let mut cc_output = String::new();
        if let Some(mut stdout) = cc.stdout.take() {
            stdout
                .read_to_string(&mut cc_output)
                .expect("Failed to read cc stdout");
        }

        cc.wait().expect("Failed to wait for cc process");

        // println!("CC output: {cc_output}");
    }
}
