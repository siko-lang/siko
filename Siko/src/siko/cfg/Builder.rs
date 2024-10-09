use crate::siko::{
    hir::{
        Function::{Block, Function, Instruction, InstructionKind, ValueKind},
        Type::Type,
    },
    ownership::Path::Path,
};

use super::CFG::{Edge, Key, Node, NodeKind, CFG};

pub struct Builder {
    cfg: CFG,
    loopStarts: Vec<Key>,
    loopEnds: Vec<Key>,
}

impl Builder {
    pub fn new(name: String, result: Type) -> Builder {
        let mut cfg = CFG::new(name);
        let end = Node::new(NodeKind::End, result);
        cfg.addNode(Key::End, end);
        Builder {
            cfg: cfg,
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
        let node = Node::new(kind, i.ty.clone().expect("ty not found"));
        self.cfg.addNode(key.clone(), node);
        if let Some(last) = last {
            let edge = Edge::new(last, key.clone());
            self.cfg.addEdge(edge);
        }
        key
    }

    fn processBlock(&mut self, block: &Block, mut last: Option<Key>, f: &Function) -> Option<Key> {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::FunctionCall(_, _) => {
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
                }
                InstructionKind::DynamicFunctionCall(_, _) => todo!(),
                InstructionKind::If(_, trueBranch, falseBranch) => {
                    let ifKey = Key::If(instruction.id);
                    let ifEnd = Node::new(
                        NodeKind::IfEnd,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(ifKey.clone(), ifEnd);
                    let block = f.getBlockById(*trueBranch);
                    let trueLast = self.processBlock(block, last.clone(), f);
                    if let Some(trueLast) = trueLast {
                        self.cfg.addEdge(Edge::new(trueLast, ifKey.clone()));
                    }
                    if let Some(falseBranch) = falseBranch {
                        let block = f.getBlockById(*falseBranch);
                        let falseLast = self.processBlock(block, last.clone(), f);
                        if let Some(falseLast) = falseLast {
                            self.cfg.addEdge(Edge::new(falseLast, ifKey.clone()));
                        }
                    }
                    last = Some(ifKey);
                }
                InstructionKind::BlockRef(id) => {
                    let block = f.getBlockById(*id);
                    last = self.processBlock(block, last, f);
                }
                InstructionKind::ValueRef(v, fields, _) => {
                    if let ValueKind::LoopVar(_) = v {
                        last = Some(self.processGenericInstruction(
                            instruction,
                            last,
                            NodeKind::Generic,
                        ));
                        continue;
                    }
                    let value = v.getValue();
                    let key = Key::Instruction(instruction.id);
                    let mut node = Node::new(
                        NodeKind::ValueRef,
                        instruction.ty.clone().expect("ty not found"),
                    );
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
                InstructionKind::StringLiteral(_) => {
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
                }
                InstructionKind::IntegerLiteral(_) => {
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
                }
                InstructionKind::CharLiteral(_) => {
                    last =
                        Some(self.processGenericInstruction(instruction, last, NodeKind::Generic));
                }
                InstructionKind::Loop(_, _, body) => {
                    let startKey = Key::LoopStart(instruction.id);
                    let start = Node::new(
                        NodeKind::LoopStart,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(startKey.clone(), start);
                    if let Some(last) = last.clone() {
                        let edge = Edge::new(last, startKey.clone());
                        self.cfg.addEdge(edge);
                    }
                    self.loopStarts.push(startKey.clone());
                    let endKey = Key::LoopEnd(instruction.id);
                    let end = Node::new(
                        NodeKind::LoopEnd,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(endKey.clone(), end);
                    self.loopEnds.push(endKey.clone());
                    let key = Key::Instruction(instruction.id);
                    let loopNode = Node::new(
                        NodeKind::Generic,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(key.clone(), loopNode);
                    let edge = Edge::new(startKey.clone(), key.clone());
                    self.cfg.addEdge(edge);
                    let loopBody = f.getBlockById(*body);
                    let loopLast = self.processBlock(loopBody, Some(key), f);
                    if let Some(loopLast) = loopLast {
                        let edge = Edge::new(loopLast, startKey.clone());
                        self.cfg.addEdge(edge);
                    }
                    self.loopStarts.pop();
                    self.loopEnds.pop();
                    last = Some(endKey);
                }
                InstructionKind::Continue(_, _) => {
                    let key = Key::Instruction(instruction.id);
                    let node = Node::new(
                        NodeKind::Generic,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(key.clone(), node);
                    if let Some(last) = last {
                        let edge = Edge::new(last, key.clone());
                        self.cfg.addEdge(edge);
                    }
                    let edge = Edge::new(key.clone(), self.loopStarts.last().unwrap().clone());
                    self.cfg.addEdge(edge);
                    last = None;
                }
                InstructionKind::Break(_, _) => {
                    let key = Key::Instruction(instruction.id);
                    let node = Node::new(
                        NodeKind::Generic,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(key.clone(), node);
                    if let Some(last) = last {
                        let edge = Edge::new(last, key.clone());
                        self.cfg.addEdge(edge);
                    }
                    let edge = Edge::new(key.clone(), self.loopEnds.last().unwrap().clone());
                    self.cfg.addEdge(edge);
                    last = None;
                }
                InstructionKind::Return(_) => {
                    if let Some(last) = last {
                        let edge = Edge::new(last, Key::End);
                        self.cfg.addEdge(edge);
                    }
                    last = None;
                }
                InstructionKind::Ref(_) => {
                    let prev = self.cfg.nodes.get_mut(&last.clone().unwrap()).unwrap();
                    let usage = prev.usage.clone().unwrap();
                    prev.usage = None;
                    let key = Key::Instruction(instruction.id);
                    let mut node = Node::new(
                        NodeKind::Generic,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    node.usage = Some(usage);
                    self.cfg.addNode(key.clone(), node);
                    if let Some(last) = last {
                        let edge = Edge::new(last, key.clone());
                        self.cfg.addEdge(edge);
                    }
                    last = Some(key);
                }
                InstructionKind::Drop(values) => {
                    let key = Key::DropKey(instruction.id, format!("[{}]", values.join(", ")));
                    let node = Node::new(
                        NodeKind::Generic,
                        instruction.ty.clone().expect("ty not found"),
                    );
                    self.cfg.addNode(key.clone(), node);
                    if let Some(last) = last {
                        let edge = Edge::new(last, key.clone());
                        self.cfg.addEdge(edge);
                    }
                    last = Some(key);
                }
            }
        }
        last
    }

    pub fn build(&mut self, f: &Function) {
        let block = &f.getFirstBlock();
        if let Some(last) = self.processBlock(block, None, f) {
            let edge = Edge::new(last, Key::End);
            self.cfg.addEdge(edge);
        }
        self.cfg.updateEdges();
    }
}
