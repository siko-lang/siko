use std::fmt;

use crate::siko::qualifiedname::{build, QualifiedName};

pub fn getArrayTypeName() -> QualifiedName {
    build("Array", "Array")
}

pub fn getArrayUninitializedName() -> QualifiedName {
    build("Array", "Array").add("uninitialized".to_string())
}

pub fn getArrayLenName() -> QualifiedName {
    build("Array", "Array").add("len".to_string())
}

pub fn getArrayBaseName() -> QualifiedName {
    build("Array", "Array").add("base".to_string())
}

pub fn getMainName() -> QualifiedName {
    build("Main", "main")
}

pub fn getBoolTypeName() -> QualifiedName {
    build("Bool", "Bool")
}

pub fn getIntTypeName() -> QualifiedName {
    build("Int", "Int")
}

pub fn getU8TypeName() -> QualifiedName {
    build("Int", "U8")
}

pub fn getI8TypeName() -> QualifiedName {
    build("Int", "I8")
}

pub fn getU64TypeName() -> QualifiedName {
    build("Int", "U64")
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

pub fn getTrueName() -> QualifiedName {
    build("Bool", "Bool").add("True".to_string())
}

pub fn getFalseName() -> QualifiedName {
    build("Bool", "Bool").add("False".to_string())
}

pub fn getStringEqName() -> QualifiedName {
    build("String", "String").add("eq".to_string())
}

pub fn getNativePtrSizeOfName() -> QualifiedName {
    build("NativePtr", "sizeOf")
}

pub fn getNativePtrTransmuteName() -> QualifiedName {
    build("NativePtr", "transmute")
}

pub fn getNativePtrCastName() -> QualifiedName {
    build("NativePtr", "cast")
}

pub fn getNativePtrCloneName() -> QualifiedName {
    build("NativePtr", "clone")
}

pub fn getNativePtrIsNullName() -> QualifiedName {
    build("NativePtr", "isNull")
}

pub fn getCloneFnName() -> QualifiedName {
    build("Std.Ops.Basic", "Clone").add(format!("clone"))
}

pub enum IntKind {
    Int,
    U8,
    U32,
    U64,
    I32,
}

impl fmt::Display for IntKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntKind::Int => write!(f, "Int"),
            IntKind::U8 => write!(f, "U8"),
            IntKind::U32 => write!(f, "U32"),
            IntKind::U64 => write!(f, "U64"),
            IntKind::I32 => write!(f, "I32"),
        }
    }
}

pub fn getIntToU8Name() -> QualifiedName {
    build("Int", "Int").add(format!("toU8"))
}

pub fn getIntToU32Name() -> QualifiedName {
    build("Int", "Int").add(format!("toU32"))
}

pub fn getIntToU64Name() -> QualifiedName {
    build("Int", "Int").add(format!("toU64"))
}

pub fn getIntToI32Name() -> QualifiedName {
    build("Int", "Int").add(format!("toI32"))
}

pub fn getIntAddName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("add"))
}

pub fn getIntSubName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("sub"))
}

pub fn getIntMulName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("mul"))
}

pub fn getIntDivName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("div"))
}

pub fn getIntModName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("mod"))
}

pub fn getIntEqName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("eq"))
}

pub fn getIntLessThanName(kind: IntKind) -> QualifiedName {
    build("Int", &kind.to_string()).add(format!("lessThan"))
}

pub fn getDropFnName() -> QualifiedName {
    build("Std.Ops.Basic", "Drop").add(format!("drop"))
}

pub fn getDropName() -> QualifiedName {
    build("Std.Ops.Basic", "Drop")
}

pub fn getCopyName() -> QualifiedName {
    build("Std.Ops.Basic", "Copy")
}

pub fn getImplicitConvertName() -> QualifiedName {
    build("Std.Ops.Basic", "ImplicitConvert")
}

pub fn getImplicitConvertFnName() -> QualifiedName {
    build("Std.Ops.Basic", "ImplicitConvert").add(format!("implicitConvert"))
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

pub fn getRangeCtorName() -> QualifiedName {
    build("Range", "Range").add("range".to_string())
}
