use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

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
    moved: BTreeMap<Path, Location>,
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
        self.moved = self
            .moved
            .clone()
            .into_iter()
            .filter(|(path, _)| path.items[0] != var.value.to_string())
            .collect();
    }

    fn addMove(&mut self, paths: &BTreeMap<VariableName, Path>, var: &Variable) {
        if var.getType().isReference() {
            return;
        }
        let currentPath = if let Some(path) = paths.get(&var.value) {
            path.clone()
        } else {
            Path::new().add(var.value.to_string())
        };
        //println!("addmove {}", currentPath);
        for (path, movLoc) in &self.moved {
            //println!("checking {} and {}", path, currentPath);
            if path.parent(&currentPath) {
                let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.userPath()));
                let mut entries = Vec::new();
                entries.push(Entry::new(None, var.location.clone()));
                entries.push(Entry::new(Some(format!("NOTE: previously moved here")), movLoc.clone()));
                let r = Report::build(self.ctx, slogan, entries);
                r.print();
                self.exit();
            }
        }
        self.moved.insert(currentPath, var.location.clone());
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VisitedBlock<'a> {
    ctx: Context<'a>,
    id: BlockId,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Path {
    items: Vec<String>,
}

impl Path {
    fn new() -> Path {
        Path { items: Vec::new() }
    }

    fn add(&self, item: String) -> Path {
        let mut p = self.clone();
        p.items.push(item);
        p
    }

    fn userPath(&self) -> String {
        self.items.join(".")
    }

    fn parent(&self, other: &Path) -> bool {
        if self.items.len() > other.items.len() {
            return false;
        }
        let otherItems = &other.items[0..self.items.len()];
        self.items == otherItems
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({})", self.items.join(","))
    }
}

pub struct DropChecker<'a> {
    function: &'a Function,
    visited: BTreeSet<VisitedBlock<'a>>,
    paths: BTreeMap<VariableName, Path>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function) -> DropChecker<'a> {
        DropChecker {
            function: f,
            visited: BTreeSet::new(),
            paths: BTreeMap::new(),
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
                        context.addMove(&self.paths, arg);
                    }
                    context.addLive(dest);
                }
                InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::ValueRef(dest, src) => {
                    context.addMove(&self.paths, src);
                    context.addLive(dest);
                }
                InstructionKind::FieldRef(dest, receiver, fieldName) => {
                    if let Some(path) = self.paths.get(&receiver.value) {
                        self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
                    } else {
                        self.paths
                            .insert(dest.value.clone(), Path::new().add(receiver.value.to_string()).add(fieldName.clone()));
                    }
                    //println!("{}.{}", receiver, fieldName);
                    context.addLive(dest);
                }
                InstructionKind::TupleIndex(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::Bind(dest, src, _) => {
                    context.addMove(&self.paths, src);
                    context.addLive(dest);
                }
                InstructionKind::Tuple(dest, args) => {
                    for arg in args {
                        context.addMove(&self.paths, arg);
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
                InstructionKind::Ref(dest, src) => {
                    context.addLive(dest);
                }
                InstructionKind::Drop(_) => {}
                InstructionKind::Jump(_, id) => {
                    self.processBlock(*id, context);
                    return;
                }
                InstructionKind::Assign(dest, src) => {
                    context.addMove(&self.paths, src);
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
                    context.addMove(&self.paths, root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::IntegerSwitch(root, cases) => {
                    context.addMove(&self.paths, root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::StringSwitch(root, cases) => {
                    context.addMove(&self.paths, root);
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
