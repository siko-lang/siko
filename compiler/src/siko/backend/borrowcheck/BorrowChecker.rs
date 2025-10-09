use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{
    backend::{
        borrowcheck::{
            functionprofiles::{
                FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
            },
            DataGroups::{DataGroups, ExtendedType},
        },
        path::SimplePath::{buildSegments, SimplePath},
    },
    hir::{
        Block::BlockId, BlockGroupBuilder::BlockGroupBuilder, Function::Function, Instruction::InstructionKind,
        Program::Program, Type::Type, Variable::Variable,
    },
    location::{
        Location::Location,
        Report::{Entry, Report, ReportContext},
    },
    qualifiedname::QualifiedName,
    util::Runner::Runner,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path {
    pub p: SimplePath,
}

impl Path {
    pub fn userVisible(&self) -> String {
        let mut s = self.p.root.visibleName();
        for item in &self.p.items {
            s.push('.');
            s.push_str(&item.to_string());
        }
        s
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p)
    }
}

struct BorrowInfo {
    pub location: Location,
}

struct BorrowSet {
    paths: BTreeMap<Path, BorrowInfo>,
}

pub struct BorrowChecker<'a> {
    ctx: &'a ReportContext,
    borrows: BTreeMap<Type, BorrowSet>,
    profileBuilder: FunctionProfileBuilder<'a>,
    blockEnvs: BTreeMap<BlockId, Environment>,
    links: BTreeMap<Type, Type>,
    traceEnabled: bool,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        f: &'a Function,
        program: &'a Program,
        dataGroups: &'a DataGroups<'a>,
        profileStore: &'a mut FunctionProfileStore,
        functionGroup: Vec<QualifiedName>,
        runner: Runner,
    ) -> BorrowChecker<'a> {
        let traceEnabled = runner.getConfig().dumpCfg.borrowCheckerTraceEnabled;
        BorrowChecker {
            ctx,
            borrows: BTreeMap::new(),
            profileBuilder: FunctionProfileBuilder::new(f, program, dataGroups, profileStore, functionGroup, runner),
            blockEnvs: BTreeMap::new(),
            links: BTreeMap::new(),
            traceEnabled: traceEnabled,
        }
    }

    pub fn process(&mut self) {
        if self.profileBuilder.f.body.is_none() {
            return;
        }
        if self.traceEnabled {
            println!("Borrow checking function: {}", self.profileBuilder.f.name);
            println!("Function profile {}", self.profileBuilder.f);
        }
        self.profileBuilder.process(false);
        if self.traceEnabled {
            println!("{:?}", self.profileBuilder.profile.links);
            // TODO: handle links
        }
        for link in &self.profileBuilder.profile.links {
            self.links.insert(link.from.clone(), link.to.clone());
        }
        let blockGroupBuilder = BlockGroupBuilder::new(self.profileBuilder.f);
        let groupInfo = blockGroupBuilder.process();
        //println!("Block groups: {:?}", groupInfo.groups);
        for group in groupInfo.groups {
            //println!("Processing block group {:?}", group.items);
            let mut queue = Vec::new();
            for blockId in &group.items {
                queue.push(blockId.clone());
                self.blockEnvs.entry(blockId.clone()).or_insert_with(Environment::new);
            }
            while let Some(blockId) = queue.pop() {
                let entryEnv = self.getEnvForBlock(blockId.clone());
                let env = entryEnv.snapshot();
                let jumpTargets = self.processBlock(blockId.clone(), env);
                for (target, targetEnv) in jumpTargets {
                    let mut updated = false;
                    let entry = self.blockEnvs.entry(target.clone()).or_insert_with(|| {
                        updated = true;
                        Environment::new()
                    });
                    if entry.merge(&targetEnv) {
                        updated = true;
                    }
                    if updated && group.items.contains(&target) {
                        queue.push(target);
                    }
                }
            }
        }
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> Environment {
        let env = self.blockEnvs.entry(blockId).or_insert_with(Environment::new);
        env.clone()
    }

    fn processBlock(&mut self, blockId: BlockId, env: Environment) -> Vec<(BlockId, Environment)> {
        //println!(" Processing block: {}", blockId);
        let block = self.profileBuilder.f.getBlockById(blockId);
        let inner = block.getInner();
        let b = inner.borrow();
        let mut jumpTargets = Vec::new();
        for i in &b.instructions {
            let vars = i.kind.collectVariables();
            let mut varTypes = Vec::new();
            for v in vars {
                let varType = self.profileBuilder.getFinalVarType(&v);
                varTypes.push((v.name(), varType));
            }
            if self.traceEnabled {
                println!("   Instr: {} {:?}", i, varTypes);
            }
            match &i.kind {
                InstructionKind::Ref(dest, arg) => {
                    let _argType = self.profileBuilder.getFinalVarType(arg);
                    let destType = self.profileBuilder.getFinalVarType(dest);
                    if self.traceEnabled {
                        println!("    Ref: {} -> {}", _argType, destType);
                    }
                    let refTyVar = destType.vars.first().expect("ref type must have a var");
                    self.borrowPath(varToPath(arg), refTyVar, arg.location().clone());
                }
                InstructionKind::AddressOfField(dest, receiver, fields, isRaw) => {
                    if *isRaw {
                    } else {
                        self.useVar(&env, receiver.clone(), true);
                        let segments = buildSegments(fields);
                        let path = Path {
                            p: SimplePath {
                                root: receiver.name(),
                                items: segments,
                            },
                        };
                        if self.traceEnabled {
                            println!("    AddressOfField: {} -> {}", receiver.name(), path);
                        }
                        let destType = self.profileBuilder.getFinalVarType(dest);
                        self.borrowPath(
                            path,
                            destType.vars.first().expect("dest type must have a var"),
                            receiver.location().clone(),
                        );
                    }
                }
                InstructionKind::PtrOf(dest, _) => {
                    env.revivePath(&varToPath(&dest));
                }
                InstructionKind::Assign(dest, src) => {
                    self.checkVar(&env, &src, false);
                    env.revivePath(&varToPath(&dest));
                }
                InstructionKind::FieldAssign(_, src, _) => {
                    self.checkVar(&env, &src, true);
                }
                InstructionKind::FieldRef(dest, _, _) => {
                    env.revivePath(&varToPath(&dest));
                }
                InstructionKind::Jump(_, target) => {
                    jumpTargets.push((target.clone(), env.snapshot()));
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    for case in cases {
                        jumpTargets.push((case.branch.clone(), env.snapshot()));
                    }
                }
                InstructionKind::IntegerSwitch(_, cases) => {
                    for case in cases {
                        jumpTargets.push((case.branch.clone(), env.snapshot()));
                    }
                }
                _ => {
                    self.processInstruction(&env, &i.kind);
                }
            }
        }
        jumpTargets
    }

    fn processInstruction(&mut self, env: &Environment, i: &InstructionKind) {
        let mut usedVars = i.collectVariables();
        if let Some(v) = i.getResultVar() {
            usedVars.retain(|x| *x != v);
        }
        for usedVar in usedVars {
            self.useVar(env, usedVar, false);
        }
        if let Some(v) = i.getResultVar() {
            env.revivePath(&varToPath(&v));
        }
    }

    fn useVar(&mut self, env: &Environment, usedVar: Variable, read: bool) {
        let varType = self.checkVar(env, &usedVar, read);
        if self.traceEnabled {
            println!("    Checking used var: {} of type {}", usedVar, varType);
        }
        for tyVar in &varType.vars {
            if let Some(borrows) = self.borrows.get(tyVar) {
                if self.traceEnabled {
                    for p in borrows.paths.keys() {
                        println!("     Borrows for type {}: {}", tyVar, p);
                    }
                }
                for (path, info) in &borrows.paths {
                    if let Some(deathInfo) = env.isPathDead(&path) {
                        if deathInfo.isDrop {
                            BorrowCheckerError::UseAfterDrop(
                                path.userVisible(),
                                usedVar.location(),
                                info.location.clone(),
                            )
                            .report(self.ctx);
                        } else {
                            BorrowCheckerError::UseAfterMove(
                                path.userVisible(),
                                usedVar.location(),
                                deathInfo.location,
                                info.location.clone(),
                            )
                            .report(self.ctx);
                        }
                    }
                }
            }
        }
    }

    fn borrowPath(&mut self, path: Path, refTyVar: &Type, location: Location) {
        self.borrowInner(path.clone(), refTyVar, location.clone());
        let mut affected = Vec::new();
        let mut current = vec![refTyVar.clone()];
        loop {
            let copy = current.clone();
            current.clear();
            for (from, to) in &self.links {
                if copy.contains(from) && !affected.contains(to) {
                    affected.push(to.clone());
                    current.push(to.clone());
                }
            }
            if current.is_empty() {
                break;
            }
        }
        for tyVar in affected {
            self.borrowInner(path.clone(), &tyVar, location.clone());
        }
    }

    fn borrowInner(&mut self, path: Path, refTyVar: &Type, location: Location) {
        let borrows = self
            .borrows
            .entry(refTyVar.clone())
            .or_insert_with(|| BorrowSet { paths: BTreeMap::new() });
        if self.traceEnabled {
            println!("    {} borrows: {}", refTyVar, path);
        }
        borrows.paths.insert(path, BorrowInfo { location });
    }

    fn checkVar(&self, env: &Environment, usedVar: &Variable, read: bool) -> ExtendedType {
        let varType = self.profileBuilder.getFinalVarType(&usedVar);
        if varType.ty.isNamed() && !read {
            // this is a struct or enum and it is being moved, mark its paths as dead
            if self.traceEnabled {
                println!("    Moving named var: {} of type {}", usedVar, varType);
            }
            env.markPathDead(varToPath(&usedVar), usedVar.location().clone(), usedVar.kind().isDrop());
        }
        varType
    }
}

