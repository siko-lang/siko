use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::Instruction::Instruction;
use super::Instruction::InstructionKind;
use super::Instruction::Tag;
use super::{ConstraintContext::ConstraintContext, Type::Type};

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

impl Debug for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.id)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableName {
    Transform(u32),
    MatchVar(u32),
    MatchValue(u32),
    LoopVar(u32),
    LoopFinalValue(u32),
    FunctionResult(u32),
    BlockValue(u32),
    ImplicitRef(u32),
    Ref(u32),
    FieldRef(u32),
    Unit(u32),
    Tuple(u32),
    TupleIndex(u32),
    Jump(u32),
    Literal(u32),
    Ret(u32),
    Call(u32),
    Local(String, u32),
    Arg(String),
    DropVar(u32),
    AutoDropResult(u32),
    ImplicitCloneRef(u32),
    ImplicitClone(u32),
    LocalArg(u32),
}

impl VariableName {
    pub fn visibleName(&self) -> String {
        match self {
            VariableName::Transform(i) => format!("transform{}", i),
            VariableName::MatchVar(i) => format!("matchVar{}", i),
            VariableName::MatchValue(i) => format!("matchValue{}", i),
            VariableName::LoopVar(i) => format!("loopVar{}", i),
            VariableName::LoopFinalValue(i) => format!("loopFinalValue{}", i),
            VariableName::FunctionResult(i) => format!("functionResult{}", i),
            VariableName::BlockValue(i) => format!("blockValue{}", i),
            VariableName::ImplicitRef(i) => format!("implicitRef{}", i),
            VariableName::Ref(i) => format!("ref{}", i),
            VariableName::FieldRef(i) => format!("fieldRef{}", i),
            VariableName::Unit(i) => format!("unit{}", i),
            VariableName::Tuple(i) => format!("tuple{}", i),
            VariableName::TupleIndex(i) => format!("tupleIndex{}", i),
            VariableName::Jump(i) => format!("jump{}", i),
            VariableName::Literal(i) => format!("lit{}", i),
            VariableName::Ret(i) => format!("ret{}", i),
            VariableName::Call(i) => format!("call{}", i),
            VariableName::Local(n, _) => n.clone(),
            VariableName::Arg(n) => n.clone(),
            VariableName::DropVar(i) => format!("dropVar{}", i),
            VariableName::AutoDropResult(i) => format!("autoDropResult{}", i),
            VariableName::ImplicitCloneRef(i) => format!("implicitCloneRef{}", i),
            VariableName::ImplicitClone(i) => format!("implicitClone{}", i),
            VariableName::LocalArg(i) => format!("localArg{}", i),
        }
    }
}

impl Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Transform(i) => write!(f, "transform{}", i),
            VariableName::MatchVar(i) => write!(f, "matchVar{}", i),
            VariableName::MatchValue(i) => write!(f, "matchValue{}", i),
            VariableName::LoopVar(i) => write!(f, "loopVar{}", i),
            VariableName::LoopFinalValue(i) => write!(f, "loopFinalValue{}", i),
            VariableName::FunctionResult(i) => write!(f, "functionResult{}", i),
            VariableName::BlockValue(i) => write!(f, "blockValue{}", i),
            VariableName::ImplicitRef(i) => write!(f, "implicitRef{}", i),
            VariableName::Ref(i) => write!(f, "ref{}", i),
            VariableName::FieldRef(i) => write!(f, "fieldRef{}", i),
            VariableName::Unit(i) => write!(f, "unit{}", i),
            VariableName::Tuple(i) => write!(f, "tuple{}", i),
            VariableName::TupleIndex(i) => write!(f, "tupleIndex{}", i),
            VariableName::Jump(i) => write!(f, "jump{}", i),
            VariableName::Literal(i) => write!(f, "lit{}", i),
            VariableName::Ret(i) => write!(f, "ret{}", i),
            VariableName::Call(i) => write!(f, "call{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropVar(i) => write!(f, "dropVar{}", i),
            VariableName::AutoDropResult(i) => write!(f, "autoDropResult{}", i),
            VariableName::ImplicitCloneRef(i) => write!(f, "implicitCloneRef{}", i),
            VariableName::ImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::LocalArg(i) => write!(f, "localArg{}", i),
        }
    }
}

