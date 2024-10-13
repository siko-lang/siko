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

pub enum Value {
    Void,
    LiteralNumeric(i64),
    Var(Variable),
    Field(Box<Value>, String),
}

pub enum Instruction {
    StackAllocate(Variable),
    Reference(Variable, Variable),
    Call(Variable, String, Vec<Variable>),
    Assignment(Value, Value),
    Return(Value),
}
