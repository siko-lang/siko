use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, ApplyVariable},
        Function::{BlockId, Function, Instruction, InstructionKind, Variable, VariableName},
        Program::Program,
        Substitution::VariableSubstitution,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
    },
    location::Report::{Entry, Report, ReportContext},
    qualifiedname::getCloneName,
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
struct SyntaxBlock {
    id: String,
    childBlocks: Vec<SyntaxBlock>,
}

impl SyntaxBlock {
    fn new(id: String) -> SyntaxBlock {
        SyntaxBlock {
            id: id,
            childBlocks: Vec::new(),
        }
    }

    fn addBlock(&mut self, block: SyntaxBlock) {
        if self.childBlocks.is_empty() {
            self.childBlocks.push(block);
        } else {
            self.childBlocks.last_mut().unwrap().addBlock(block);
        }
    }

    fn getCurrentBlockId(&self) -> String {
        if self.childBlocks.is_empty() {
            return format!("{}", self.id);
        } else {
            return format!("{}.{}", self.id, self.childBlocks.last().unwrap().getCurrentBlockId());
        }
    }

    fn endBlock(&mut self) {
        assert!(!self.childBlocks.is_empty());
        if self.childBlocks.last().unwrap().childBlocks.is_empty() {
            self.childBlocks.pop();
        } else {
            self.childBlocks.last_mut().unwrap().endBlock();
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Context {
    live: Vec<Variable>,
    moved: Vec<Usage>,
    rootBlock: SyntaxBlock,
}

enum Result {
    AlreadyMoved(Path, Usage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveKind {
    Fully,
    Partially,
    NotMoved,
}

struct DropList {
    paths: Vec<Path>,
}

impl DropList {
    fn new() -> DropList {
        DropList { paths: Vec::new() }
    }

    fn add(&mut self, path: Path) {
        self.paths.push(path);
    }
}

impl Display for DropList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.paths.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(", "))
    }
}

impl Context {
    fn new() -> Context {
        let rootBlock = SyntaxBlock::new(format!("0"));
        Context {
            live: Vec::new(),
            moved: Vec::new(),
            rootBlock,
        }
    }

    fn isLive(&self, var: &VariableName) -> bool {
        for v in &self.live {
            if v.value == *var {
                return true;
            }
        }
        false
    }

    fn addLive(&mut self, var: &Variable) {
        //println!("    addLive {} in block {}", var.value, self.rootBlock.getCurrentBlockId());
        if !self.live.contains(var) {
            self.live.push(var.clone());
        }
        self.moved.retain(|usage| usage.path.root.value != var.value);
    }

    fn removeSpecificMoveByRoot(&mut self, var: &Variable) {
        self.moved.retain(|usage| usage.path.root.value != var.value);
    }

    fn removeSpecificMove(&mut self, var: &Variable) {
        self.moved.retain(|usage| usage.var != *var);
    }

    fn isMoved(&self, path: &Path) -> MoveKind {
        for usage in &self.moved {
            if usage.path.same(path) {
                //println!("paths {} {}", usage.path, path,);
                if path.contains(&usage.path) {
                    return MoveKind::Fully;
                } else {
                    return MoveKind::Partially;
                }
            }
        }
        MoveKind::NotMoved
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
        for usage in &self.moved {
            //println!("checking {} and {}", path, currentPath);
            if usage.path.same(&currentPath) {
                return Some(Result::AlreadyMoved(currentPath.clone(), usage.clone()));
            }
        }
        //println!("addMove {}", currentPath);
        self.moved.push(Usage {
            var: var.clone(),
            path: currentPath,
        });
        return None;
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " live {}, moved {}, block {}",
            self.live.iter().map(|v| v.value.visibleName()).collect::<Vec<String>>().join(", "),
            self.moved.iter().map(|u| u.path.userPath()).collect::<Vec<String>>().join(", "),
            self.rootBlock.getCurrentBlockId()
        )
    }
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

    fn contains(&self, other: &Path) -> bool {
        if self.root.value != other.root.value {
            return false;
        }
        if self.items.len() < other.items.len() {
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
            write!(f, "{}", self.root.value)
        } else {
            write!(f, "{}.{}", self.root.value.visibleName(), self.items.join("."))
        }
    }
}

struct ValueInfo {
    var: Variable,
    block: String,
}

impl Display for ValueInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.var.value, self.block)
    }
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
    paths: BTreeMap<VariableName, Path>,
    implicitClones: BTreeSet<Variable>,
    values: BTreeMap<VariableName, ValueInfo>,
    dropLists: BTreeMap<String, DropList>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function, ctx: &'a ReportContext, program: &'a Program) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            function: f,
            program: program,
            visited: BTreeMap::new(),
            paths: BTreeMap::new(),
            implicitClones: BTreeSet::new(),
            values: BTreeMap::new(),
            dropLists: BTreeMap::new(),
        }
    }

    fn addImplicitClone(&mut self, result: &mut Function) {
        let mut nextVar = 1;
        let body = result.body.as_mut().expect("no body");
        for block in &mut body.blocks {
            let mut index = 0;
            loop {
                if index >= block.instructions.len() {
                    break;
                }
                let mut instruction = block.instructions[index].clone();
                let vars = instruction.kind.collectVariables();
                let mut instructionIndex = index;
                for var in vars {
                    if self.implicitClones.contains(&var) {
                        let mut implicitRefDest = var.clone();
                        implicitRefDest.value = VariableName::Local(format!("implicitRef"), nextVar);
                        nextVar += 1;
                        let ty = Type::Reference(Box::new(var.getType().clone()), None);
                        implicitRefDest.ty = Some(ty);
                        let kind = InstructionKind::Ref(implicitRefDest.clone(), var.clone());
                        let implicitRef = Instruction {
                            implicit: true,
                            kind: kind,
                            location: instruction.location.clone(),
                        };
                        block.instructions.insert(index, implicitRef);
                        instructionIndex += 1;
                        let mut implicitCloneDest = var.clone();
                        implicitCloneDest.value = VariableName::Local(format!("implicitClone"), nextVar);
                        nextVar += 1;
                        let mut varSwap = VariableSubstitution::new();
                        varSwap.add(var.clone(), implicitCloneDest.clone());
                        let kind = InstructionKind::FunctionCall(implicitCloneDest.clone(), getCloneName(), vec![implicitRefDest]);
                        let implicitRef = Instruction {
                            implicit: true,
                            kind: kind,
                            location: instruction.location.clone(),
                        };
                        instruction.kind = instruction.kind.applyVar(&varSwap);
                        block.instructions.insert(instructionIndex, implicitRef);
                        instructionIndex += 1;
                        self.implicitClones.remove(&var);
                    }
                }
                block.instructions[instructionIndex] = instruction;
                index += 1;
            }
        }
    }

    fn process(&mut self) -> Function {
        let mut result = self.function.clone();
        if self.function.body.is_some() {
            self.processBlock(BlockId::first(), Context::new());

            // println!("delcared values:");
            // for (_, info) in &self.values {
            //     println!("{}", info);
            // }

            self.addImplicitClone(&mut result);
            self.generateDrops(&mut result);
        }
        result
    }

    fn generateDrops(&mut self, result: &mut Function) {
        let mut nextDropVar = 0;
        let body = result.body.as_mut().expect("no body");
        for block in &mut body.blocks {
            let mut index = 0;
            loop {
                if index >= block.instructions.len() {
                    break;
                }
                let instruction = block.instructions[index].clone();
                let mut instructionIndex = index;
                if let InstructionKind::BlockEnd(id) = &instruction.kind {
                    if let Some(dropList) = self.dropLists.remove(&id.id) {
                        for path in &dropList.paths {
                            // create FieldRef instructionsfor the path and drop the  dest of the fieldref
                            let mut receiver = path.root.clone();
                            let mut currentTy = receiver.getType().clone();

                            for item in &path.items {
                                if let Some(className) = currentTy.getName() {
                                    if let Some(classDef) = self.program.getClass(&className) {
                                        let mut allocator = TypeVarAllocator::new();
                                        let classInstance = instantiateClass(&mut allocator, &classDef, &currentTy);
                                        for field in &classInstance.fields {
                                            if field.name == *item {
                                                currentTy = field.ty.clone();
                                                break;
                                            }
                                        }
                                    }
                                }

                                let mut dest = Variable {
                                    value: VariableName::Local(format!("drop"), nextDropVar),
                                    ty: Some(currentTy.clone()),
                                    location: instruction.location.clone(),
                                    index: 0,
                                };

                                nextDropVar += 1;

                                let kind = InstructionKind::FieldRef(dest.clone(), receiver, item.clone());
                                let fieldRef = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(instructionIndex, fieldRef);
                                instructionIndex += 1;
                                dest.index = 1;
                                receiver = dest;
                            }

                            let dropRes = Variable {
                                value: VariableName::Local(format!("autoDropRes"), nextDropVar),
                                ty: Some(Type::getUnitType()),
                                location: instruction.location.clone(),
                                index: 0,
                            };

                            nextDropVar += 1;

                            let kind = InstructionKind::Drop(dropRes, receiver.clone());
                            let drop = Instruction {
                                implicit: true,
                                kind: kind,
                                location: instruction.location.clone(),
                            };
                            //println!("Adding drop for {}", path);
                            block.instructions.insert(instructionIndex, drop);
                            instructionIndex += 1;
                        }
                    }
                }
                index += 1;
            }
        }
    }

    fn checkMove(&mut self, context: &mut Context, var: &Variable) {
        if let Some(Result::AlreadyMoved(currentPath, prevUsage)) = context.addMove(&self.paths, var) {
            if self.program.instanceResolver.isCopy(&prevUsage.var.getType().clone()) {
                self.implicitClones.insert(prevUsage.var.clone());
                context.removeSpecificMove(&prevUsage.var);
                context.addMove(&self.paths, var);
                return;
            }

            if prevUsage.var == *var {
                let slogan = format!("Value {} moved in previous iteration of loop", self.ctx.yellow(&currentPath.userPath()));
                //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
                let mut entries = Vec::new();
                entries.push(Entry::new(None, var.location.clone()));
                let r = Report::build(self.ctx, slogan, entries);
                r.print();
            } else {
                let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.userPath()));
                //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
                let mut entries = Vec::new();
                entries.push(Entry::new(None, var.location.clone()));
                entries.push(Entry::new(Some(format!("NOTE: previously moved here")), prevUsage.var.location.clone()));
                let r = Report::build(self.ctx, slogan, entries);
                r.print();
            }
            std::process::exit(1)
        }
    }

    fn declareValue(&mut self, var: &Variable, context: &mut Context) {
        context.addLive(var);
        self.values.insert(
            var.value.clone(),
            ValueInfo {
                var: var.clone(),
                block: context.rootBlock.getCurrentBlockId(),
            },
        );
    }

    fn processBlock(&mut self, blockId: BlockId, mut context: Context) {
        let contexts = self.visited.entry(blockId).or_insert(BTreeSet::new());
        if contexts.is_empty() {
            contexts.insert(context.clone());
        } else {
            let first = contexts.first().unwrap();
            context.rootBlock = first.rootBlock.clone();
            if !contexts.insert(context.clone()) {
                //println!("already visited {}", context);
                return;
            }
        }
        let block = self.function.getBlockById(blockId);
        for instruction in &block.instructions {
            //println!("PROCESSING {}", instruction.kind);
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, _, args) => {
                    for arg in args {
                        self.checkMove(&mut context, arg);
                    }
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::ValueRef(dest, src) => {
                    self.checkMove(&mut context, src);
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::FieldRef(dest, receiver, fieldName) => {
                    if let Some(path) = self.paths.get(&receiver.value) {
                        self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
                    } else {
                        self.paths.insert(dest.value.clone(), Path::new(receiver.clone()).add(fieldName.clone()));
                    }
                }
                InstructionKind::TupleIndex(dest, _, _) => {
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::Bind(dest, src, _) => {
                    self.checkMove(&mut context, src);
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::Tuple(dest, args) => {
                    for arg in args {
                        self.checkMove(&mut context, arg);
                    }
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::StringLiteral(dest, _) => {
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::IntegerLiteral(dest, _) => {
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::CharLiteral(dest, _) => {
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::Return(_, _) => return,
                InstructionKind::Ref(dest, _) => {
                    self.declareValue(dest, &mut context);
                }
                InstructionKind::Drop(_, _) => {}
                InstructionKind::Jump(_, id) => {
                    self.processBlock(*id, context);
                    return;
                }
                InstructionKind::Assign(dest, src) => {
                    self.checkMove(&mut context, src);
                    context.removeSpecificMoveByRoot(dest);
                    if !dest.getType().isReference() {
                        if context.isLive(&dest.value) {
                            let mut dropList = DropList::new();
                            self.dropPath(&Path::new(dest.clone()), &dest.getType(), &context, &mut dropList);
                            //println!("drop list {}", dropList);
                        }
                    }
                }
                InstructionKind::FieldAssign(dest, _, _) => {
                    context.addLive(dest);
                }
                InstructionKind::DeclareVar(_) => {}
                InstructionKind::Transform(dest, _, _) => {
                    self.declareValue(dest, &mut context);
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
                InstructionKind::BlockStart(name) => {
                    let block = SyntaxBlock::new(name.id.clone());
                    context.rootBlock.addBlock(block);
                    //println!("block start {}", context.rootBlock.getCurrentBlockId());
                }
                InstructionKind::BlockEnd(endId) => {
                    //println!("block end {}", context.rootBlock.getCurrentBlockId());
                    let current = context.rootBlock.getCurrentBlockId();
                    let mut dropList = DropList::new();
                    self.dropValues(&mut context, current, &mut dropList);
                    self.dropLists.insert(endId.id.clone(), dropList);
                    context.rootBlock.endBlock();
                }
            }
        }
    }

    fn dropPath(&self, rootPath: &Path, ty: &Type, context: &Context, dropList: &mut DropList) {
        match context.isMoved(&&rootPath) {
            MoveKind::NotMoved => {
                //println!("not moved - drop {}", rootPath);
                dropList.add(rootPath.clone());
            }
            MoveKind::Partially => {
                //println!("partially moved {}", rootPath);
                //println!("already moved (maybe partially?) {}", rootPath);
                if let Some(className) = ty.getName() {
                    if let Some(classDef) = self.program.getClass(&className) {
                        let mut allocator = TypeVarAllocator::new();
                        let classInstance = instantiateClass(&mut allocator, &classDef, ty);
                        for field in &classInstance.fields {
                            let path = rootPath.add(field.name.clone());
                            self.dropPath(&path, &field.ty, context, dropList);
                        }
                    }
                }
            }
            MoveKind::Fully => {
                //println!("already moved {}", rootPath);
            }
        }
    }

    fn dropValues(&mut self, context: &mut Context, block: String, dropList: &mut DropList) {
        //println!("Dropping in {}", block);
        // for usage in &context.moved {
        //     println!("move {}", usage.path);
        // }
        for var in &context.live {
            if let Some(info) = self.values.get(&var.value) {
                if info.block == block {
                    //println!("live {}", var.value);
                    if !var.getType().isReference() {
                        self.dropPath(&Path::new(var.clone()), &var.getType(), context, dropList);
                    }
                }
            }
        }
        //println!("---");
    }
}
