use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
        },
        DataGroups::{DataGroups, ExtendedType},
    },
    hir::{
        Block::BlockId,
        Function::Function,
        Instruction::InstructionKind,
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
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
    pub root: VariableName,
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root.visibleName())
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
        BorrowChecker {
            ctx,
            borrows: BTreeMap::new(),
            profileBuilder: FunctionProfileBuilder::new(f, program, dataGroups, profileStore, functionGroup, runner),
            blockEnvs: BTreeMap::new(),
            links: BTreeMap::new(),
            traceEnabled: false,
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
        self.processBlock(BlockId::first());
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> Environment {
        let env = self.blockEnvs.entry(blockId).or_insert_with(|| Environment {
            deadPaths: Rc::new(RefCell::new(BTreeMap::new())),
        });
        env.clone()
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let env = self.getEnvForBlock(blockId);
        let block = self.profileBuilder.f.getBlockById(blockId);
        let inner = block.getInner();
        let b = inner.borrow();
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
                InstructionKind::PtrOf(dest, _) => {
                    env.revivePath(&varToPath(&dest));
                }
                InstructionKind::Assign(dest, src) => {
                    self.checkVar(&env, &src);
                    env.revivePath(&varToPath(&dest));
                }
                InstructionKind::FieldAssign(_, src, _) => {
                    self.checkVar(&env, &src);
                }
                InstructionKind::FieldRef(dest, _, _) => {
                    env.revivePath(&varToPath(&dest));
                }
                _ => {
                    let mut usedVars = i.kind.collectVariables();
                    if let Some(v) = i.kind.getResultVar() {
                        usedVars.retain(|x| *x != v);
                    }
                    for usedVar in usedVars {
                        let varType = self.checkVar(&env, &usedVar);
                        for tyVar in &varType.vars {
                            if let Some(borrows) = self.borrows.get(tyVar) {
                                for (path, info) in &borrows.paths {
                                    if env.isPathDead(&path) {
                                        let deathInfo =
                                            env.getDeathInfo(&path).expect("dead path must have death info");
                                        if deathInfo.isDrop {
                                            BorrowCheckerError::UseAfterDrop(
                                                path.to_string(),
                                                usedVar.location(),
                                                info.location.clone(),
                                            )
                                            .report(self.ctx);
                                        } else {
                                            BorrowCheckerError::UseAfterMove(
                                                path.to_string(),
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

    fn checkVar(&self, env: &Environment, usedVar: &Variable) -> ExtendedType {
        let varType = self.profileBuilder.getFinalVarType(&usedVar);
        if varType.ty.isNamed() {
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
    Path { root: v.name() }
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
    fn markPathDead(&self, path: Path, location: Location, isDrop: bool) {
        //println!("    Marking path dead: {}", path);
        self.deadPaths.borrow_mut().insert(path, DeathInfo { location, isDrop });
    }

    fn revivePath(&self, path: &Path) {
        //println!("    Reviving path: {}", path);
        self.deadPaths.borrow_mut().remove(path);
    }

    fn isPathDead(&self, path: &Path) -> bool {
        self.deadPaths.borrow().contains_key(path)
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
