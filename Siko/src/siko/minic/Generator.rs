use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::minic::Function::Value;

use super::{
    Data::Struct,
    Function::{Function, Instruction},
    Program::Program,
    Type::Type,
};

pub struct MiniCGenerator {
    fileName: String,
    program: Program,
}

pub fn getStructName(name: &String) -> String {
    format!("{}", name.replace(".", "_"))
}

pub fn getTypeName(ty: &Type) -> String {
    match &ty {
        Type::Void => "void".to_string(),
        Type::Int8 => "u8_t".to_string(),
        Type::Int16 => "int16_t".to_string(),
        Type::Int32 => "int32_t".to_string(),
        Type::Int64 => "int64_t".to_string(),
        Type::Struct(n) => format!("struct {}", getStructName(n)),
        Type::Ptr(i) => format!("{}*", getTypeName(i)),
        Type::Array(s, itemSize) => format!("int{}_t[{}]", itemSize, s),
    }
}

impl MiniCGenerator {
    pub fn new(outputFile: String, program: Program) -> MiniCGenerator {
        MiniCGenerator {
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
            Type::Array(_, itemSize) => *itemSize / 8,
        }
    }

    fn dumpStruct(&self, s: &Struct, buf: &mut File) -> io::Result<()> {
        let name = getStructName(&s.name);
        writeln!(buf, "struct {} {{", name)?;
        for (index, field) in s.fields.iter().enumerate() {
            if index + 1 == s.fields.len() {
                writeln!(buf, "  {} field{}", getTypeName(&field.ty), index)?;
            } else {
                writeln!(buf, "  {} field{},", getTypeName(&field.ty), index)?;
            }
        }
        writeln!(buf, "}};\n")?;
        Ok(())
    }

    fn dumpInstruction(&self, instruction: &Instruction) -> String {
        match &instruction {
            Instruction::Allocate(var) => {
                format!("{} {};", getTypeName(&var.ty), var.name,)
            }
            Instruction::Store(dest, src) => match src {
                Value::Numeric(value, ty) => {
                    format!(
                        "store volatile {} {}, ptr {}, align {}",
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
                    if arg.ty.isPtr() {
                        argRefs.push(format!("{}", arg.name));
                    } else {
                        argRefs.push(format!("&{}", arg.name));
                    }
                }
                format!("{}({});", name, argRefs.join(", "))
            }
            Instruction::FunctionCallValue(dest, name, args) => {
                let mut argRefs = Vec::new();
                for arg in args {
                    argRefs.push(format!("ptr {}", arg.name));
                }
                format!("{} = call {} @{}({})", dest.name, getTypeName(&dest.ty), name, argRefs.join(", "))
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
                Value::Void => format!("return;"),
                Value::Variable(var) => {
                    format!("return {};", var.name)
                }
                Value::String(_, _) => {
                    unreachable!()
                }
                Value::Numeric(v, _) => {
                    format!("return {};", v)
                }
            },
            Instruction::Jump(label) => {
                format!("br label %{}", label)
            }
            Instruction::Memcpy(src, dest) => match dest.ty.getName() {
                Some(_) => {
                    if dest.ty.isPtr() {
                        format!("*{} = {};", dest.name, src.name)
                    } else {
                        format!("{} = {};", dest.name, src.name)
                    }
                    //let def = self.program.getStruct(&name);
                }
                None => {
                    format!("ups {:?}", dest.ty)
                }
            },
            Instruction::MemcpyPtr(src, dest) => {
                format!(
                    "call void @llvm.memcpy.p0.p0.i64(ptr align {} {}, ptr align {} {}, i64 {}, i1 false)",
                    8, dest.name, 8, src.name, 8
                )
            }
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
            args.push(format!("{}* {}", getTypeName(&arg.ty), arg.name,));
        }
        if !f.blocks.is_empty() {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
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

    fn dumpFunctionDeclaration(&self, f: &Function, buf: &mut File) -> io::Result<()> {
        let mut args = Vec::new();
        for arg in &f.args {
            args.push(format!("{}* {}", getTypeName(&arg.ty), arg.name,));
        }
        writeln!(buf, "{} {}({});\n", getTypeName(&f.result), f.name, args.join(", "))?;
        Ok(())
    }

    pub fn dump(&mut self) -> io::Result<()> {
        let mut output = File::create(&self.fileName).expect("Failed to open llvm output");

        writeln!(output, "#include <siko_runtime.h>")?;
        writeln!(output, "")?;

        for s in &self.program.strings {
            writeln!(output, "const char* {} = \"{}\";", s.name, s.value)?;
        }

        writeln!(output, "")?;

        for (_, s) in &self.program.structs {
            writeln!(output, "struct {};", s.name)?;
        }

        writeln!(output, "")?;

        for (_, s) in &self.program.structs {
            self.dumpStruct(s, &mut output)?;
        }

        for f in &self.program.functions {
            self.dumpFunctionDeclaration(f, &mut output)?;
        }

        for f in &self.program.functions {
            self.dumpFunction(f, &mut output)?;
        }

        writeln!(output, "int main() {{")?;
        writeln!(output, "   struct siko_Tuple_ res;")?;
        writeln!(output, "   Main_main(&res);")?;
        writeln!(output, "   return 0;")?;
        writeln!(output, "}}\n\n")?;
        Ok(())
    }
}
