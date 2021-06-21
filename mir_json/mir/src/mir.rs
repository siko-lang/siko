use std::collections::BTreeMap;

pub struct Variant {
    pub name: String,
    pub ty: String,
}

pub struct Adt {
    pub name: String,
    pub variants: Vec<Variant>,
}

pub struct Field {
    pub name: String,
    pub ty: String,
}

pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub external: bool,
}

pub struct Expr {
    pub id: String,
    pub ty: String,
    pub kind: ExprKind,
}

pub struct Case {
    pub checker: String,
    pub body: i64,
}

pub enum ExprKind {
    Do(Vec<i64>),
    StaticFunctionCall(Vec<i64>),
    IntegerLiteral(String),
    StringLiteral(String),
    FloatLiteral(String),
    CharLiteral(String),
    VarDecl(String, i64),
    VarRef(String),
    FieldAccess(String, i64),
    If(i64, i64, i64),
    List(Vec<i64>),
    Return(i64),
    Continue(i64),
    Break(i64),
    Loop(String, i64, i64),
    CaseOf(i64, Vec<Case>),
    Converter(i64),
}

pub enum FunctionKind {
    Normal(Vec<Expr>),
    VariantCtor(i64),
    RecordCtor,
    External,
}

pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub result: String,
    pub kind: FunctionKind,
}

pub struct Program {
    pub adts: BTreeMap<String, Adt>,
    pub records: BTreeMap<String, Record>,
    pub functions: BTreeMap<String, Function>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            adts: BTreeMap::new(),
            records: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }
}
