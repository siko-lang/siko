use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::llvm::Function::Value;

use super::{
    Data::Struct,
    Function::{Function, Instruction},
    Program::Program,
    Type::Type,
};

pub struct Generator {
    fileName: String,
    program: Program,
}

pub fn getStructName(name: &String) -> String {
    format!("%struct.{}", name.replace(".", "_"))
}

pub fn getTypeName(ty: &Type) -> String {
    match &ty {
        Type::Void => "void".to_string(),
        Type::Int8 => "i8".to_string(),
        Type::Int16 => "i16".to_string(),
        Type::Int32 => "i32".to_string(),
        Type::Int64 => "i64".to_string(),
        Type::Struct(n) => getStructName(n),
        Type::Ptr(_) => todo!(),
    }
}

impl Generator {
    pub fn new(outputFile: String, program: Program) -> Generator {
        Generator {
            fileName: outputFile,
            program: program,
        }
    }

    fn getAlignment(&self, ty: &Type) -> u32 {
        match &ty {
            Type::Void => 0,
            Type::Int8 => 1,
            Type::Int16 => 2,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::Struct(n) => self.program.getStruct(n).alignment,
            Type::Ptr(_) => 8,
        }
    }

    fn dumpStruct(&self, s: &Struct, buf: &mut File) -> io::Result<()> {
        let name = getStructName(&s.name);
        write!(buf, "{} = type {{ ", name)?;
        for (index, field) in s.fields.iter().enumerate() {
            if index == 0 {
                write!(buf, "{}", getTypeName(&field.ty))?;
            } else {
                write!(buf, ", {}", getTypeName(&field.ty))?;
            }
        }
        writeln!(buf, " }}\n")?;
        Ok(())
    }

    fn dumpInstruction(&self, instruction: &Instruction) -> String {
        match &instruction {
            Instruction::Allocate(var) => {
                format!(
                    "{} = alloca {}, align {}",
                    var.name,
                    getTypeName(&var.ty),
                    self.getAlignment(&var.ty),
                )
            }
            Instruction::Store(dest, src) => match src {
                Value::Void => unreachable!(),
                Value::Numeric(value) => {
                    format!(
                        "store {} {}, ptr {}, align {}, !dbg !31",
                        getTypeName(&dest.ty),
                        value,
                        dest.name,
                        self.getAlignment(&dest.ty),
                    )
                }
                Value::Variable(src) => {
                    format!(
                        "store {} {}, ptr {}, align {}, !dbg !31",
                        getTypeName(&src.ty),
                        src.name,
                        dest.name,
                        self.getAlignment(&dest.ty),
                    )
                }
            },
            Instruction::FunctionCall(res, name, args) => {
                let mut argRefs = Vec::new();
                for arg in args {
                    argRefs.push(arg.name.clone());
                }
                if res.ty == Type::Void {
                    format!(
                        "call {} {}({})",
                        getTypeName(&res.ty),
                        name,
                        argRefs.join(", ")
                    )
                } else {
                    format!(
                        "{} = call {} {}({})",
                        res.name,
                        getTypeName(&res.ty),
                        name,
                        argRefs.join(", ")
                    )
                }
            }
            Instruction::LoadVar(dest, src) => {
                format!(
                    "{} = load {}, ptr {}, align {}",
                    dest.name,
                    getTypeName(&dest.ty),
                    src.name,
                    self.getAlignment(&src.ty),
                )
            }
            Instruction::Return(value) => match value {
                Value::Void => format!("ret void"),
                Value::Variable(var) => {
                    format!("ret {} {}", getTypeName(&var.ty), var.name)
                }
                Value::Numeric(v) => {
                    format!("ret i64 {}", v)
                }
            },
        }
    }

    fn dumpFunction(&self, f: &Function, buf: &mut File) -> io::Result<()> {
        let mut args = Vec::new();
        for arg in &f.args {
            match &arg.ty {
                Type::Struct(name) => {
                    let s = self.program.getStruct(name);
                    args.push(format!(
                        "ptr noundef byval({}) align {} {}",
                        getStructName(name),
                        s.alignment,
                        arg.name,
                    ));
                }
                Type::Int64 => {
                    args.push(format!("i64 {}", arg.name));
                }
                _ => todo!(),
            }
        }
        writeln!(
            buf,
            "define {} {}({}) {{",
            getTypeName(&f.result),
            f.name,
            args.join(", ")
        )?;
        for block in &f.blocks {
            for i in &block.instructions {
                let i = self.dumpInstruction(i);
                writeln!(buf, "   {}", i)?;
            }
        }
        writeln!(buf, "}}\n")?;
        Ok(())
    }

    pub fn dump(&mut self) -> io::Result<()> {
        let mut output = File::create(&self.fileName).expect("Failed to open llvm output");
        for (_, s) in &self.program.structs {
            self.dumpStruct(s, &mut output)?;
        }

        for f in &self.program.functions {
            self.dumpFunction(f, &mut output)?;
        }

        writeln!(output, "define i32 @main() {{")?;
        writeln!(output, "   call void @Main_main()")?;
        writeln!(output, "   ret i32 0")?;
        writeln!(output, "}}\n\n")?;
        Ok(())
    }
}
