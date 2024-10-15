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

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone)]
pub enum Value {
    Void,
    Variable(Variable),
    Numeric(i64),
}

pub enum Instruction {
    Allocate(Variable),
    Store(Variable, Value),
    LoadVar(Variable, Variable),
    FunctionCall(Variable, String, Vec<Variable>),
    Return(Value),
    GetFieldRef(Variable, Variable, i32),
    IntegerLiteral(Variable, String),
    Jump(String),
}
