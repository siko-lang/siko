use std::fmt::Display;

use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug, Clone)]
pub enum ValueKind {
    Arg(String),
    Value(String, InstructionId),
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ValueKind::Arg(n) => write!(f, "@arg/{}", n),
            ValueKind::Value(n, bindId) => write!(f, "${}/{}", n, bindId),
        }
    }
}

#[derive(Debug)]
pub enum Parameter {
    Named(String, Type, bool),
    SelfParam(bool),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BlockId {
    pub id: u32,
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.id)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct InstructionId {
    blockId: BlockId,
    id: u32,
}

impl Display for InstructionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}.{})", self.blockId, self.id)
    }
}

impl std::fmt::Debug for InstructionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}.{})", self.blockId, self.id)
    }
}

#[derive(Debug)]
pub enum InstructionKind {
    FunctionCall(QualifiedName, Vec<InstructionId>),
    DynamicFunctionCall(InstructionId, Vec<InstructionId>),
    If(InstructionId, InstructionId, Option<InstructionId>),
    BlockRef(BlockId),
    ValueRef(ValueKind),
    Bind(String, InstructionId),
}

impl InstructionKind {
    pub fn dump(&self) -> String {
        match self {
            InstructionKind::FunctionCall(name, args) => format!("{}({:?})", name, args),
            InstructionKind::DynamicFunctionCall(callable, args) => {
                format!("{}({:?})", callable, args)
            }
            InstructionKind::If(cond, t, f) => match f {
                Some(f) => format!("if {} {{ {} }} else {{ {} }}", cond, t, f),
                None => format!("if {} {{ {} }}", cond, t),
            },
            InstructionKind::BlockRef(id) => format!("blockref: {}", id),
            InstructionKind::ValueRef(v) => format!("{}", v),
            InstructionKind::Bind(v, rhs) => format!("${} = {}", v, rhs),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub id: InstructionId,
    pub kind: InstructionKind,
}

impl Instruction {
    pub fn dump(&self) {
        println!("    {}: {}", self.id, self.kind.dump());
    }
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

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for instruction in &self.instructions {
            instruction.dump();
        }
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

    pub fn dump(&self) {
        for block in &self.blocks {
            block.dump();
        }
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

    pub fn dump(&self) {
        match &self.body {
            Some(body) => body.dump(),
            None => println!("<no body>"),
        }
    }
}
