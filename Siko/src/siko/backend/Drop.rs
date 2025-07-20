use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    hir::{
        Apply::{instantiateStruct, ApplyVariable},
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::InstructionKind,
        Program::Program,
        Substitution::VariableSubstitution,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Variable::{Variable, VariableName},
    },
    location::{
        Location::Location,
        Report::{Entry, Report, ReportContext},
    },
    qualifiedname::getCloneFnName,
    util::DependencyProcessor,
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut checker = DropChecker::new(f, ctx, &program);
        //println!("Checking drops for {}", name);
        checker.processDeps();
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UsageKind {
    Move,
    Ref,
}

impl Display for UsageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageKind::Move => write!(f, "move"),
            UsageKind::Ref => write!(f, "ref"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Usage {
    var: Variable,
    path: Path,
    kind: UsageKind,
}

impl Usage {
    fn isMove(&self) -> bool {
        self.kind == UsageKind::Move
    }
}

impl Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.kind, self.path, self.var)
    }
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
    usages: Vec<Usage>,
    rootBlock: SyntaxBlock,
}

enum Result {
    AlreadyMoved(Path, Usage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MoveKind {
    Fully(Variable),
    Partially,
    NotMoved,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PossibleCollision {
    first: Variable,
    second: Variable,
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
        write!(
            f,
            "[{}]",
            self.paths
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Context {
    fn new() -> Context {
        let rootBlock = SyntaxBlock::new(format!("0"));
        Context {
            live: Vec::new(),
            usages: Vec::new(),
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
        // println!(
        //     "    addLive {} in block {}",
        //     var.value,
        //     self.rootBlock.getCurrentBlockId()
        // );
        if !self.live.contains(var) {
            self.live.push(var.clone());
        }
        self.usages.retain(|usage| usage.path.root.value != var.value);
    }

    fn removeSpecificMoveByRoot(&mut self, var: &Variable) {
        self.usages.retain(|usage| usage.path.root.value != var.value);
    }

    fn removeSpecificMoveByPath(&mut self, path: &Path) {
        self.usages.retain(|usage| !usage.path.contains(path));
    }

    fn removeSpecificMove(&mut self, var: &Variable) {
        self.usages.retain(|usage| usage.var != *var);
    }

    fn isMoved(&self, path: &Path) -> MoveKind {
        for usage in &self.usages {
            if usage.path.shares_prefix_with(path) && usage.isMove() {
                //println!("paths {} {}", usage.path, path,);
                if path.contains(&usage.path) {
                    return MoveKind::Fully(usage.var.clone());
                } else {
                    return MoveKind::Partially;
                }
            }
        }
        MoveKind::NotMoved
    }

    fn addUsage(
        &mut self,
        paths: &BTreeMap<VariableName, Path>,
        var: &Variable,
        kind: UsageKind,
        collisions: &mut BTreeSet<PossibleCollision>,
        usages: &mut BTreeMap<Variable, Usage>,
    ) {
        //println!("    addUsage {} {}", var, kind);
        let currentPath = if let Some(path) = paths.get(&var.value) {
            path.clone()
        } else {
            Path::new(var.clone())
        };

        let mut alreadyAdded = false;

        for usage in &self.usages {
            //println!("checking {}/{} and {}/{}", usage.var, usage.path, currentPath.root, var);
            if usage.path.shares_prefix_with(&currentPath) && usage.isMove() {
                collisions.insert(PossibleCollision {
                    first: usage.var.clone(),
                    second: var.clone(),
                });
            }
            if usage.var == *var {
                alreadyAdded = true;
            }
        }

        if alreadyAdded {
            //println!("    already added");
            return;
        }

        let usage = Usage {
            var: var.clone(),
            path: currentPath,
            kind: kind,
        };
        //println!("    addUsage {}", usage);
        self.usages.push(usage.clone());
        usages.insert(var.clone(), usage);
    }

    fn merge(&mut self, terminal_context: &Context) {
        for var in &terminal_context.live {
            self.addLive(var);
        }
        for usage in &terminal_context.usages {
            if self.usages.contains(usage) {
                continue;
            }
            self.usages.push(usage.clone());
        }
        self.rootBlock = terminal_context.rootBlock.clone();
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " live {}, moved {}, block {}",
            self.live
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.usages
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<String>>()
                .join(", "),
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

    fn shares_prefix_with(&self, other: &Path) -> bool {
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

    fn same(&self, other: &Path) -> bool {
        if self.root.value != other.root.value {
            return false;
        }
        if self.items.len() != other.items.len() {
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

fn getUsageKind(var: &Variable) -> UsageKind {
    let ty = var.getType();
    if ty.isReference() || ty.isPtr() {
        UsageKind::Ref
    } else {
        UsageKind::Move
    }
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
    paths: BTreeMap<VariableName, Path>,
    implicitClones: BTreeSet<Variable>,
    values: BTreeMap<VariableName, ValueInfo>,
    dropLists: BTreeMap<String, DropList>,
    terminalContexts: BTreeMap<BlockId, Context>,
    collisions: BTreeSet<PossibleCollision>,
    usages: BTreeMap<Variable, Usage>,
    assigns: BTreeMap<Variable, DropList>,
    returns: BTreeMap<Variable, DropList>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function, ctx: &'a ReportContext, program: &'a Program) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            visited: BTreeMap::new(),
            paths: BTreeMap::new(),
            implicitClones: BTreeSet::new(),
            values: BTreeMap::new(),
            dropLists: BTreeMap::new(),
            terminalContexts: BTreeMap::new(),
            collisions: BTreeSet::new(),
            usages: BTreeMap::new(),
            assigns: BTreeMap::new(),
            returns: BTreeMap::new(),
        }
    }

    fn addImplicitClone(&mut self) {
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for block in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(block);
            loop {
                if let Some(mut instruction) = builder.getInstruction() {
                    let vars = instruction.kind.collectVariables();
                    let mut kinds = Vec::new();
                    for var in vars {
                        if self.implicitClones.contains(&var) {
                            let mut implicitRefDest = self
                                .bodyBuilder
                                .createTempValue(VariableName::DropImplicitCloneRef, instruction.location.clone());
                            let ty = Type::Reference(Box::new(var.getType().clone()), None);
                            implicitRefDest.ty = Some(ty);
                            let implicitRefKind = InstructionKind::Ref(implicitRefDest.clone(), var.clone());

                            let mut implicitCloneDest = self
                                .bodyBuilder
                                .createTempValue(VariableName::DropImplicitClone, instruction.location.clone());
                            implicitCloneDest.ty = var.ty.clone();
                            let mut varSwap = VariableSubstitution::new();
                            varSwap.add(var.clone(), implicitCloneDest.clone());
                            let implicitCloneKind = InstructionKind::FunctionCall(
                                implicitCloneDest.clone(),
                                getCloneFnName(),
                                vec![implicitRefDest],
                            );
                            instruction.kind = instruction.kind.applyVar(&varSwap);
                            kinds.push(implicitCloneKind);
                            kinds.push(implicitRefKind);
                            self.implicitClones.remove(&var);
                        }
                    }
                    builder.replaceInstruction(instruction.kind, instruction.location.clone());
                    for k in kinds {
                        builder.addInstruction(k, instruction.location.clone());
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    fn processDeps(&mut self) {
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        let mut blockDeps = BTreeMap::new();
        for blockId in &allblocksIds {
            blockDeps.insert(*blockId, Vec::new());
        }
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    builder.step();
                    match instruction.kind {
                        InstructionKind::Jump(_, id, _) => {
                            blockDeps.entry(id).or_default().push(blockId);
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for case in cases {
                                blockDeps.entry(case.branch).or_default().push(blockId);
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for case in cases {
                                blockDeps.entry(case.branch).or_default().push(blockId);
                            }
                        }
                        InstructionKind::StringSwitch(_, cases) => {
                            for case in cases {
                                blockDeps.entry(case.branch).or_default().push(blockId);
                            }
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }
        let groups = DependencyProcessor::processDependencies(&mut blockDeps);
        //println!("all deps {:?}", blockDeps);
        //println!("groups {:?}", groups);
        for g in &groups {
            //println!("groups {:?}", g);
            self.processGroup(&g.items, &blockDeps);
        }
    }

    fn processCollision(&mut self) {
        let mut failed = false;
        for collision in &self.collisions {
            let mut prevUsage = self.usages.get(&collision.first).unwrap().clone();
            let currentUsage = self.usages.get(&collision.second).unwrap().clone();

            if !prevUsage.isMove() {
                continue;
            }

            //println!("collision {} {}", prevUsage.var, currentUsage.var);
            if self.program.instanceResolver.isCopy(prevUsage.var.getType()) {
                //println!("implict clone for {}", prevUsage.var);
                self.implicitClones.insert(prevUsage.var.clone());
                prevUsage.kind = UsageKind::Ref;
                self.usages.insert(prevUsage.var.clone(), prevUsage);
                continue;
            }

            failed = true;

            if prevUsage.var == currentUsage.var {
                let slogan = format!(
                    "Value {} moved in previous iteration of loop",
                    self.ctx.yellow(&currentUsage.path.userPath())
                );
                //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
                let mut entries = Vec::new();
                entries.push(Entry::new(None, currentUsage.var.location.clone()));
                let r = Report::build(self.ctx, slogan, entries);
                r.print();
            } else {
                let slogan = format!("Value {} already moved", self.ctx.yellow(&currentUsage.path.userPath()));
                //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
                let mut entries = Vec::new();
                entries.push(Entry::new(None, currentUsage.var.location.clone()));
                entries.push(Entry::new(
                    Some(format!("NOTE: previously moved here")),
                    prevUsage.var.location.clone(),
                ));
                let r = Report::build(self.ctx, slogan, entries);
                r.print();
            }
        }

        if failed {
            std::process::exit(1);
        }
    }

    fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        self.processBlock(BlockId::first(), Context::new());
        self.processCollision();
        self.addImplicitClone();
        self.generateDrops();
        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
    }

    fn processDropList(&mut self, dropList: DropList, builder: &mut BlockBuilder, location: Location) {
        let mut kinds = Vec::new();
        for path in &dropList.paths {
            // create FieldRef instructionsfor the path and drop the dest of the fieldref
            let mut receiver = path.root.clone();
            let mut currentTy = receiver.getType().clone();

            for item in &path.items {
                if let Some(structName) = currentTy.getName() {
                    if let Some(structDef) = self.program.getStruct(&structName) {
                        let mut allocator = TypeVarAllocator::new();
                        let structInstance = instantiateStruct(&mut allocator, &structDef, &currentTy);
                        for field in &structInstance.fields {
                            if field.name == *item {
                                currentTy = field.ty.clone();
                                break;
                            }
                        }
                    }
                }

                let mut dest = self
                    .bodyBuilder
                    .createTempValue(VariableName::DropVar, location.clone());
                dest.ty = Some(currentTy.clone());

                let fieldRefKind = InstructionKind::FieldRef(dest.clone(), receiver, item.clone());
                kinds.push(fieldRefKind);
                dest.index = 1;
                receiver = dest;
            }

            let mut dropRes = self
                .bodyBuilder
                .createTempValue(VariableName::AutoDropResult, location.clone());
            dropRes.ty = Some(Type::getUnitType());

            let dropKind = InstructionKind::Drop(dropRes, receiver.clone());

            kinds.push(dropKind);
        }
        for k in kinds.into_iter().rev() {
            builder.addInstruction(k, location.clone());
        }
    }

    fn generateDrops(&mut self) {
        let allBlocks = self.bodyBuilder.getAllBlockIds();
        for blockId in allBlocks {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match &instruction.kind {
                        InstructionKind::Assign(dest, _) => {
                            if let Some(dropList) = self.assigns.remove(&dest) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        InstructionKind::BlockEnd(id) => {
                            if let Some(dropList) = self.dropLists.remove(&id.id) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        InstructionKind::Return(dest, _) => {
                            if let Some(dropList) = self.returns.remove(&dest) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    fn checkMove(&mut self, context: &mut Context, var: &Variable, kind: UsageKind) {
        context.addUsage(&self.paths, var, kind, &mut self.collisions, &mut self.usages)
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

    fn processBlock(&mut self, blockId: BlockId, mut context: Context) -> bool {
        let contexts = self.visited.entry(blockId).or_insert(BTreeSet::new());
        if contexts.is_empty() {
            contexts.insert(context.clone());
        } else {
            let first = contexts.first().unwrap();
            context.rootBlock = first.rootBlock.clone();
            if !contexts.insert(context.clone()) {
                //println!("already visited {}", context);
                return false;
            }
        }
        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            if let Some(instruction) = builder.getInstruction() {
                //println!("PROCESSING {}", instruction.kind);
                match &instruction.kind {
                    InstructionKind::FunctionCall(dest, _, args) => {
                        for arg in args {
                            self.checkMove(&mut context, arg, getUsageKind(arg));
                        }
                        self.declareValue(dest, &mut context);
                    }
                    InstructionKind::Converter(dest, source) => {
                        unreachable!("converter in Drop checker");
                    }
                    InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
                    InstructionKind::DynamicFunctionCall(_, _, _) => {}
                    InstructionKind::ValueRef(dest, src) => {
                        self.checkMove(&mut context, src, getUsageKind(src));
                        self.declareValue(dest, &mut context);
                    }
                    InstructionKind::FieldRef(dest, receiver, fieldName) => {
                        if let Some(path) = self.paths.get(&receiver.value) {
                            self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
                        } else {
                            self.paths
                                .insert(dest.value.clone(), Path::new(receiver.clone()).add(fieldName.clone()));
                        }
                    }
                    InstructionKind::TupleIndex(dest, src, index) => {
                        let fieldName = format!("{}", index);
                        if let Some(path) = self.paths.get(&src.value) {
                            self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
                        } else {
                            self.paths
                                .insert(dest.value.clone(), Path::new(src.clone()).add(fieldName.clone()));
                        }
                    }
                    InstructionKind::Bind(dest, src, _) => {
                        self.checkMove(&mut context, src, getUsageKind(src));
                        self.declareValue(dest, &mut context);
                    }
                    InstructionKind::Tuple(dest, args) => {
                        for arg in args {
                            self.checkMove(&mut context, arg, getUsageKind(arg));
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
                    InstructionKind::Return(dest, src) => {
                        self.checkMove(&mut context, src, getUsageKind(src));
                        let mut dropList = DropList::new();
                        self.dropValues(&mut context, format!("0"), &mut dropList);
                        //println!("drop list {}", dropList);
                        let mut collisions = BTreeSet::new();
                        for p in &dropList.paths {
                            context.addUsage(&self.paths, &p.root, UsageKind::Move, &mut collisions, &mut self.usages);
                        }
                        self.returns.insert(dest.clone(), dropList);
                    }
                    InstructionKind::Ref(dest, value) => {
                        self.checkMove(&mut context, value, UsageKind::Ref);
                        self.declareValue(dest, &mut context);
                    }
                    InstructionKind::Drop(_, _) => {}
                    InstructionKind::Jump(_, _, _) => {}
                    InstructionKind::Assign(dest, src) => {
                        self.checkMove(&mut context, src, getUsageKind(src));
                        if !dest.getType().isReference() {
                            if context.isLive(&dest.value) {
                                let mut dropList = DropList::new();
                                self.dropPath(&Path::new(dest.clone()), &dest.getType(), &context, &mut dropList);
                                //println!("drop list {}", dropList);
                                self.assigns.insert(dest.clone(), dropList);
                            }
                        }
                        context.removeSpecificMoveByRoot(dest);
                    }
                    InstructionKind::FieldAssign(dest, src, fields) => {
                        self.checkMove(&mut context, src, getUsageKind(src));
                        let ty = fields.last().unwrap().ty.clone().unwrap();
                        if !ty.isReference() {
                            let mut dropList = DropList::new();
                            let mut path = Path::new(dest.clone());
                            for f in fields {
                                path = path.add(f.name.clone());
                            }
                            let mut parent = path.clone();
                            parent.items.pop();
                            for usage in &context.usages {
                                //println!("checking {}/{} and {}/{}", usage.var, usage.path, currentPath.root, var);
                                if usage.path.shares_prefix_with(&parent) && !usage.path.same(&path) && usage.isMove() {
                                    self.collisions.insert(PossibleCollision {
                                        first: usage.var.clone(),
                                        second: dest.clone(),
                                    });
                                }
                            }
                            self.usages.insert(
                                dest.clone(),
                                Usage {
                                    var: dest.clone(),
                                    path: parent.clone(),
                                    kind: UsageKind::Move,
                                },
                            );
                            self.dropPath(&path, &dest.getType(), &context, &mut dropList);
                            //println!("drop list {}", dropList);
                            context.removeSpecificMoveByPath(&path);
                        }
                    }
                    InstructionKind::DeclareVar(_) => {}
                    InstructionKind::Transform(dest, _, _) => {
                        self.declareValue(dest, &mut context);
                    }
                    InstructionKind::EnumSwitch(root, _) => {
                        self.checkMove(&mut context, root, getUsageKind(root));
                    }
                    InstructionKind::IntegerSwitch(root, _) => {
                        self.checkMove(&mut context, root, getUsageKind(root));
                    }
                    InstructionKind::StringSwitch(root, _) => {
                        self.checkMove(&mut context, root, getUsageKind(root));
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
                        //println!("drop list {}", dropList);
                        let mut collisions = BTreeSet::new();
                        for p in &dropList.paths {
                            context.addUsage(&self.paths, &p.root, UsageKind::Move, &mut collisions, &mut self.usages);
                        }
                        self.dropLists.insert(endId.id.clone(), dropList);
                        context.rootBlock.endBlock();
                    }
                }
                builder.step();
            } else {
                break;
            }
        }
        let old = self.terminalContexts.insert(blockId, context.clone());
        match old {
            Some(oldContext) => oldContext != context,
            None => {
                return true;
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
                if let Some(structName) = ty.getName() {
                    if let Some(structDef) = self.program.getStruct(&structName) {
                        let mut allocator = TypeVarAllocator::new();
                        let structInstance = instantiateStruct(&mut allocator, &structDef, ty);
                        for field in &structInstance.fields {
                            let path = rootPath.add(field.name.clone());
                            self.dropPath(&path, &field.ty, context, dropList);
                        }
                    }
                }
            }
            MoveKind::Fully(var) => {
                //println!("already moved {} by {}", rootPath, var);
            }
        }
    }

    fn dropValues(&mut self, context: &mut Context, block: String, dropList: &mut DropList) {
        //println!("Dropping in {}", block);
        for var in &context.live {
            if let Some(info) = self.values.get(&var.value) {
                if info.block.starts_with(&block) {
                    //println!("live {}", var.value);
                    if !var.getType().isReference() {
                        self.dropPath(&Path::new(var.clone()), &var.getType(), context, dropList);
                    }
                }
            }
        }
        //println!("---");
    }

    fn processGroup(&mut self, items: &Vec<BlockId>, blockDeps: &BTreeMap<BlockId, Vec<BlockId>>) {
        loop {
            let mut changed = false;
            for item in items {
                //println!("processing block {}", item);
                let deps = blockDeps.get(item).expect("deps not found");
                let mut mergedContext = Context::new();
                let mut empty = !deps.is_empty();
                for dep in deps {
                    if let Some(terminalContext) = self.terminalContexts.get(dep) {
                        mergedContext.merge(terminalContext);
                        empty = false;
                    } else {
                        if !items.contains(dep) {
                            panic!("terminal context not found for {}", dep);
                        }
                    }
                }
                if empty {
                    continue;
                }
                //println!("merged context {}", mergedContext);
                if self.processBlock(item.clone(), mergedContext) {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
}
