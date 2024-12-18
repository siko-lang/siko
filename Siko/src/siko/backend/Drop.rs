use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    hir::{
        Function::{BlockId, Function, InstructionKind, Variable, VariableName},
        InstanceResolver::ResolutionResult,
        Program::Program,
        TypeVarAllocator::TypeVarAllocator,
    },
    location::Report::{Entry, Report, ReportContext},
    qualifiedname::getCopyName,
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut checker = DropChecker::new(f, ctx, &program);
        let f = checker.process();
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
struct Usage {
    var: Variable,
    path: Path,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Context {
    live: BTreeSet<VariableName>,
    moved: Vec<Usage>,
}

enum Result {
    AlreadyMoved(Path, Usage),
}

impl Context {
    fn new() -> Context {
        Context {
            live: BTreeSet::new(),
            moved: Vec::new(),
        }
    }

    fn addLive(&mut self, var: &Variable) {
        //println!("addLive {}", var.value);
        self.live.insert(var.value.clone());
        self.moved.retain(|usage| usage.path.root.value != var.value);
    }

    fn removeSpecificMove(&mut self, var: &Variable) {
        self.moved.retain(|usage| usage.var != *var);
    }

    fn addMove(&mut self, paths: &BTreeMap<VariableName, Path>, var: &Variable) -> Option<Result> {
        if var.getType().isReference() {
            return None;
        }
        let currentPath = if let Some(path) = paths.get(&var.value) {
            path.clone()
        } else {
            Path::new(var.clone())
        };
        //println!("addmove {}", currentPath);
        for usage in &self.moved {
            //println!("checking {} and {}", path, currentPath);
            if usage.path.same(&currentPath) {
                return Some(Result::AlreadyMoved(currentPath.clone(), usage.clone()));
            }
        }
        self.moved.push(Usage {
            var: var.clone(),
            path: currentPath,
        });
        return None;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VisitedBlock {
    ctx: Context,
    id: BlockId,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Path {
    root: Variable,
    items: Vec<String>,
}

impl Path {
    fn new(root: Variable) -> Path {
        Path {
            root: root,
            items: Vec::new(),
        }
    }

    fn add(&self, item: String) -> Path {
        let mut p = self.clone();
        p.items.push(item);
        p
    }

    fn userPath(&self) -> String {
        if self.items.is_empty() {
            self.root.value.visibleName()
        } else {
            format!("{}.{}", self.root.value.visibleName(), self.items.join("."))
        }
    }

    fn same(&self, other: &Path) -> bool {
        if self.root.value != other.root.value {
            return false;
        }
        for (i1, i2) in self.items.iter().zip(other.items.iter()) {
            if i1 != i2 {
                return false;
            }
        }
        true
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            writeln!(f, "{}", self.root.value)
        } else {
            writeln!(f, "{}.{}", self.root.value.visibleName(), self.items.join("."))
        }
    }
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeSet<VisitedBlock>,
    paths: BTreeMap<VariableName, Path>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function, ctx: &'a ReportContext, program: &'a Program) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            function: f,
            program: program,
            visited: BTreeSet::new(),
            paths: BTreeMap::new(),
        }
    }

    fn process(&mut self) -> Function {
        if self.function.body.is_some() {
            self.processBlock(BlockId::first(), Context::new());
        }
        self.function.clone()
    }

    fn checkMove(&mut self, context: &mut Context, var: &Variable) {
        if let Some(Result::AlreadyMoved(currentPath, prevUsage)) = context.addMove(&self.paths, var) {
            if let Some(instances) = self.program.instanceResolver.lookupInstances(&getCopyName()) {
                let mut allocator = TypeVarAllocator::new();
                let result = instances.find(&mut allocator, &vec![prevUsage.var.getType().clone()]);
                if let ResolutionResult::Winner(_) = result {
                    //println!("Copy found for {}", prevUsage.var);
                    context.removeSpecificMove(&prevUsage.var);
                    context.addMove(&self.paths, var);
                    return;
                }
            }

            let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.userPath()));
            //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
            let mut entries = Vec::new();
            entries.push(Entry::new(None, var.location.clone()));
            entries.push(Entry::new(Some(format!("NOTE: previously moved here")), prevUsage.var.location.clone()));
            let r = Report::build(self.ctx, slogan, entries);
            r.print();
            std::process::exit(1)
        }
    }

    fn processBlock(&mut self, blockId: BlockId, mut context: Context) {
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
                        self.checkMove(&mut context, arg);
                    }
                    context.addLive(dest);
                }
                InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::ValueRef(dest, src) => {
                    self.checkMove(&mut context, src);
                    context.addLive(dest);
                }
                InstructionKind::FieldRef(dest, receiver, fieldName) => {
                    if let Some(path) = self.paths.get(&receiver.value) {
                        self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
                    } else {
                        self.paths.insert(dest.value.clone(), Path::new(receiver.clone()).add(fieldName.clone()));
                    }
                    context.addLive(dest);
                }
                InstructionKind::TupleIndex(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::Bind(dest, src, _) => {
                    self.checkMove(&mut context, src);
                    context.addLive(dest);
                }
                InstructionKind::Tuple(dest, args) => {
                    for arg in args {
                        self.checkMove(&mut context, arg);
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
                    self.checkMove(&mut context, src);
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
                    self.checkMove(&mut context, root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::IntegerSwitch(root, cases) => {
                    self.checkMove(&mut context, root);
                    for case in cases {
                        self.processBlock(case.branch, context.clone());
                    }
                }
                InstructionKind::StringSwitch(root, cases) => {
                    self.checkMove(&mut context, root);
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
