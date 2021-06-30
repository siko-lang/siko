use std::collections::BTreeMap;

pub struct Variant {
    pub name: String,
    pub ty: ExtendedType,
}

pub struct Adt {
    pub name: String,
    pub variants: Vec<Variant>,
    pub args: Vec<i64>,
}

pub struct Field {
    pub name: String,
    pub ty: ExtendedType,
}

pub struct External {
    pub ty: ExtendedType,
}

pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub externals: Option<Vec<External>>,
    pub args: Vec<i64>,
}

pub struct Expr {
    pub id: i64,
    pub ty: ExtendedType,
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum Checker {
    Variant(i64, String, String),
    Other(String),
    Wildcard,
}

pub struct Case {
    pub checker: Checker,
    pub body: i64,
}

pub enum ExprKind {
    Do(Vec<i64>),
    StaticFunctionCall(String, Vec<i64>),
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

pub struct ExtendedType {
    pub ty: String,
    pub args: Vec<i64>,
}

impl ExtendedType {
    pub fn new(ty: String) -> ExtendedType {
        ExtendedType {
            ty: ty,
            args: Vec::new(),
        }
    }

    pub fn add_args(&mut self, mut args: Vec<i64>) {
        self.args.append(&mut args);
    }
}

pub struct Function {
    pub name: String,
    pub args: Vec<ExtendedType>,
    pub result: ExtendedType,
    pub kind: FunctionKind,
}

pub enum Data {
    Adt(Adt),
    Record(Record),
}

pub struct Program {
    pub data: BTreeMap<String, Data>,
    pub functions: BTreeMap<String, Function>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            data: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }
}

pub enum Type {
    Adt(String),
    Record(String),
    Never,
}
