use std::fmt::Display;

use crate::siko::{
    location::Location::Location, ownership::Borrowchecker::BorrowInfo,
    qualifiedname::QualifiedName,
};

use super::Type::Type;

#[derive(Debug, Clone)]
pub enum ValueKind {
    Arg(String),
    LoopVar(String),
    Value(String, InstructionId),
    Implicit(InstructionId),
}

impl ValueKind {
    pub fn getValue(&self) -> Option<String> {
        match &self {
            ValueKind::Arg(v) => Some(v.clone()),
            ValueKind::LoopVar(v) => Some(v.clone()),
            ValueKind::Value(v, _) => Some(v.clone()),
            ValueKind::Implicit(_) => None,
        }
    }
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ValueKind::Arg(n) => write!(f, "@arg/{}", n),
            ValueKind::LoopVar(n) => write!(f, "loop(${})", n),
            ValueKind::Value(n, bindId) => write!(f, "${}/{}", n, bindId),
            ValueKind::Implicit(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Parameter {
    Named(String, Type, bool),
    SelfParam(bool, Type),
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

impl InstructionId {
    pub fn empty() -> InstructionId {
        InstructionId {
            blockId: BlockId { id: 0 },
            id: 0,
        }
    }
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

#[derive(Debug, Clone)]
pub enum InstructionKind {
    FunctionCall(QualifiedName, Vec<InstructionId>),
    DynamicFunctionCall(InstructionId, Vec<InstructionId>),
    If(InstructionId, BlockId, Option<BlockId>),
    BlockRef(BlockId),
    Loop(String, InstructionId, BlockId),
    ValueRef(ValueKind, Vec<String>),
    Bind(String, InstructionId),
    Tuple(Vec<InstructionId>),
    TupleIndex(InstructionId, u32),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(char),
    Continue(InstructionId, InstructionId),
    Break(InstructionId, InstructionId),
    Return(InstructionId),
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
            InstructionKind::ValueRef(v, names) => format!("{}/{:?}", v, names),
            InstructionKind::Bind(v, rhs) => format!("${} = {}", v, rhs),
            InstructionKind::Loop(v, init, body) => format!("loop ${} = {} {}", v, init, body),
            InstructionKind::Tuple(args) => format!("tuple({:?})", args),
            InstructionKind::TupleIndex(id, index) => format!("tupleindex:{}.{}", id, index),
            InstructionKind::StringLiteral(v) => format!("s:[{}]", v),
            InstructionKind::IntegerLiteral(v) => format!("i:[{}]", v),
            InstructionKind::CharLiteral(v) => format!("c:[{}]", v),
            InstructionKind::Continue(id, loopId) => format!("continue({}, {})", id, loopId),
            InstructionKind::Break(id, loopId) => format!("break({}, {})", id, loopId),
            InstructionKind::Return(id) => format!("return({})", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: InstructionId,
    pub kind: InstructionKind,
    pub ty: Option<Type>,
    pub location: Location,
    pub borrowInfo: Option<BorrowInfo>,
}

impl Instruction {
    pub fn dump(&self) {
        if let Some(ty) = &self.ty {
            println!("    {}: {} {}", self.id, self.kind.dump(), ty);
        } else {
            println!("    {}: {}", self.id, self.kind.dump());
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.kind.dump())
    }
}

#[derive(Debug, Clone)]
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

    pub fn peekNextInstructionId(&self) -> InstructionId {
        let id = InstructionId {
            blockId: self.id,
            id: self.instructions.len() as u32,
        };
        id
    }

    pub fn add(&mut self, kind: InstructionKind, location: Location) -> InstructionId {
        let id = InstructionId {
            blockId: self.id,
            id: self.instructions.len() as u32,
        };
        self.instructions.push(Instruction {
            id: id,
            kind: kind,
            ty: None,
            location: location,
            borrowInfo: None,
        });
        id
    }

    pub fn getLastId(&self) -> InstructionId {
        self.instructions.last().expect("Empty block!").id
    }

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for instruction in &self.instructions {
            instruction.dump();
        }
    }
}

#[derive(Debug, Clone)]
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
    pub fn setBorrowInfo(&mut self, id: InstructionId, borrowInfo: BorrowInfo) {
        self.blocks[id.blockId.id as usize].instructions[id.id as usize].borrowInfo =
            Some(borrowInfo);
    }

    pub fn getBlockByRef(&self, id: InstructionId) -> &Block {
        let i = self.getInstruction(id);
        match &i.kind {
            InstructionKind::BlockRef(id) => &self.blocks[id.id as usize],
            _ => panic!("getBlockByRef: instruction is not block ref!"),
        }
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        &self.blocks[id.id as usize]
    }

    pub fn getInstruction(&self, id: InstructionId) -> &Instruction {
        &self.blocks[id.blockId.id as usize].instructions[id.id as usize]
    }

    pub fn dump(&self) {
        for block in &self.blocks {
            block.dump();
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn setBorrowInfo(&mut self, id: InstructionId, borrowInfo: BorrowInfo) {
        if let Some(body) = &mut self.body {
            body.setBorrowInfo(id, borrowInfo)
        } else {
            panic!("setBorrowInfo: no body found");
        }
    }

    pub fn getBlockByRef(&self, id: InstructionId) -> &Block {
        if let Some(body) = &self.body {
            body.getBlockByRef(id)
        } else {
            panic!("getBlockByRef: no body found");
        }
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        if let Some(body) = &self.body {
            body.getBlockById(id)
        } else {
            panic!("getBlockById: no body found");
        }
    }

    pub fn getFirstBlock(&self) -> &Block {
        if let Some(body) = &self.body {
            &body.blocks[0]
        } else {
            panic!("getFirstBlock: no body found");
        }
    }

    pub fn getType(&self) -> Type {
        let mut args = Vec::new();
        for param in &self.params {
            match &param {
                Parameter::Named(_, ty, _) => args.push(ty.clone()),
                Parameter::SelfParam(_, ty) => args.push(ty.clone()),
            }
        }
        Type::Function(args, Box::new(self.result.clone()))
    }

    pub fn dump(&self) {
        println!("{}:", self.name);
        match &self.body {
            Some(body) => body.dump(),
            None => println!("  <no body>"),
        }
    }
}