fn varToPath(v: &Variable) -> Path {
    Path {
        p: SimplePath::new(v.name()),
    }
}

#[derive(Clone)]
pub struct DeathInfo {
    pub location: Location,
    pub isDrop: bool,
}

#[derive(Clone)]
struct Environment {
    deadPaths: Rc<RefCell<BTreeMap<Path, DeathInfo>>>,
}

impl Environment {
    fn new() -> Environment {
        Environment {
            deadPaths: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn snapshot(&self) -> Environment {
        let copy = self.deadPaths.borrow().clone();
        Environment {
            deadPaths: Rc::new(RefCell::new(copy)),
        }
    }

    fn merge(&self, other: &Environment) -> bool {
        let mut changed = false;
        let otherDeadPaths = other.deadPaths.borrow();
        let mut selfDeadPaths = self.deadPaths.borrow_mut();
        for (path, info) in otherDeadPaths.iter() {
            match selfDeadPaths.get_mut(path) {
                Some(existing) => {
                    if !existing.isDrop && info.isDrop {
                        *existing = info.clone();
                        changed = true;
                    }
                }
                None => {
                    selfDeadPaths.insert(path.clone(), info.clone());
                    changed = true;
                }
            }
        }
        changed
    }

    fn markPathDead(&self, path: Path, location: Location, isDrop: bool) {
        //println!("    Marking path dead: {}", path);
        self.deadPaths.borrow_mut().insert(path, DeathInfo { location, isDrop });
    }

    fn revivePath(&self, path: &Path) {
        //println!("    Reviving path: {}", path);
        self.deadPaths.borrow_mut().remove(path);
    }

    fn isPathDead(&self, path: &Path) -> Option<DeathInfo> {
        //println!("    Checking if path is dead: {}", path);
        //println!("    Dead paths:");
        for (p, info) in self.deadPaths.borrow().iter() {
            //println!("      {}", p);
            if path.p.contains(&p.p) {
                return Some(info.clone());
            }
        }
        None
    }

    fn getDeathInfo(&self, path: &Path) -> Option<DeathInfo> {
        self.deadPaths.borrow().get(path).cloned()
    }
}

enum BorrowCheckerError {
    UseAfterMove(String, Location, Location, Location),
    UseAfterDrop(String, Location, Location),
}

impl BorrowCheckerError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            BorrowCheckerError::UseAfterMove(path, useLocation, moveLocation, borrowLocation) => {
                let slogan = format!(
                    "Trying to use borrow of {} but {} is already moved at this point",
                    ctx.yellow(&path.to_string()),
                    ctx.yellow(&path.to_string())
                );
                let mut entries = Vec::new();
                entries.push(Entry::new(
                    Some("NOTE: Value used here".to_string()),
                    useLocation.clone(),
                ));
                entries.push(Entry::new(
                    Some("NOTE: Value moved here".to_string()),
                    moveLocation.clone(),
                ));
                entries.push(Entry::new(
                    Some("NOTE: Value borrowed here".to_string()),
                    borrowLocation.clone(),
                ));
                let r = Report::build(ctx, slogan, entries);
                r.print();
            }
            BorrowCheckerError::UseAfterDrop(path, useLocation, borrowLocation) => {
                let slogan = format!(
                    "Trying to use borrow of {} but {} is already dropped at this point",
                    ctx.yellow(&path.to_string()),
                    ctx.yellow(&path.to_string())
                );
                let mut entries = Vec::new();
                entries.push(Entry::new(
                    Some("NOTE: Value used here".to_string()),
                    useLocation.clone(),
                ));
                entries.push(Entry::new(
                    Some("NOTE: Value borrowed here".to_string()),
                    borrowLocation.clone(),
                ));
                let r = Report::build(ctx, slogan, entries);
                r.print();
            }
        }
        std::process::exit(1);
    }
}
