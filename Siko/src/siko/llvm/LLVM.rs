use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::{
    mir::{
        Function::{Function, InstructionKind},
        Program::Program,
    },
    qualifiedname::QualifiedName,
};

pub struct Generator {
    output: File,
}

pub fn getFunctionName(name: &QualifiedName) -> String {
    format!("@{}", name.toString().replace(".", "_"))
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            output: File::create("llvm.ll").expect("Failed to open llvm.ll"),
        }
    }

    fn dumpFunction(&mut self, f: &Function) -> io::Result<()> {
        let name = getFunctionName(&f.name);
        writeln!(self.output, "define void {}() {{", name)?;
        for block in &f.blocks {
            for i in &block.instructions {
                match &i.kind {
                    InstructionKind::FunctionCall(name) => {
                        let name = getFunctionName(name);
                        writeln!(self.output, "call void {}()", name)?;
                    }
                }
            }
        }
        writeln!(self.output, "ret void")?;
        writeln!(self.output, "}}\n\n")?;
        Ok(())
    }

    pub fn dump(&mut self, program: &Program) -> io::Result<()> {
        for f in &program.functions {
            self.dumpFunction(f)?;
        }

        writeln!(self.output, "define void @main() {{")?;
        writeln!(self.output, "call void @Main_main()")?;
        writeln!(self.output, "ret void")?;
        writeln!(self.output, "}}\n\n")?;
        Ok(())
    }
}
