use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::qualifiedname::{
    builtins::{
        getIntAddName, getIntDivName, getIntEqName, getIntLessThanName, getIntModName, getIntMulName, getIntSubName,
        getIntToI32Name, getIntToU32Name, getIntToU64Name, getIntToU8Name, getNativePtrCastName,
        getNativePtrSizeOfName, getNativePtrTransmuteName, getStdBasicUtilAbortName, IntKind,
    },
    QualifiedName,
};

use super::{Function::Function, Generator::getTypeName};

pub fn dumpBuiltinFunction(f: &Function, args: &Vec<String>, buf: &mut File) -> io::Result<bool> {
    if f.name
        .starts_with(&getNativePtrSizeOfName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return sizeof(*addr);")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntAddName(kind)) {
            addInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntSubName(kind)) {
            subInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntMulName(kind)) {
            mulInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntDivName(kind)) {
            divInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntModName(kind)) {
            modInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntEqName(kind)) {
            eqInt(f, args, buf)?;
        }
    }

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntLessThanName(kind)) {
            lessThanInt(f, args, buf)?;
        }
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

    if f.name.starts_with(&getIntToU8Name().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (uint8_t)*self;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getIntToU32Name().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (uint32_t)*self;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getIntToU64Name().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (uint64_t)*self;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getIntToI32Name().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (int32_t)*self;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getStdBasicUtilAbortName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    abort();")?;
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

fn addInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return self + other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn subInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return self - other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn mulInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return self * other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn divInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return self / other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn modInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return self % other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn eqInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return *self == *other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn lessThanInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return *self < *other;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}

fn cloneInt(f: &Function, args: &Vec<String>, buf: &mut File) -> Result<(), io::Error> {
    writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
    writeln!(buf, "    return *self;")?;
    writeln!(buf, "}}\n")?;
    Ok(())
}
