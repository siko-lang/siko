use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::{
    minic::{Program::Program, Type::Type},
    qualifiedname::{
        builtins::{
            getArrayBaseName, getArrayLenName, getArrayUninitializedName, getNativePtrCastName, getNativePtrSizeOfName,
            getNativePtrTransmuteName,
        },
        QualifiedName,
    },
};

use super::{Function::Function, Generator::getTypeName};

pub fn dumpBuiltinFunction(f: &Function, args: &Vec<String>, buf: &mut File, program: &Program) -> io::Result<bool> {
    if isFn(f, &getArrayUninitializedName()) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} val;", getTypeName(&f.result))?;
        writeln!(buf, "    return val;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if isFn(f, &getArrayLenName()) {
        let arg = f.args.get(0).expect("Array.len without param");
        let s = match &arg.ty.getBase() {
            Type::Struct(s) => program.getStruct(s),
            ty => panic!("Array.len param is not a struct: {}", getTypeName(ty)),
        };
        let len = match s.fields[0].ty {
            Type::Array(_, len) => len,
            _ => panic!("Array.len param field is not array"),
        };
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return {};", len)?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if isFn(f, &getArrayBaseName()) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return self->field0;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if isFn(f, &getNativePtrSizeOfName()) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return sizeof(*addr);")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if isFn(f, &getNativePtrCastName()) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return ({} *)addr;", getTypeName(&f.result))?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if isFn(f, &getNativePtrTransmuteName()) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return ({})v;", getTypeName(&f.result))?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    Ok(false)
}

fn isFn(f: &Function, qn: &QualifiedName) -> bool {
    if f.name.starts_with(&qn.toString().replace(".", "_")) {
        return true;
    }
    false
}
