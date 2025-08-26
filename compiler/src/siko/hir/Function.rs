use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fmt::Display;
use std::io::Write;

use crate::siko::hir::VariableAllocator::VariableAllocator;
use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::Instruction::Instruction;
use super::Instruction::InstructionKind;
use super::Variable::Variable;
use super::Variable::VariableName;
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
            kind: kind.setVariableKinds(),
            location: location,
        });
    }

    pub fn insert(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        self.instructions.insert(
            index,
            Instruction {
                implicit: implicit,
                kind: kind.setVariableKinds(),
                location: location,
            },
        );
    }

    pub fn replace(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        let isImplicit = self.instructions[index].implicit || implicit;
        self.instructions[index] = Instruction {
            implicit: isImplicit,
            kind: kind.setVariableKinds(),
            location: location,
        };
    }

    pub fn remove(&mut self, index: usize) {
        self.instructions.remove(index);
    }

    pub fn dump(&self) {
        println!("  Block {}:", self.id);
        for (index, instruction) in self.instructions.iter().enumerate() {
            print!("{}: ", index);
            instruction.dump();
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block {}:", self.id)?;
        for (index, instruction) in self.instructions.iter().enumerate() {
            writeln!(f, "    {:3}: {}", index, instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub blocks: BTreeMap<BlockId, Block>,
    pub varTypes: BTreeMap<VariableName, Type>,
    pub varAllocator: VariableAllocator,
}

impl Body {
    pub fn new() -> Body {
        Body {
            blocks: BTreeMap::new(),
            varTypes: BTreeMap::new(),
            varAllocator: VariableAllocator::new(),
        }
    }

    pub fn addBlock(&mut self, block: Block) {
        self.blocks.insert(block.id, block);
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        &self.blocks.get(&id).expect("Block not found")
    }

    pub fn setType(&mut self, var: Variable, ty: Type) {
        self.varTypes.insert(var.name(), ty);
    }

    pub fn getType(&self, var: &Variable) -> Option<Type> {
        self.varTypes.get(&var.name()).cloned()
    }

    pub fn dump(&self) {
        for (_, block) in &self.blocks {
            block.dump();
        }
    }

    pub fn getInstruction(&self, blockId: BlockId, index: usize) -> Option<Instruction> {
        if let Some(block) = self.blocks.get(&blockId) {
            if let Some(instruction) = block.instructions.get(index) {
                return Some(instruction.clone());
            }
        }
        None
    }

    pub fn getAllBlockIds(&self) -> VecDeque<BlockId> {
        let mut ids = VecDeque::new();
        for (id, _) in &self.blocks {
            ids.push_back(*id);
        }
        ids
    }

    pub fn cutBlock(&mut self, blockId: BlockId, index: usize, newBlockId: BlockId) {
        let block = self.blocks.get_mut(&blockId).expect("Block not found");
        let otherInstructions = block.instructions.split_off(index);
        let newBlock = self.blocks.get_mut(&newBlockId).expect("New block not found");
        newBlock.instructions = otherInstructions;
    }

    pub fn getBlockSize(&self, blockId: BlockId) -> usize {
        self.blocks.get(&blockId).expect("Block not found").instructions.len()
    }

    pub fn removeBlock(&mut self, block_id: BlockId) {
        self.blocks.remove(&block_id);
    }

    pub fn mergeBlocks(&mut self, sourceBlockId: BlockId, targetBlockId: BlockId) {
        let mut targetBlock = self.blocks.remove(&targetBlockId).expect("Target block not found");
        let sourceBlock = self.blocks.get_mut(&sourceBlockId).expect("Source block not found");
        sourceBlock.instructions.pop();
        sourceBlock.instructions.append(&mut targetBlock.instructions);
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, block) in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternKind {
    C,
    Builtin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    UserDefined,
    VariantCtor(i64),
    StructCtor,
    Extern(ExternKind),
    TraitMemberDecl(QualifiedName),
    TraitMemberDefinition(QualifiedName),
    EffectMemberDecl(QualifiedName),
    EffectMemberDefinition(QualifiedName),
}

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionKind::UserDefined => write!(f, "UserDefined"),
            FunctionKind::VariantCtor(id) => write!(f, "VariantCtor({})", id),
            FunctionKind::StructCtor => write!(f, "StructCtor"),
            FunctionKind::Extern(kind) => write!(f, "Extern({:?})", kind),
            FunctionKind::TraitMemberDecl(qn) => {
                write!(f, "TraitMemberDecl({})", qn)
            }
            FunctionKind::TraitMemberDefinition(qn) => {
                write!(f, "TraitMemberDefinition({})", qn)
            }
            FunctionKind::EffectMemberDecl(qn) => {
                write!(f, "EffectMemberDecl({})", qn)
            }
            FunctionKind::EffectMemberDefinition(qn) => {
                write!(f, "EffectMemberDefinition({})", qn)
            }
        }
    }
}

impl FunctionKind {
    pub fn isExternC(&self) -> bool {
        match self {
            FunctionKind::Extern(kind) => *kind == ExternKind::C,
            _ => false,
        }
    }

    pub fn isBuiltin(&self) -> bool {
        match self {
            FunctionKind::Extern(kind) => *kind == ExternKind::Builtin,
            _ => false,
        }
    }

    pub fn isCtor(&self) -> bool {
        match self {
            FunctionKind::VariantCtor(_) | FunctionKind::StructCtor => true,
            _ => false,
        }
    }

    pub fn isTraitCall(&self) -> bool {
        match self {
            FunctionKind::TraitMemberDecl(_) | FunctionKind::TraitMemberDefinition(_) => true,
            _ => false,
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

    pub fn isPure(&self) -> bool {
        match self.kind {
            FunctionKind::VariantCtor(_) | FunctionKind::StructCtor => true,
            _ => false,
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
            &body.blocks.get(&BlockId::first()).expect("Block not found")
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

    pub fn dumpToFile(&self, name: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(name).map_err(|e| {
            eprintln!("Error creating file {}: {}", name, e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to create function file")
        })?;
        writeln!(file, "{}", self).map_err(|e| {
            eprintln!("Error writing to file {}: {}", name, e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to write function name")
        })?;
        Ok(())
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
