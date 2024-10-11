use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

pub struct Function {
    pub name: QualifiedName,
    pub result: Type,
    pub blocks: Vec<BasicBlock>,
}

impl Function {
    pub fn new(name: QualifiedName, result: Type) -> Function {
        Function {
            name: name,
            result: result,
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Aligment {
    pub alignment: u32,
}

#[derive(Debug, Clone)]
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
    LoadVar(Variable, Variable),
    StoreNumeric(Variable, i64),
    FunctionCall(Variable, QualifiedName),
    Return(Variable),
    ReturnVoid,
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
