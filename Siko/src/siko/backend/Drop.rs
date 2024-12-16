use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{
        Function::{BlockId, Function, InstructionKind, Variable, VariableName},
        Program::Program,
    },
    location::{
        Location::Location,
        Report::{Entry, Report, ReportContext},
    },
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut checker = DropChecker::new(f);
        let f = checker.process(ctx);
        result.functions.insert(name.clone(), f);
    }
    result
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct InstructionId {
    block: usize,
    id: usize,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Context<'a> {
    ctx: &'a ReportContext,
    live: BTreeSet<VariableName>,
    moved: BTreeMap<VariableName, Location>,
}

impl<'a> Context<'a> {
    fn new(ctx: &'a ReportContext) -> Context<'a> {
        Context {
            ctx: ctx,
            live: BTreeSet::new(),
            moved: BTreeMap::new(),
        }
    }

    fn exit(&self) {
        std::process::exit(1)
    }

    fn addLive(&mut self, var: &Variable) {
        //println!("addLive {}", var.value);
        self.live.insert(var.value.clone());
        self.moved.remove(&var.value);
    }

    fn addMove(&mut self, var: &Variable) {
        if var.getType().isReference() {
            return;
        }
        //println!("addMove {}", var.value);
        if let Some(movLoc) = self.moved.get(&var.value) {
            let slogan = format!("Value {} already moved", self.ctx.yellow(&var.value.userName()));
            let mut entries = Vec::new();
            entries.push(Entry::new(None, var.location.clone()));
            entries.push(Entry::new(Some(format!("NOTE: previous moved here")), movLoc.clone()));
            let r = Report::build(self.ctx, slogan, entries);
            r.print();
            self.exit();
        }
        self.moved.insert(var.value.clone(), var.location.clone());
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VisitedBlock<'a> {
    ctx: Context<'a>,
    id: BlockId,
}

pub struct DropChecker<'a> {
    function: &'a Function,
    visited: BTreeSet<VisitedBlock<'a>>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function) -> DropChecker<'a> {
        DropChecker {
            function: f,
            visited: BTreeSet::new(),
        }
    }

    fn process(&mut self, ctx: &'a ReportContext) -> Function {
        if self.function.body.is_some() {
            self.processBlock(BlockId::first(), Context::new(ctx));
        }
        self.function.clone()
    }

    fn processBlock(&mut self, blockId: BlockId, mut context: Context<'a>) {
        let visitedBlock = VisitedBlock {
            ctx: context.clone(),
            id: blockId,
        };
        if self.visited.contains(&visitedBlock) {
            return;
        }
        self.visited.insert(visitedBlock);
        let block = self.function.getBlockById(blockId);
        for (index, instruction) in block.instructions.iter().enumerate() {
            //println!("PROCESSING {}", instruction.kind);
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, _, args) => {
                    for arg in args {
                        context.addMove(arg);
                    }
                    context.addLive(dest);
                }
                InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::ValueRef(dest, src) => {
                    context.addMove(src);
                    context.addLive(dest);
                }
                InstructionKind::FieldRef(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::TupleIndex(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::Bind(dest, src, _) => {
                    context.addMove(src);
                    context.addLive(dest);
                }
                InstructionKind::Tuple(dest, args) => {
                    for arg in args {
                        context.addMove(arg);
                    }
                    context.addLive(dest);
                }
                InstructionKind::StringLiteral(dest, _) => {
                    context.addLive(dest);
                }
                InstructionKind::IntegerLiteral(dest, _) => {
                    context.addLive(dest);
                }
                InstructionKind::CharLiteral(dest, _) => {
                    context.addLive(dest);
                }
                InstructionKind::Return(_, _) => return,
                InstructionKind::Ref(dest, _) => {
                    context.addLive(dest);
                }
                InstructionKind::Drop(_) => {}
                InstructionKind::Jump(_, id) => {
                    self.processBlock(*id, context);
                    return;
                }
                InstructionKind::Assign(dest, src) => {
                    context.addMove(src);
                    context.addLive(dest);
                }
                InstructionKind::FieldAssign(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::DeclareVar(var) => {
                    context.addLive(var);
                }
                InstructionKind::Transform(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::EnumSwitch(root, cases) => {
                    context.addMove(root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::IntegerSwitch(root, cases) => {
                    context.addMove(root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::StringSwitch(root, cases) => {
                    context.addMove(root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
            }
        }
    }
}
