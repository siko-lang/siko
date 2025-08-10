use crate::siko::qualifiedname::{build, QualifiedName};

pub fn getBoolTypeName() -> QualifiedName {
    build("Bool", "Bool")
}

pub fn getIntTypeName() -> QualifiedName {
    build("Int", "Int")
}

pub fn getU8TypeName() -> QualifiedName {
    build("Int", "U8")
}

pub fn getI32TypeName() -> QualifiedName {
    build("Int", "I32")
}

pub fn getStringTypeName() -> QualifiedName {
    build("String", "String")
}

pub fn getStringLiteralTypeName() -> QualifiedName {
    build("String", "StringLiteral")
}

pub fn getCharTypeName() -> QualifiedName {
    build("Char", "Char")
}

pub fn getTrueName() -> QualifiedName {
    build("Bool", "Bool").add("True".to_string())
}

pub fn getFalseName() -> QualifiedName {
    build("Bool", "Bool").add("False".to_string())
}

pub fn getStringEqName() -> QualifiedName {
    build("String", "String").add("eq".to_string())
}

pub fn getNativePtrNullName() -> QualifiedName {
    build("NativePtr", "null")
}

pub fn getNativePtrAllocateArrayName() -> QualifiedName {
    build("NativePtr", "allocateArray")
}

pub fn getNativePtrDeallocateName() -> QualifiedName {
    build("NativePtr", "deallocate")
}

pub fn getNativePtrMemcpyName() -> QualifiedName {
    build("NativePtr", "memcpy")
}

pub fn getNativePtrMemcmpName() -> QualifiedName {
    build("NativePtr", "memcmp")
}

pub fn getNativePtrOffsetName() -> QualifiedName {
    build("NativePtr", "offset")
}

pub fn getNativePtrStoreName() -> QualifiedName {
    build("NativePtr", "store")
}

pub fn getNativePtrToRefName() -> QualifiedName {
    build("NativePtr", "toRef")
}

pub fn getNativePtrPrintName() -> QualifiedName {
    build("NativePtr", "print")
}

pub fn getNativePtrCloneName() -> QualifiedName {
    build("NativePtr", "clone")
}

pub fn getNativePtrLoadName() -> QualifiedName {
    build("NativePtr", "load")
}

pub fn getNativePtrIsNullName() -> QualifiedName {
    build("NativePtr", "isNull")
}

pub fn getNativePtrEqName() -> QualifiedName {
    build("NativePtr", "eq")
}

pub fn getCloneFnName() -> QualifiedName {
    build("Std.Ops", "Clone").add(format!("clone"))
}

pub fn getIntAddName() -> QualifiedName {
    build("Int", "Int").add(format!("add"))
}

pub fn getIntSubName() -> QualifiedName {
    build("Int", "Int").add(format!("sub"))
}

pub fn getIntMulName() -> QualifiedName {
    build("Int", "Int").add(format!("mul"))
}

pub fn getIntDivName() -> QualifiedName {
    build("Int", "Int").add(format!("div"))
}

pub fn getIntModName() -> QualifiedName {
    build("Int", "Int").add(format!("mod"))
}

pub fn getIntToU8Name() -> QualifiedName {
    build("Int", "Int").add(format!("toU8"))
}

pub fn getIntEqName() -> QualifiedName {
    build("Int", "Int").add(format!("eq"))
}

pub fn getIntLessThanName() -> QualifiedName {
    build("Int", "Int").add(format!("lessThan"))
}

pub fn getIntCloneName() -> QualifiedName {
    build("Int", "Int").add(format!("clone"))
}

pub fn getU8AddName() -> QualifiedName {
    build("Int", "U8").add(format!("add"))
}

pub fn getU8SubName() -> QualifiedName {
    build("Int", "U8").add(format!("sub"))
}

pub fn getU8MulName() -> QualifiedName {
    build("Int", "U8").add(format!("mul"))
}

pub fn getU8DivName() -> QualifiedName {
    build("Int", "U8").add(format!("div"))
}

pub fn getU8EqName() -> QualifiedName {
    build("Int", "U8").add(format!("eq"))
}

pub fn getU8LessThanName() -> QualifiedName {
    build("Int", "U8").add(format!("lessThan"))
}

pub fn getU8CloneName() -> QualifiedName {
    build("Int", "U8").add(format!("clone"))
}

pub fn getDropFnName() -> QualifiedName {
    build("Std.Ops", "Drop").add(format!("drop"))
}

pub fn getDropName() -> QualifiedName {
    build("Std.Ops", "Drop")
}

pub fn getCopyName() -> QualifiedName {
    build("Std.Ops", "Copy")
}

pub fn getDerefName() -> QualifiedName {
    build("Std.Ops", "Deref")
}

pub fn getDerefGetName() -> QualifiedName {
    build("Std.Ops", "Deref").add(format!("get"))
}

pub fn getDerefSetName() -> QualifiedName {
    build("Std.Ops", "Deref").add(format!("set"))
}

pub fn getImplicitConvertName() -> QualifiedName {
    build("Std.Ops", "ImplicitConvert")
}

pub fn getImplicitConvertFnName() -> QualifiedName {
    build("Std.Ops", "ImplicitConvert").add(format!("implicitConvert"))
}

pub fn getAutoDropFnName() -> QualifiedName {
    build("siko", "autoDrop")
}

pub fn getVecNewName() -> QualifiedName {
    build("Vec", "Vec").add(format!("new"))
}

pub fn getVecPushName() -> QualifiedName {
    build("Vec", "Vec").add(format!("push"))
}

pub fn getStdBasicUtilAbortName() -> QualifiedName {
    build("Std.Basic.Util", "abort")
}

pub fn getStdBasicUtilPrintStrName() -> QualifiedName {
    build("Std.Basic.Util", "printStr")
}

pub fn getBoxTypeName() -> QualifiedName {
    build("Box", "Box")
}

pub fn getBoxNewFnName() -> QualifiedName {
    build("Box", "Box").add(format!("new"))
}

pub fn getBoxReleaseFnName() -> QualifiedName {
    build("Box", "Box").add(format!("release"))
}

pub fn getBoxGetFnName() -> QualifiedName {
    build("Box", "Box").add(format!("get"))
}
