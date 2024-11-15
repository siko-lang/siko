use super::Type::Type;

pub struct Param {
    pub name: String,
    pub ty: Type,
}

pub struct Function {
    pub name: String,
    pub args: Vec<Param>,
    pub result: Type,
    pub blocks: Vec<Block>,
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
    Void,
    Variable(Variable),
    Numeric(String, Type),
    String(String, Type),
}

pub struct Branch {
    pub value: Value,
    pub block: String,
}

pub enum Instruction {
    Allocate(Variable),
    Store(Variable, Value),
    LoadVar(Variable, Variable),
    Reference(Variable, Variable),
    FunctionCall(String, Vec<Variable>),
    FunctionCallValue(Variable, String, Vec<Variable>),
    Return(Value),
    GetFieldRef(Variable, Variable, i32),
    SetField(Variable, Variable, Vec<i32>),
    Jump(String),
    Memcpy(Variable, Variable),
    MemcpyPtr(Variable, Variable),
    Bitcast(Variable, Variable),
    Switch(Variable, String, Vec<Branch>),
}
