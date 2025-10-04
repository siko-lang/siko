use super::Type::Type;

pub struct Param {
    pub name: String,
    pub ty: Type,
}

pub struct CExternInfo {
    pub name: String,
    pub headerName: Option<String>,
}

pub enum ExternKind {
    C(CExternInfo),
    Builtin,
}

pub struct Function {
    pub name: String,
    pub fullName: String,
    pub args: Vec<Param>,
    pub result: Type,
    pub blocks: Vec<Block>,
    pub externKind: Option<ExternKind>,
}

impl Function {
    pub fn isExternC(&self) -> bool {
        self.externKind
            .as_ref()
            .map(|kind| match kind {
                ExternKind::C(_) => true,
                ExternKind::Builtin => false,
            })
            .unwrap_or(false)
    }

    pub fn hasHeaderName(&self) -> bool {
        self.externKind
            .as_ref()
            .map(|kind| match kind {
                ExternKind::C(info) => info.headerName.is_some(),
                ExternKind::Builtin => false,
            })
            .unwrap_or(false)
    }
}

pub struct Block {
    pub id: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone)]
pub enum Value {
    Numeric(String, Type),
    String(String, Type),
}

pub struct Branch {
    pub value: Value,
    pub block: String,
}

pub enum GetMode {
    Noop,
    Ref,
}

pub enum IntegerOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    LessThan,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
}

pub enum Instruction {
    Declare(Variable),
    StoreLiteral(Variable, Value),
    LoadPtr(Variable, Variable),
    StorePtr(Variable, Variable),
    Reference(Variable, Variable),
    FunctionCall(Option<Variable>, String, Vec<Variable>),
    Return(Variable),
    GetField(Variable, Variable, i32, GetMode),
    SetField(Variable, Variable, Vec<i32>),
    Jump(String),
    Memcpy(Variable, Variable),
    Bitcast(Variable, Variable),
    Switch(Variable, String, Vec<Branch>),
    AddressOfField(Variable, Variable, i32),
    IntegerOp(Variable, Variable, Variable, IntegerOp),
    FunctionPtr(Variable, String),
    FunctionPtrCall(Variable, Variable, Vec<Variable>),
    Sizeof(Variable, Variable),
    Transmute(Variable, Variable),
    CreateUninitializedArray(Variable),
}
