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
        Type::Ptr(_) => "ptr".to_string(),
        Type::ByteArray(s) => format!("[{} x i8]", s),
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
            Type::ByteArray(_) => 1,
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
                format!("{} = alloca {}, align {}", var.name, getTypeName(&var.ty), self.getAlignment(&var.ty),)
            }
            Instruction::Store(dest, src) => match src {
                Value::Numeric(value, ty) => {
                    format!(
                        "store {} {}, ptr {}, align {}",
                        getTypeName(ty),
                        value,
                        dest.name,
                        self.getAlignment(&dest.ty),
                    )
                }
                Value::String(value, ty) => {
                    format!(
                        "store {} @.{}, ptr {}, align {}",
                        getTypeName(ty),
                        value,
                        dest.name,
                        self.getAlignment(&dest.ty),
                    )
                }
                Value::Variable(src) => {
                    format!(
                        "store {} {}, ptr {}, align {}",
                        getTypeName(&src.ty),
                        src.name,
                        dest.name,
                        self.getAlignment(&dest.ty),
                    )
                }
                Value::Void => unreachable!(),
            },
            Instruction::FunctionCall(name, args) => {
                let mut argRefs = Vec::new();
                for arg in args {
                    argRefs.push(format!("ptr {}", arg.name));
                }
                format!("call void @{}({})", name, argRefs.join(", "))
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
            Instruction::GetFieldRef(dest, root, index) => {
                format!(
                    "{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                    dest.name,
                    getTypeName(&root.ty),
                    root.name,
                    index
                )
            }
            Instruction::Return(value) => match value {
                Value::Void => format!("ret void"),
                Value::Variable(var) => {
                    format!("ret {} {}", getTypeName(&var.ty), var.name)
                }
                Value::String(_, _) => {
                    unreachable!()
                }
                Value::Numeric(v, ty) => {
                    format!("ret {} {}", getTypeName(ty), v)
                }
            },
            Instruction::Jump(label) => {
                format!("br label %{}", label)
            }
            Instruction::Memcpy(src, dest) => match dest.ty.getName() {
                Some(name) => {
                    let def = self.program.getStruct(&name);
                    format!(
                        "call void @llvm.memcpy.p0.p0.i64(ptr align {} {}, ptr align {} {}, i64 {}, i1 false)",
                        def.alignment, dest.name, def.alignment, src.name, def.size
                    )
                }
                None => {
                    format!("ups {:?}", dest.ty)
                }
            },
            Instruction::Bitcast(dest, src) => {
                format!(
                    "{} = bitcast {}* {} to {}*",
                    dest.name,
                    getTypeName(&src.ty),
                    src.name,
                    getTypeName(&dest.ty)
                )
            }
            Instruction::Switch(root, default, branches) => {
                let branches: Vec<_> = branches
                    .iter()
                    .map(|b| match &b.value {
                        Value::Numeric(v, ty) => format!("{} {}, label %{}", getTypeName(&ty), v, b.block),
                        _ => todo!(),
                    })
                    .collect();
                format!(
                    "switch {} {}, label %{} [\n{}\n]\n",
                    getTypeName(&root.ty),
                    root.name,
                    default,
                    branches.join("\n")
                )
            }
        }
    }

    fn dumpFunction(&self, f: &Function, buf: &mut File) -> io::Result<()> {
        let mut args = Vec::new();
        for arg in &f.args {
            args.push(format!("ptr noundef %{}", arg.name,));
        }
        if f.blocks.is_empty() {
            writeln!(buf, "declare {} @{}({})\n", getTypeName(&f.result), f.name, args.join(", "))?;
        } else {
            writeln!(buf, "define private {} @{}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            for block in &f.blocks {
                writeln!(buf, "{}:", block.id)?;
                for i in &block.instructions {
                    let i = self.dumpInstruction(i);
                    writeln!(buf, "   {}", i)?;
                }
            }
            writeln!(buf, "}}\n")?;
        }
        Ok(())
    }

    pub fn dump(&mut self) -> io::Result<()> {
        let mut output = File::create(&self.fileName).expect("Failed to open llvm output");

        for s in &self.program.strings {
            writeln!(
                output,
                "@.{} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
                s.name,
                s.value.len(),
                s.value
            )?;
        }

        for (_, s) in &self.program.structs {
            self.dumpStruct(s, &mut output)?;
        }

        for f in &self.program.functions {
            self.dumpFunction(f, &mut output)?;
        }

        writeln!(output, "define i32 @main() {{")?;
        writeln!(output, "   %res = alloca %struct.siko_Tuple_, align 4")?;
        writeln!(output, "   call void @Main_main(ptr %res)")?;
        writeln!(output, "   ret i32 0")?;
        writeln!(output, "}}\n\n")?;
        Ok(())
    }
}
