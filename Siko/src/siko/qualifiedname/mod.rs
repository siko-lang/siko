use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum QualifiedName {
    Module(String),
    Instance(Box<QualifiedName>, u64),
    Item(Box<QualifiedName>, String),
    Monomorphized(Box<QualifiedName>, String),
}

impl QualifiedName {
    pub fn add(&self, item: String) -> QualifiedName {
        QualifiedName::Item(Box::new(self.clone()), item)
    }

    pub fn module(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Instance(p, _) => p.module(),
            QualifiedName::Item(p, _) => p.module(),
            QualifiedName::Monomorphized(p, _) => p.module(),
        }
    }

    pub fn base(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Instance(p, _) => *p.clone(),
            QualifiedName::Item(p, _) => *p.clone(),
            QualifiedName::Monomorphized(p, _) => *p.clone(),
        }
    }

    pub fn monomorphized(&self, args: String) -> QualifiedName {
        QualifiedName::Monomorphized(Box::new(self.clone()), args)
    }

    pub fn toString(&self) -> String {
        format!("{}", self)
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            QualifiedName::Module(i) => write!(f, "{}", i),
            QualifiedName::Instance(p, i) => write!(f, "{}/{}", p, i),
            QualifiedName::Item(p, i) => write!(f, "{}.{}", p, i),
            QualifiedName::Monomorphized(p, args) => {
                if args.is_empty() {
                    write!(f, "{}", p)
                } else {
                    write!(f, "{}#{}", p, args)
                }
            }
        }
    }
}

pub fn build(m: &str, name: &str) -> QualifiedName {
    QualifiedName::Item(Box::new(QualifiedName::Module(m.to_string())), name.to_string())
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
