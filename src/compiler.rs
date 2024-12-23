use crate::{
    parser::{Ast, FuncCall},
    Expr,
};
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

pub struct Compiler {
    pub ast: Ast,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self { ast }
    }

    fn handle_func_call(
        &self,
        func_call: &FuncCall,
        ir: &mut String,
        data_sections: &mut String,
        index: usize,
    ) {
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

                let data_label = format!("$str{}", index);
                data_sections.push_str(&format!(
                    "data {} = {{ b \"{}\", b 0 }}\n",
                    data_label,
                    concat_msg
                        .replace("\\", "\\\\")
                        .replace("\"", "\\\"")
                ));

                ir.push_str(&format!("  %r{} =w call $puts(l {})\n", index, data_label));
            }
            _ => unimplemented!("Function '{}' is not implemented", func_call.name),
        }
    }

    pub fn generate_ir(&self) -> String {
        let mut ir = String::new();
        let mut data_sections = String::new();

        ir.push_str("export function w $main() {\n@start\n");

        for (i, node) in self.ast.iter().enumerate() {
            match node {
                Expr::FuncCall(func_call) => {
                    self.handle_func_call(func_call, &mut ir, &mut data_sections, i)
                }
                _ => unimplemented!("This expression type is not yet implemented"),
            }
        }

        ir.push_str("  ret 0\n}");

        format!("{}\n{}", data_sections, ir)
    }

    pub fn compile(&self, output_file: PathBuf) {
        let ir = self.generate_ir();
        // println!("compiled:\n{}", ir);

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
