use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Write},
};

use crate::siko::{
    minic::Function::Value,
    qualifiedname::builtins::{
        getIntAddName, getIntCloneName, getIntDivName, getIntEqName, getIntLessThanName, getIntModName, getIntMulName,
        getIntSubName, getIntToU8Name, getNativePtrAllocateArrayName, getNativePtrCloneName,
        getNativePtrDeallocateName, getNativePtrEqName, getNativePtrIsNullName, getNativePtrLoadName,
        getNativePtrMemcmpName, getNativePtrMemcpyName, getNativePtrNullName, getNativePtrOffsetName,
        getNativePtrPrintName, getNativePtrStoreName, getStdBasicUtilAbortName, getStdBasicUtilPrintStrName,
        getStdBasicUtilPrintlnStrName, getU8AddName, getU8CloneName, getU8DivName, getU8EqName, getU8LessThanName,
        getU8MulName, getU8SubName,
    },
    util::DependencyProcessor::processDependencies,
};

use super::{
    Data::Struct,
    Function::{Function, GetMode, Instruction, ReadMode},
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
        Type::UInt8 => "uint8_t".to_string(),
        Type::Int16 => "int16_t".to_string(),
        Type::Int32 => "int32_t".to_string(),
        Type::Int64 => "int64_t".to_string(),
        Type::Struct(n) => format!("struct {}", getStructName(n)),
        Type::Ptr(i) => format!("{}*", getTypeName(i)),
        Type::Array(_, itemSize) => format!("int{}_t", itemSize),
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
            Type::UInt8 => 1,
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
            if field.ty.isArray() {
                writeln!(
                    buf,
                    "  {} field{}[{}];",
                    getTypeName(&field.ty),
                    index,
                    field.ty.getArraySize()
                )?;
            } else {
                writeln!(buf, "  {} field{};", getTypeName(&field.ty), index)?;
            }
        }
        writeln!(buf, "}};\n")?;
        Ok(())
    }

    fn dumpInstruction(&self, instruction: &Instruction) -> Option<String> {
        let s = match &instruction {
            Instruction::Allocate(_) => return None,
            Instruction::Store(dest, src) => match src {
                Value::Numeric(value, _) => {
                    format!("{} = {};", dest.name, value)
                }
                Value::String(value, _) => {
                    format!("{} = (uint8_t*){};", dest.name, value)
                }
                Value::Variable(src) => {
                    format!("{} = {};", dest.name, src.name,)
                }
                Value::Void => unreachable!(),
            },
            Instruction::FunctionCallValue(dest, name, args) => {
                let mut argRefs = Vec::new();
                for arg in args {
                    argRefs.push(format!("{}", arg.name));
                }
                if dest.ty.isVoid() {
                    format!("{}({});", name, argRefs.join(", "))
                } else {
                    format!("{} = {}({});", dest.name, name, argRefs.join(", "))
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
            Instruction::GetField(dest, root, index, mode) => {
                let mode = match mode {
                    GetMode::Noop => "",
                    GetMode::Ref => "&",
                    GetMode::Deref => "*",
                };
                if root.ty.isPtr() {
                    format!("{} = {}{}->field{};", dest.name, mode, root.name, index)
                } else {
                    format!("{} = {}{}.field{};", dest.name, mode, root.name, index)
                }
            }
            Instruction::SetField(dest, src, indices, mode) => {
                let path: Vec<_> = indices.iter().map(|i| format!(".field{}", i)).collect();
                let path: String = path.join("");
                match *mode {
                    ReadMode::Noop => {
                        format!("{}{} = {};", dest.name, path, src.name)
                    }
                    ReadMode::Ref => {
                        format!("{}{} = *{};", dest.name, path, src.name)
                    }
                    ReadMode::Deref => {
                        format!("{}{} = *{};", dest.name, path, src.name)
                    }
                }
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
                format!("goto {};", label)
            }
            Instruction::Memcpy(src, dest) => match &dest.ty {
                ty if ty.getName().is_some() => {
                    if dest.ty.isPtr() {
                        if src.ty.isPtr() {
                            format!("*{} = *({}){};", dest.name, getTypeName(&dest.ty), src.name)
                        } else {
                            format!("*{} = *({})&{};", dest.name, getTypeName(&dest.ty), src.name)
                        }
                    } else {
                        format!("{} = {};", dest.name, src.name)
                    }
                    //let def = self.program.getStruct(&name);
                }
                Type::Int64 => {
                    format!("{} = {};", dest.name, src.name)
                }
                Type::UInt8 => {
                    format!("{} = {};", dest.name, src.name)
                }
                _ => format!("INTERNALERROR"),
            },
            Instruction::MemcpyPtr(src, dest) => {
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
                        Value::Numeric(v, _) => format!("   case {}:\n      goto {};\n", v, b.block),
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
                    Instruction::Allocate(v) => {
                        localVars.insert(v.clone());
                    }
                    Instruction::Store(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::LoadVar(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Reference(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::FunctionCallValue(dest, _, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Return(_) => {}
                    Instruction::GetField(dest, _, _, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::SetField(dest, _, _, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Jump(_) => {}
                    Instruction::Memcpy(_, dest) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::MemcpyPtr(_, dest) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Bitcast(dest, _) => {
                        localVars.insert(dest.clone());
                    }
                    Instruction::Switch(_, _, _) => {}
                }
            }
        }

        if f.name
            .starts_with(&getNativePtrMemcpyName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    memcpy(dest, src, sizeof(*src) * count);")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrMemcmpName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return memcmp(dest, src, sizeof(*src) * count);")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getNativePtrNullName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return NULL;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrAllocateArrayName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(
                buf,
                "    return malloc(sizeof({}) * count);",
                getTypeName(&f.result.getBase())
            )?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrDeallocateName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    free(addr);")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrOffsetName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return &base[count];")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrCloneName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *addr;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrStoreName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    *addr = item;")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getNativePtrLoadName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *addr;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrPrintName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    printf(\"%p\\n\", addr);")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getNativePtrIsNullName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return addr == NULL;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getNativePtrEqName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return a == b;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntAddName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self + other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntSubName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self - other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntMulName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self * other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntDivName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self / other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntModName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self % other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntToU8Name().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return (uint8_t)*self;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntEqName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self == *other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntLessThanName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self < *other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getIntCloneName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8AddName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self + other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8SubName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self - other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8MulName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self * other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8DivName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return self / other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8EqName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self == *other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8LessThanName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self < *other;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name.starts_with(&getU8CloneName().toString().replace(".", "_")) {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    return *self;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getStdBasicUtilAbortName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    abort();")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getStdBasicUtilPrintStrName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    printf(\"%.*s\", (int)v->field1, v->field0);")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

        if f.name
            .starts_with(&getStdBasicUtilPrintlnStrName().toString().replace(".", "_"))
        {
            writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
            writeln!(buf, "    {} result;", getTypeName(&f.result))?;
            writeln!(buf, "    printf(\"%.*s\\n\", (int)v->field1, v->field0);")?;
            writeln!(buf, "    return result;")?;
            writeln!(buf, "}}\n")?;
        }

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
        if f.result.isVoid() {
            write!(buf, "[[ noreturn ]] ")?;
        }
        writeln!(buf, "{} {}({});\n", getTypeName(&f.result), f.name, args.join(", "))?;
        Ok(())
    }

    pub fn dump(&mut self) -> io::Result<()> {
        let mut output = File::create(&self.fileName).expect("Failed to open llvm output");

        writeln!(output, "#include <stdlib.h>")?;
        writeln!(output, "#include <stdint.h>")?;
        writeln!(output, "#include <string.h>")?;
        writeln!(output, "#include <stdio.h>")?;
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
            self.dumpFunctionDeclaration(f, &mut output)?;
        }

        for f in &self.program.functions {
            self.dumpFunction(f, &mut output)?;
        }

        writeln!(output, "int main() {{")?;
        writeln!(output, "   Main_main_();")?;
        writeln!(output, "   return 0;")?;
        writeln!(output, "}}\n\n")?;
        Ok(())
    }
}
