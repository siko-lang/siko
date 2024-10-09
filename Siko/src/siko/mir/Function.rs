use crate::siko::qualifiedname::QualifiedName;

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

pub enum InstructionKind {
    FunctionCall(QualifiedName),
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
