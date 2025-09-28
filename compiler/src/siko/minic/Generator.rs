use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Write},
};

use crate::siko::{
    minic::Function::{ExternKind, IntegerOp, Value},
    util::DependencyProcessor::processDependencies,
};

use super::{
    Builtins::dumpBuiltinFunction,
    Data::Struct,
    Function::{Function, GetMode, Instruction},
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
        Type::VoidPtr => "void*".to_string(),
        Type::UInt8 => "uint8_t".to_string(),
        Type::UInt16 => "uint16_t".to_string(),
        Type::UInt32 => "uint32_t".to_string(),
        Type::UInt64 => "uint64_t".to_string(),
        Type::Int8 => "int8_t".to_string(),
        Type::Int16 => "int16_t".to_string(),
        Type::Int32 => "int32_t".to_string(),
        Type::Int64 => "int64_t".to_string(),
        Type::Struct(n) => format!("struct {}", getStructName(n)),
        Type::Ptr(i) => format!("{}*", getTypeName(i)),
        Type::Array(ty, size) => format!("{}[{}]", getTypeName(ty), size),
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
            Type::VoidPtr => 8,
            Type::UInt8 => 1,
            Type::UInt16 => 2,
            Type::UInt32 => 4,
            Type::UInt64 => 8,
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
        writeln!(buf, "// Original name: {}", s.originalName)?;
        writeln!(buf, "struct {} {{", name)?;
        for (index, field) in s.fields.iter().enumerate() {
            if let Type::Array(ty, size) = &field.ty {
                writeln!(buf, "  {} field{}[{}];", getTypeName(&ty), index, size)?;
            } else {
                writeln!(buf, "  {} field{};", getTypeName(&field.ty), index)?;
            }
        }
        writeln!(buf, "}};\n")?;
        Ok(())
    }

    fn dumpInstruction(&self, instruction: &Instruction) -> Option<String> {
        let s = match &instruction {
            Instruction::Declare(_) => return None,
            Instruction::StoreLiteral(dest, src) => match src {
                Value::Numeric(value, _) => {
                    format!("{} = {};", dest.name, value)
                }
                Value::String(value, _) => {
                    format!("{} = (uint8_t*){};", dest.name, value)
                }
            },
            Instruction::FunctionCall(dest, name, args) => {
                let mut argRefs = Vec::new();
                for arg in args {
                    argRefs.push(format!("{}", arg.name));
                }
                match dest {
                    Some(dest) => format!("{} = {}({});", dest.name, name, argRefs.join(", ")),
                    None => format!("{}({});", name, argRefs.join(", ")),
                }
            }
            Instruction::LoadPtr(dest, src) => {
                format!("{} = *{};", dest.name, src.name)
            }
            Instruction::StorePtr(dest, src) => {
                format!("*{} = {};", dest.name, src.name)
            }
            Instruction::GetField(dest, root, index, mode) => {
                let mode = match mode {
                    GetMode::Noop => "",
                    GetMode::Ref => "&",
                };
                if root.ty.isPtr() {
                    format!("{} = {}{}->field{};", dest.name, mode, root.name, index)
                } else {
                    format!("{} = {}{}.field{};", dest.name, mode, root.name, index)
                }
            }
            Instruction::SetField(dest, src, indices) => {
                let path: Vec<_> = indices.iter().map(|i| format!(".field{}", i)).collect();
                let path: String = path.join("");
                format!("{}{} = {};", dest.name, path, src.name)
            }
            Instruction::Return(var) => {
                format!("return {};", var.name)
            }
            Instruction::Jump(label) => {
                format!("goto {};", label)
            }
            Instruction::Memcpy(src, dest) => {
                if !src.ty.isPtr() || !dest.ty.isVoidPtr() {
                    assert_eq!(src.ty, dest.ty);
                }
                format!("{} = {};", dest.name, src.name)
            }
            Instruction::Reference(dest, src) => {
                format!("{} = &{};", dest.name, src.name)
            }
            Instruction::Bitcast(dest, src) => {
                if dest.ty.isPtr() {
                    format!("{} = ({}){};", dest.name, getTypeName(&dest.ty), src.name)
                } else {
                    format!("{} = *({}*)&{};", dest.name, getTypeName(&dest.ty), src.name)
                }
            }
            Instruction::Switch(root, default, branches) => {
                let branches: Vec<_> = branches
                    .iter()
                    .map(|b| match &b.value {
                        Value::Numeric(v, _) => {
                            format!("   case {}:\n      goto {};\n", v, b.block)
                        }
                        _ => todo!(),
                    })
                    .collect();
                let value = if root.ty.isPtr() {
                    format!("*{}", root.name)
                } else {
                    root.name.clone()
                };
                format!(
                    "switch ({}) {{\n{}\n    default:\n       goto {};\n   }}",
                    value,
                    branches.join("\n"),
                    default
                )
            }
            Instruction::AddressOfField(dest, src, index) => {
                if src.ty.isPtr() {
                    format!("{} = &{}->field{};", dest.name, src.name, index)
                } else {
                    format!("{} = &{}.field{};", dest.name, src.name, index)
                }
            }
            Instruction::IntegerOp(dest, left, right, op) => {
                let (opStr, isPtr) = match op {
                    IntegerOp::Add => ("+", false),
                    IntegerOp::Sub => ("-", false),
                    IntegerOp::Mul => ("*", false),
                    IntegerOp::Div => ("/", false),
                    IntegerOp::Mod => ("%", false),
                    IntegerOp::Eq => ("==", true),
                    IntegerOp::LessThan => ("<", true),
                    IntegerOp::ShiftLeft => ("<<", false),
                    IntegerOp::ShiftRight => (">>", false),
                    IntegerOp::BitAnd => ("&", false),
                    IntegerOp::BitOr => ("|", false),
                    IntegerOp::BitXor => ("^", false),
                };
                if isPtr {
                    format!("{} = *{} {} *{};", dest.name, left.name, opStr, right.name)
                } else {
                    format!("{} = {} {} {};", dest.name, left.name, opStr, right.name)
                }
            }
        };
        Some(s)
    }

    fn dumpFunction(&self, f: &Function, buf: &mut File) -> io::Result<()> {
        let mut args = Vec::new();
        let mut argNames = Vec::new();
        for arg in &f.args {
            args.push(format!("{} {}", getTypeName(&arg.ty), arg.name,));
            argNames.push(arg.name.clone());
        }

        let mut localVars = BTreeSet::new();

        for block in &f.blocks {
            for i in &block.instructions {
                match i {
                    Instruction::Declare(v) => {
                        localVars.insert(v.clone());
                    }
                    Instruction::StoreLiteral(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::LoadPtr(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::StorePtr(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Reference(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::FunctionCall(dest, _, _) => {
                        if let Some(dest) = dest {
                            localVars.insert(dest.clone());
                        }
                    }
                    Instruction::Return(_) => {}
                    Instruction::GetField(dest, _, _, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::SetField(dest, _, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Jump(_) => {}
                    Instruction::Memcpy(_, dest) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Bitcast(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Switch(_, _, _) => {}
                    Instruction::AddressOfField(dest, src, _) => {
                        localVars.insert(dest.clone());
                        localVars.insert(src.clone());
                    }
                    Instruction::IntegerOp(dest, left, right, _) => {
                        localVars.insert(dest.clone());
                        localVars.insert(left.clone());
                        localVars.insert(right.clone());
                    }
                }
            }
        }

        if dumpBuiltinFunction(f, &args, buf, &self.program)? {
            return Ok(());
        }

        writeln!(buf, "// Full Name: {}", f.fullName)?;
        if !f.blocks.is_empty() {
            if f.result.isVoid() {
                write!(buf, "[[ noreturn ]] ")?;
            }
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            for v in localVars {
                if argNames.contains(&v.name) {
                    continue;
                }
                if v.ty.isVoid() {
                    continue;
                }
                writeln!(buf, "   {} {};", getTypeName(&v.ty), v.name)?;
            }
            for block in &f.blocks {
                writeln!(buf, "{}:", block.id)?;
                for i in &block.instructions {
                    if let Some(i) = self.dumpInstruction(i) {
                        writeln!(buf, "   {}", i)?;
                    }
                }
            }
            writeln!(buf, "}}\n")?;
        }
        Ok(())
    }

    fn dumpFunctionDeclaration(&self, f: &Function, buf: &mut File) -> io::Result<()> {
        let mut args = Vec::new();
        for arg in &f.args {
            args.push(format!("{} {}", getTypeName(&arg.ty), arg.name,));
        }
        writeln!(buf, "// Full Name: {}", f.fullName)?;
        if f.result.isVoid() {
            write!(buf, "[[ noreturn ]] ")?;
        }
        writeln!(buf, "{} {}({});\n", getTypeName(&f.result), f.name, args.join(", "))?;
        Ok(())
    }

    pub fn dump(&mut self) -> io::Result<()> {
        let mut output = File::create(&self.fileName).expect("Failed to open llvm output");

        let mut headers = BTreeSet::new();
        for f in &self.program.functions {
            match &f.externKind {
                Some(ExternKind::C(info)) => {
                    if let Some(header) = &info.headerName {
                        headers.insert(header.clone());
                    }
                }
                _ => continue,
            }
        }

        writeln!(output, "#include <stdint.h>")?;
        for header in &headers {
            writeln!(output, "#include <{}>", header)?;
        }

        writeln!(output, "")?;

        for s in &self.program.strings {
            writeln!(output, "const char* {} = \"{}\";", s.name, s.value)?;
        }

        writeln!(output, "")?;

        let mut deps = BTreeMap::new();

        for (_, s) in &self.program.structs {
            deps.insert(s.name.clone(), Vec::new());
        }

        for (_, s) in &self.program.structs {
            let deps = deps.entry(s.name.clone()).or_insert_with(|| Vec::new());
            for f in &s.fields {
                match &f.ty {
                    Type::Struct(n) => deps.push(n.clone()),
                    _ => {}
                }
            }
        }

        let groups = processDependencies(&deps);

        for (_, s) in &self.program.structs {
            writeln!(output, "struct {};", s.name)?;
        }

        writeln!(output, "")?;

        for group in groups {
            assert_eq!(group.items.len(), 1);
            for item in group.items {
                let s = self.program.getStruct(&item);
                self.dumpStruct(&s, &mut output)?;
            }
        }

        for f in &self.program.functions {
            if f.isExternC() && f.hasHeaderName() {
                continue;
            }
            self.dumpFunctionDeclaration(f, &mut output)?;
        }

        for f in &self.program.functions {
            if f.isExternC() {
                continue;
            }
            self.dumpFunction(f, &mut output)?;
        }

        writeln!(output, "int main() {{")?;
        writeln!(output, "   Main_main();")?;
        writeln!(output, "   return 0;")?;
        writeln!(output, "}}\n\n")?;
        Ok(())
    }
}
