use std::cell::RefCell;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::siko::hir::Instruction::Instruction;
use crate::siko::hir::Instruction::InstructionKind;
use crate::siko::hir::Variable::CopyHandler;
use crate::siko::location::Location::Location;

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
pub struct BlockInner {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
}

impl BlockInner {
    pub fn new(id: BlockId) -> BlockInner {
        BlockInner {
            id: id,
            instructions: Vec::new(),
        }
    }

    pub fn newWith(id: BlockId, instructions: Vec<Instruction>) -> BlockInner {
        BlockInner { id, instructions }
    }

    pub fn copy(&self, map: &mut CopyHandler) -> BlockInner {
        let mut instructions = Vec::new();
        for instr in &self.instructions {
            instructions.push(instr.copy(map));
        }
        BlockInner {
            id: self.id,
            instructions,
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

    pub fn getInstruction(&self, index: usize) -> Instruction {
        self.instructions
            .get(index)
            .cloned()
            .expect("Invalid instruction index")
    }

    pub fn getInstructionOpt(&self, index: usize) -> Option<Instruction> {
        self.instructions.get(index).cloned()
    }

    pub fn getInstructions(&self) -> Vec<Instruction> {
        self.instructions.clone()
    }

    pub fn getLastInstruction(&self) -> Instruction {
        self.instructions.last().cloned().expect("No instructions in block")
    }

    pub fn getId(&self) -> BlockId {
        self.id
    }

    pub fn setId(&mut self, id: BlockId) {
        self.id = id;
    }

    pub fn isEmpty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn size(&self) -> usize {
        self.instructions.len()
    }
}

impl Display for BlockInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block {}:", self.id)?;
        for (index, instruction) in self.instructions.iter().enumerate() {
            writeln!(f, "    {:3}: {}", index, instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    inner: Rc<RefCell<BlockInner>>,
}

impl Block {
    pub fn new(id: BlockId) -> Block {
        Block {
            inner: Rc::new(RefCell::new(BlockInner::new(id))),
        }
    }

    pub fn copy(&self, map: &mut CopyHandler) -> Block {
        Block {
            inner: Rc::new(RefCell::new(self.inner.borrow().copy(map))),
        }
    }

    pub fn newWith(id: BlockId, instructions: Vec<Instruction>) -> Block {
        Block {
            inner: Rc::new(RefCell::new(BlockInner::newWith(id, instructions))),
        }
    }

    pub fn add(&mut self, kind: InstructionKind, location: Location, implicit: bool) {
        self.inner.borrow_mut().add(kind, location, implicit);
    }

    pub fn insert(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        self.inner.borrow_mut().insert(index, kind, location, implicit);
    }

    pub fn replace(&mut self, index: usize, kind: InstructionKind, location: Location, implicit: bool) {
        self.inner.borrow_mut().replace(index, kind, location, implicit);
    }

    pub fn remove(&mut self, index: usize) {
        self.inner.borrow_mut().remove(index);
    }

    pub fn dump(&self) {
        self.inner.borrow().dump();
    }

    pub fn getInstruction(&self, index: usize) -> Instruction {
        self.inner.borrow().getInstruction(index)
    }

    pub fn getInstructionOpt(&self, index: usize) -> Option<Instruction> {
        self.inner.borrow().getInstructionOpt(index)
    }

    pub fn getInstructions(&self) -> Vec<Instruction> {
        self.inner.borrow().getInstructions()
    }

    pub fn getLastInstruction(&self) -> Instruction {
        self.inner.borrow().getLastInstruction()
    }

    pub fn getId(&self) -> BlockId {
        self.inner.borrow().getId()
    }

    pub fn setId(&self, id: BlockId) {
        self.inner.borrow_mut().setId(id);
    }

    pub fn getInner(&self) -> Rc<RefCell<BlockInner>> {
        self.inner.clone()
    }

    pub fn isEmpty(&self) -> bool {
        self.inner.borrow().isEmpty()
    }

    pub fn size(&self) -> usize {
        self.inner.borrow().size()
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.borrow())
    }
}
