use crate::siko::{
    ir::Function::{Block, Function, Instruction, InstructionId, InstructionKind},
    ownership::Path::Path,
};

use super::CFG::{Edge, Key, Node, NodeKind, CFG};

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

    fn processGenericInstruction(
        &mut self,
        i: &Instruction,
        last: Option<Key>,
        kind: NodeKind,
    ) -> Key {
        let key = Key::Instruction(i.id);
        let node = Node::new(kind);
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
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
                }
                InstructionKind::DynamicFunctionCall(_, _) => todo!(),
                InstructionKind::If(_, trueBranch, falseBranch) => {
                    let ifKey = Key::If(instruction.id);
                    let ifEnd = Node::new(NodeKind::IfEnd);
                    self.cfg.addNode(ifKey.clone(), ifEnd);
                    let block = f.getBlockById(*trueBranch);
                    let trueLast = self.processBlock(block, last.clone(), f);
                    self.cfg.addEdge(Edge::new(trueLast, ifKey.clone()));
                    if let Some(falseBranch) = falseBranch {
                        let block = f.getBlockById(*falseBranch);
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
                        last = Some(self.processGenericInstruction(
                            instruction,
                            last,
                            NodeKind::Generic,
                        ));
                        continue;
                    };
                    let key = Key::Instruction(instruction.id);
                    let mut node = Node::new(NodeKind::ValueRef);
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
                    last = Some(key);
                }
                InstructionKind::Bind(v, _) => {
                    last = Some(self.processGenericInstruction(
                        instruction,
                        last,
                        NodeKind::Bind(v.clone()),
                    ));
                }
                InstructionKind::Tuple(_) => {
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
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
        f.dump();
        let block = &f.getFirstBlock();
        self.processBlock(block, None, f);
        self.cfg.updateEdges();
    }
}
