use crate::siko::{
    ir::Function::{Block, Function, Instruction, InstructionId, InstructionKind},
    ownership::Path::Path,
};

use super::CFG::{Edge, Key, Node, CFG};

pub struct Builder {
    cfg: CFG,
    loopStarts: Vec<InstructionId>,
    loopEnds: Vec<InstructionId>,
}

impl Builder {
    pub fn new(name: String) -> Builder {
        Builder {
            cfg: CFG::new(name),
            loopStarts: Vec::new(),
            loopEnds: Vec::new(),
        }
    }

    pub fn getCFG(self) -> CFG {
        self.cfg
    }

    fn processGenericInstruction(&mut self, i: &Instruction, last: Option<Key>) -> Key {
        let key = Key::Instruction(i.id);
        let node = Node::new(format!("{}", i));
        self.cfg.addNode(key.clone(), node);
        if let Some(last) = last {
            let edge = Edge::new(last, key.clone());
            self.cfg.addEdge(edge);
        }
        key
    }

    fn processBlock(&mut self, block: &Block, mut last: Option<Key>, f: &Function) -> Key {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::FunctionCall(_, _) => {
                    last = Some(self.processGenericInstruction(instruction, last));
                }
                InstructionKind::DynamicFunctionCall(_, _) => todo!(),
                InstructionKind::If(_, trueBranch, falseBranch) => {
                    let ifKey = Key::If(instruction.id);
                    let ifEnd = Node::new("if_end".to_string());
                    self.cfg.addNode(ifKey.clone(), ifEnd);
                    let block = f.getBlockByRef(*trueBranch);
                    let trueLast = self.processBlock(block, last.clone(), f);
                    self.cfg.addEdge(Edge::new(trueLast, ifKey.clone()));
                    if let Some(falseBranch) = falseBranch {
                        let block = f.getBlockByRef(*falseBranch);
                        let falseLast = self.processBlock(block, last.clone(), f);
                        self.cfg.addEdge(Edge::new(falseLast, ifKey.clone()));
                    }
                    last = Some(ifKey);
                }
                InstructionKind::BlockRef(id) => {
                    let block = f.getBlockById(*id);
                    last = Some(self.processBlock(block, last, f));
                }
                InstructionKind::ValueRef(v, fields) => {
                    let value = if let Some(v) = v.getValue() {
                        v
                    } else {
                        last = Some(self.processGenericInstruction(instruction, last));
                        continue;
                    };
                    let key = Key::Instruction(instruction.id);
                    let mut node = Node::new(format!("{}", instruction));
                    if fields.is_empty() {
                        node.usage = Some(Path::WholePath(value));
                    } else {
                        node.usage = Some(Path::PartialPath(value, fields.clone()));
                    }
                    self.cfg.addNode(key.clone(), node);
                    if let Some(last) = last.clone() {
                        let edge = Edge::new(last, key.clone());
                        self.cfg.addEdge(edge);
                    }
                }
                InstructionKind::Bind(_, _) => {
                    last = Some(self.processGenericInstruction(instruction, last));
                }
                InstructionKind::Tuple(_) => {
                    last = Some(self.processGenericInstruction(instruction, last));
                }
                InstructionKind::TupleIndex(_, _) => todo!(),
                InstructionKind::StringLiteral(_) => todo!(),
                InstructionKind::IntegerLiteral(_) => todo!(),
                InstructionKind::CharLiteral(_) => todo!(),
            }
        }
        last.unwrap()
    }

    pub fn build(&mut self, f: &Function) {
        let block = &f.getFirstBlock();
        self.processBlock(block, None, f);
        self.cfg.updateEdges();
    }
}
