use std::{
    fs::File,
    io::{self, Write},
};

use crate::siko::qualifiedname::{
    builtins::{
        getIntAddName, getIntCloneName, getIntDivName, getIntEqName, getIntLessThanName, getIntModName, getIntMulName,
        getIntSubName, getIntToI32Name, getIntToU32Name, getIntToU64Name, getIntToU8Name,
        getNativePtrAllocateArrayName, getNativePtrCloneName, getNativePtrDeallocateName, getNativePtrEqName,
        getNativePtrIsNullName, getNativePtrLoadName, getNativePtrMemcmpName, getNativePtrMemcpyName,
        getNativePtrMemmoveName, getNativePtrNullName, getNativePtrOffsetName, getNativePtrPrintName,
        getNativePtrStoreName, getNativePtrToU64Name, getStdBasicUtilAbortName, getStdBasicUtilPrintStrName,
        getStdBasicUtilPrintlnStrName, getU64ToIntName, IntKind,
    },
    QualifiedName,
};

use super::{Function::Function, Generator::getTypeName};

pub fn dumpBuiltinFunction(f: &Function, args: &Vec<String>, buf: &mut File) -> io::Result<bool> {
    if f.name
        .starts_with(&getNativePtrMemcpyName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    memcpy(dest, src, sizeof(*src) * count);")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrMemmoveName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    memmove(dest, src, sizeof(*src) * count);")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrMemcmpName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return memcmp(dest, src, sizeof(*src) * count);")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getNativePtrNullName().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return NULL;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
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
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrDeallocateName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    free(addr);")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrToU64Name().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (uint64_t)addr;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrOffsetName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return &base[count];")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrCloneName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return *addr;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrStoreName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    *addr = item;")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getNativePtrLoadName().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return *addr;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrPrintName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    printf(\"%p\\n\", addr);")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getNativePtrIsNullName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return addr == NULL;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name.starts_with(&getNativePtrEqName().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return a == b;")?;
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

    for kind in [IntKind::Int, IntKind::I32, IntKind::U8, IntKind::U32, IntKind::U64] {
        if isFn(f, &getIntCloneName(kind)) {
            cloneInt(f, args, buf)?;
        }
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

    if f.name.starts_with(&getU64ToIntName().toString().replace(".", "_")) {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    return (int64_t)*self;")?;
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

    if f.name
        .starts_with(&getStdBasicUtilPrintStrName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    printf(\"%.*s\", (int)v->field1, v->field0);")?;
        writeln!(buf, "    return result;")?;
        writeln!(buf, "}}\n")?;
        return Ok(true);
    }

    if f.name
        .starts_with(&getStdBasicUtilPrintlnStrName().toString().replace(".", "_"))
    {
        writeln!(buf, "{} {}({}) {{", getTypeName(&f.result), f.name, args.join(", "))?;
        writeln!(buf, "    {} result;", getTypeName(&f.result))?;
        writeln!(buf, "    printf(\"%.*s\\n\", (int)v->field1, v->field0);")?;
        writeln!(buf, "    return result;")?;
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
