use crate::{parser::Ast, ASTNode, State};
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

fn ir_to_bin(ir: &str, output_file: &str) {
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
        .args(["-x", "assembler", "-o", output_file, "-"])
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

pub struct Compiler {
    pub ast: Ast,
    pub state: State,
}

impl Compiler {
    pub fn from_ast(ast: Ast) -> Self {
        Self {
            ast,
            state: State::new(),
        }
    }

    pub fn compile(&self, output_file: PathBuf) -> String {
        // TODO: abstractions
        let mut ir = String::new();
        let mut data_sections = String::new();

        ir.push_str("export function w $main() {\n@start\n");

        for (i, node) in self.ast.iter().enumerate() {
            match node {
                ASTNode::Print(message) => {
                    let data_label = format!("$str{}", i);
                    data_sections.push_str(&format!(
                        "data {} = {{ b \"{}\", b 0 }}\n",
                        data_label,
                        message.replace("\\", "\\\\").replace("\"", "\\\"")
                    ));
                    ir.push_str(&format!("  %r{} =w call $puts(l {})\n", i, data_label));
                }
            }
        }

        ir.push_str("  ret 0\n}");

        let ir = format!("{}\n{}", data_sections, ir);
        let out_file_str = output_file.to_str().expect("invalid output file");
        // ir_to_bin(&ir, out_file_str);

        ir
    }
}
