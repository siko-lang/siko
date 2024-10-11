use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

pub struct Function {
    pub name: QualifiedName,
    pub blocks: Vec<BasicBlock>,
}

impl Function {
    pub fn new(name: QualifiedName) -> Function {
        Function {
            name: name,
            blocks: Vec::new(),
        }
    }
}

pub struct Aligment {
    pub alignment: u32,
}

pub struct Variable {
    pub name: String,
    pub ty: Type,
    pub alignment: Aligment,
}

pub struct AllocInfo {
    pub var: Variable,
}

pub enum InstructionKind {
    Allocate(AllocInfo),
    StoreVar(Variable, Variable),
    StoreNumeric(Variable, i64),
    FunctionCall(Variable, QualifiedName),
}

pub struct Instruction {
    pub kind: InstructionKind,
}

impl Instruction {
    pub fn new(kind: InstructionKind) -> Instruction {
        Instruction { kind: kind }
    }
}

pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            instructions: Vec::new(),
        }
    }
}
