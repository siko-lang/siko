use super::Type::Type;

#[derive(Debug)]
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

pub enum Value {
    Numeric(i64),
    Var(Variable),
}

pub enum Instruction {
    Declare(Variable),
    GetFieldRef(Variable, Variable, i32),
    Reference(Variable, Variable),
    Call(Variable, String, Vec<Variable>),
    Assign(Variable, Value),
    Return(Variable),
}
