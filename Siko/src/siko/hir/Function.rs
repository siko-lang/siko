use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::{ConstraintContext::ConstraintContext, Type::Type};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Arg(String, i64),
    Value(String, InstructionId),
}

impl ValueKind {
    pub fn getValue(&self) -> String {
        match &self {
            ValueKind::Arg(v, _) => v.clone(),
            ValueKind::Value(v, _) => v.clone(),
        }
    }
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ValueKind::Arg(n, index) => write!(f, "@arg/{}/{}", n, index),
            ValueKind::Value(n, bindId) => write!(f, "${}/{}", n, bindId),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Parameter {
    Named(String, Type, bool), // mutable
    SelfParam(bool, Type),     // mutable
}

impl Parameter {
    pub fn getName(&self) -> String {
        match &self {
            Parameter::Named(n, _, _) => n.clone(),
            Parameter::SelfParam(_, _) => "self".to_string(),
        }
    }

    pub fn getType(&self) -> Type {
        match &self {
            Parameter::Named(_, ty, _) => ty.clone(),
            Parameter::SelfParam(_, ty) => ty.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BlockId {
    pub id: u32,
}

impl BlockId {
    pub fn first() -> BlockId {
        BlockId { id: 0 }
    }
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
    pub fn first() -> InstructionId {
        InstructionId {
            blockId: BlockId { id: 0 },
            id: 0,
        }
    }

    pub fn simple(&self) -> String {
        format!("{}_{}", self.blockId.id, self.id)
    }

    pub fn getBlockById(&self) -> BlockId {
        self.blockId
    }

    pub fn getId(&self) -> u32 {
        self.id
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

#[derive(Clone, PartialEq)]
pub enum InstructionKind {
    FunctionCall(QualifiedName, Vec<InstructionId>),
    DynamicFunctionCall(InstructionId, Vec<InstructionId>),
    If(InstructionId, BlockId, Option<BlockId>),
    ValueRef(ValueKind, Vec<String>, Vec<u32>),
    Bind(String, InstructionId),
    Tuple(Vec<InstructionId>),
    TupleIndex(InstructionId, u32),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(char),
    Return(InstructionId),
    Ref(InstructionId),
    Drop(Vec<String>),
    Jump(BlockId),
    Assign(String, InstructionId),
    DeclareVar(String),
}

impl Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl Debug for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }
}

impl InstructionKind {
    pub fn dump(&self) -> String {
        match self {
            InstructionKind::FunctionCall(name, args) => format!("call({}({:?}))", name, args),
            InstructionKind::DynamicFunctionCall(callable, args) => {
                format!("DYN_CALL{}({:?})", callable, args)
            }
            InstructionKind::If(cond, t, f) => match f {
                Some(f) => format!("if {} {{ {} }} else {{ {} }}", cond, t, f),
                None => format!("if {} {{ {} }}", cond, t),
            },
            InstructionKind::ValueRef(v, names, _) => format!("{}/{:?}", v, names),
            InstructionKind::Bind(v, rhs) => format!("${} = {}", v, rhs),
            InstructionKind::Tuple(args) => format!("tuple({:?})", args),
            InstructionKind::TupleIndex(id, index) => format!("tupleindex:{}.{}", id, index),
            InstructionKind::StringLiteral(v) => format!("s:[{}]", v),
            InstructionKind::IntegerLiteral(v) => format!("i:[{}]", v),
            InstructionKind::CharLiteral(v) => format!("c:[{}]", v),
            InstructionKind::Return(id) => format!("return({})", id),
            InstructionKind::Ref(id) => format!("&({})", id),
            InstructionKind::Drop(values) => {
                format!("drop({})", values.join(", "))
            }
            InstructionKind::Jump(id) => {
                format!("jump({})", id)
            }
            InstructionKind::Assign(v, arg) => format!("assign({}, {})", v, arg),
            InstructionKind::DeclareVar(v) => format!("declare({})", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub id: InstructionId,
    pub implicit: bool,
    pub kind: InstructionKind,
    pub ty: Option<Type>,
    pub location: Location,
}

impl Instruction {
    pub fn dump(&self) {
        println!("    {}", self);
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "{}: {} {}", self.id, self.kind.dump(), ty)?;
        } else {
            write!(f, "{}: {}", self.id, self.kind.dump())?;
        }
        Ok(())
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

    pub fn getInstruction(&self, id: InstructionId) -> &Instruction {
        &self.instructions[id.id as usize]
    }

    pub fn add(&mut self, kind: InstructionKind, location: Location) -> InstructionId {
        self.addWithImplicit(kind, location, false)
    }

    pub fn addWithImplicit(&mut self, kind: InstructionKind, location: Location, implicit: bool) -> InstructionId {
        let id = InstructionId {
            blockId: self.id,
            id: self.instructions.len() as u32,
        };
        self.instructions.push(Instruction {
            id: id,
            implicit: implicit,
            kind: kind,
            ty: None,
            location: location,
        });
        id
    }

    pub fn getLastId(&self) -> InstructionId {
        self.instructions.iter().rev().next().expect("Empty block!").id
    }

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for instruction in &self.instructions {
            instruction.dump();
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Block {}:", self.id)?;
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        Ok(())
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

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    UserDefined,
    VariantCtor(i64),
    ClassCtor,
    Extern,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: QualifiedName,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Body>,
    pub constraintContext: ConstraintContext,
    pub kind: FunctionKind,
}

impl Function {
    pub fn new(
        name: QualifiedName,
        params: Vec<Parameter>,
        result: Type,
        body: Option<Body>,
        constraintContext: ConstraintContext,
        kind: FunctionKind,
    ) -> Function {
        Function {
            name: name,
            params: params,
            result: result,
            body: body,
            constraintContext: constraintContext,
            kind: kind,
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

    pub fn getInstruction(&self, id: InstructionId) -> &Instruction {
        if let Some(body) = &self.body {
            body.getInstruction(id)
        } else {
            panic!("getInstruction: no body found");
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
        println!("{}", self.name);
        match &self.body {
            Some(body) => body.dump(),
            None => println!("  <no body>"),
        }
    }

    pub fn instructions<'a>(&'a self) -> InstructionIterator<'a> {
        InstructionIterator::new(self)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.name, self.getType())?;
        match &self.body {
            Some(body) => write!(f, "{}", body),
            None => write!(f, "  <no body>"),
        }
    }
}

pub struct InstructionIterator<'a> {
    f: &'a Function,
    block: usize,
    instruction: usize,
}

impl<'a> InstructionIterator<'a> {
    fn new(f: &'a Function) -> InstructionIterator<'a> {
        InstructionIterator { f, block: 0, instruction: 0 }
    }
}

impl<'a> Iterator for InstructionIterator<'a> {
    type Item = &'a Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(body) = &self.f.body {
            if self.block >= body.blocks.len() {
                return None;
            }
            let block = &body.blocks[self.block];
            let item = &block.instructions[self.instruction];
            self.instruction += 1;
            if self.instruction >= block.instructions.len() {
                self.instruction = 0;
                self.block += 1;
            }
            return Some(item);
        } else {
            return None;
        }
    }
}