impl Debug for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Transform(i) => write!(f, "transform{}", i),
            VariableName::MatchVar(i) => write!(f, "matchVar{}", i),
            VariableName::MatchValue(i) => write!(f, "matchValue{}", i),
            VariableName::LoopVar(i) => write!(f, "loopVar{}", i),
            VariableName::LoopFinalValue(i) => write!(f, "loopFinalValue{}", i),
            VariableName::FunctionResult(i) => write!(f, "functionResult{}", i),
            VariableName::BlockValue(i) => write!(f, "blockValue{}", i),
            VariableName::ImplicitRef(i) => write!(f, "implicitRef{}", i),
            VariableName::Ref(i) => write!(f, "ref{}", i),
            VariableName::FieldRef(i) => write!(f, "fieldRef{}", i),
            VariableName::Unit(i) => write!(f, "unit{}", i),
            VariableName::Tuple(i) => write!(f, "tuple{}", i),
            VariableName::TupleIndex(i) => write!(f, "tupleIndex{}", i),
            VariableName::Jump(i) => write!(f, "jump{}", i),
            VariableName::Literal(i) => write!(f, "lit{}", i),
            VariableName::Ret(i) => write!(f, "ret{}", i),
            VariableName::Call(i) => write!(f, "call{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropVar(i) => write!(f, "dropVar{}", i),
            VariableName::AutoDropResult(i) => write!(f, "autoDropResult{}", i),
            VariableName::ImplicitCloneRef(i) => write!(f, "implicitCloneRef{}", i),
            VariableName::ImplicitClone(i) => write!(f, "implicitClone{}", i),
            VariableName::LocalArg(i) => write!(f, "localArg{}", i),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub value: VariableName,
    pub location: Location,
    pub ty: Option<Type>,
    pub index: u32,
}

impl Variable {
    pub fn getType(&self) -> &Type {
        match &self.ty {
            Some(ty) => ty,
            None => panic!("No type found for var {}", self.value),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "${}/{}: {}", self.value, self.index, ty)
        } else {
            write!(f, "${}/{}", self.value, self.index)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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

    pub fn add(&mut self, kind: InstructionKind, location: Location, implicit: bool) {
        self.instructions.push(Instruction {
            implicit: implicit,
            kind: kind,
            location: location,
            tags: Vec::new(),
        });
    }

    pub fn insert(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        self.instructions.insert(
            index,
            Instruction {
                implicit: implicit,
                kind: kind,
                location: location,
                tags: Vec::new(),
            },
        );
    }

    pub fn replace(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        let tags = self.instructions[index].tags.clone();
        self.instructions[index] = Instruction {
            implicit: implicit,
            kind: kind,
            location: location,
            tags: tags,
        };
    }

    pub fn addTag(&mut self, index: usize, tag: Tag) {
        self.instructions[index].tags.push(tag);
    }

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for instruction in &self.instructions {
            instruction.dump();
        }
    }

    pub fn getTags(&self, index: usize) -> Vec<Tag> {
        self.instructions[index].tags.clone()
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block {}:", self.id)?;
        for instruction in &self.instructions {
            writeln!(f, "    {}", instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub blocks: Vec<Block>,
    pub varTypes: BTreeMap<VariableName, Type>,
}

impl Body {
    pub fn new() -> Body {
        Body {
            blocks: Vec::new(),
            varTypes: BTreeMap::new(),
        }
    }

    pub fn addBlock(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        &self.blocks[id.id as usize]
    }

    pub fn setType(&mut self, var: Variable, ty: Type) {
        self.varTypes.insert(var.value, ty);
    }

    pub fn getType(&self, var: &Variable) -> Option<Type> {
        self.varTypes.get(&var.value).cloned()
    }

    pub fn dump(&self) {
        for block in &self.blocks {
            block.dump();
        }
    }

    pub fn getInstruction(&self, block_id: BlockId, index: usize) -> Option<Instruction> {
        if let Some(block) = self.blocks.get(block_id.id as usize) {
            if let Some(instruction) = block.instructions.get(index) {
                return Some(instruction.clone());
            }
        }
        None
    }

    pub fn getAllBlockIds(&self) -> VecDeque<BlockId> {
        let mut ids = VecDeque::new();
        for block in &self.blocks {
            ids.push_back(block.id);
        }
        ids
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
    TraitMemberDecl(QualifiedName),
    TraitMemberDefinition(QualifiedName),
}

impl FunctionKind {
    pub fn isTraitCall(&self) -> Option<QualifiedName> {
        match self {
            FunctionKind::TraitMemberDecl(qn) | FunctionKind::TraitMemberDefinition(qn) => Some(qn.clone()),
            _ => None,
        }
    }
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
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.name, self.getType())?;
        writeln!(f, "constraints {}", self.constraintContext)?;
        match &self.body {
            Some(body) => write!(f, "{}", body),
            None => write!(f, "  <no body>"),
        }
    }
}
