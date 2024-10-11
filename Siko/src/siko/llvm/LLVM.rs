use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::{
    mir::{
        Data::Class,
        Function::{Function, InstructionKind},
        Program::Program,
        Type::Type,
    },
    qualifiedname::QualifiedName,
};

pub struct Generator {
    output: File,
}

pub fn convertName(name: &QualifiedName) -> String {
    format!("@{}", name.toString().replace(".", "_"))
}

pub fn getStructName(name: &QualifiedName) -> String {
    format!("%struct.{}", name.toString().replace(".", "_"))
}

pub fn getTypeName(ty: &Type) -> String {
    match &ty {
        Type::Void => "void".to_string(),
        Type::I8 => "i8".to_string(),
        Type::I32 => "i32".to_string(),
        Type::Named(n) => getStructName(n),
    }
}

impl Generator {
    pub fn new(outputFile: String) -> Generator {
        Generator {
            output: File::create(outputFile).expect("Failed to open llvm.ll"),
        }
    }

    fn dumpClass(&mut self, c: &Class) -> io::Result<()> {
        let name = getStructName(&c.name);
        write!(self.output, "{} = type {{ ", name)?;
        for (index, field) in c.fields.iter().enumerate() {
            if index == 0 {
                write!(self.output, "{}", getTypeName(&field.ty))?;
            } else {
                write!(self.output, ", {}", getTypeName(&field.ty))?;
            }
        }
        writeln!(self.output, " }}\n")?;
        Ok(())
    }

    fn dumpFunction(&mut self, f: &Function) -> io::Result<()> {
        let name = convertName(&f.name);
        writeln!(self.output, "define void {}() {{", name)?;
        for block in &f.blocks {
            for i in &block.instructions {
                match &i.kind {
                    InstructionKind::Allocate(info) => {
                        writeln!(
                            self.output,
                            "alloca {}, align {}",
                            getTypeName(&info.var.ty),
                            info.var.alignment.alignment
                        )?;
                    }
                    InstructionKind::StoreVar(dest, src) => {
                        writeln!(
                            self.output,
                            "store {} {}, ptr {}, align {}, !dbg !31",
                            getTypeName(&src.ty),
                            src.name,
                            dest.name,
                            dest.alignment.alignment,
                        )?;
                    }
                    InstructionKind::StoreNumeric(dest, value) => {
                        writeln!(
                            self.output,
                            "store {} {}, ptr {}, align {}, !dbg !31",
                            getTypeName(&dest.ty),
                            value,
                            dest.name,
                            dest.alignment.alignment,
                        )?;
                    }
                    InstructionKind::FunctionCall(var, name) => {
                        let name = convertName(name);
                        writeln!(self.output, "call void {}()", name)?;
                    }
                }
            }
        }
        writeln!(self.output, "ret void")?;
        writeln!(self.output, "}}\n")?;
        Ok(())
    }

    pub fn dump(&mut self, program: &Program) -> io::Result<()> {
        for c in &program.classes {
            self.dumpClass(c)?;
        }

        for f in &program.functions {
            self.dumpFunction(f)?;
        }

        writeln!(self.output, "define i32 @main() {{")?;
        writeln!(self.output, "call void @Main_main()")?;
        writeln!(self.output, "ret i32 0")?;
        writeln!(self.output, "}}\n\n")?;
        Ok(())
    }
}
