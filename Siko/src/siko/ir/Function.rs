use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug)]
pub enum Parameter {
    Named(String, Type, bool),
    SelfParam(bool),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BlockId {
    pub id: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct InstructionId {
    blockId: BlockId,
    id: u32,
}

#[derive(Debug)]
pub enum InstructionKind {
    FunctionCall(QualifiedName, Vec<InstructionId>),
    DynamicFunctionCall(InstructionId, Vec<InstructionId>),
    If(InstructionId, InstructionId, Option<InstructionId>),
    BlockRef(BlockId),
    ValueRef(String),
}

#[derive(Debug)]
pub struct Instruction {
    pub id: InstructionId,
    pub kind: InstructionKind,
}

#[derive(Debug)]
pub struct Block {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(id: BlockId) -> Block {
        Block {
            id: id,
            instructions: Vec::new(),
        }
    }

    pub fn add(&mut self, kind: InstructionKind) -> InstructionId {
        let id = InstructionId {
            blockId: self.id,
            id: self.instructions.len() as u32,
        };
        self.instructions.push(Instruction { id: id, kind: kind });
        id
    }
}

#[derive(Debug)]
pub struct Body {
    pub blocks: Vec<Block>,
}

impl Body {
    pub fn new() -> Body {
        Body { blocks: Vec::new() }
    }

    pub fn addBlock(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Body>,
}

impl Function {
    pub fn new(
        name: QualifiedName,
        params: Vec<Parameter>,
        result: Type,
        body: Option<Body>,
    ) -> Function {
        Function {
            name: name,
            params: params,
            result: result,
            body: body,
        }
    }
}
