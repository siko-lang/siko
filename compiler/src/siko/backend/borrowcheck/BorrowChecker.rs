use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
        },
        DataGroups::DataGroups,
    },
    hir::{
        Block::BlockId, Function::Function, Instruction::InstructionKind, Program::Program, Type::Type,
        Variable::VariableName,
    },
    location::{
        Location::Location,
        Report::{Report, ReportContext},
    },
    qualifiedname::QualifiedName,
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

struct BorrowSet {
    paths: BTreeSet<Path>,
}

pub struct BorrowChecker<'a> {
    ctx: &'a ReportContext,
    borrows: BTreeMap<Type, BorrowSet>,
    profileBuilder: FunctionProfileBuilder<'a>,
    blockEnvs: BTreeMap<BlockId, Environment>,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        f: &'a Function,
        program: &'a Program,
        dataGroups: &'a DataGroups<'a>,
        profileStore: &'a mut FunctionProfileStore,
        functionGroup: Vec<QualifiedName>,
    ) -> BorrowChecker<'a> {
        BorrowChecker {
            ctx,
            borrows: BTreeMap::new(),
            profileBuilder: FunctionProfileBuilder::new(f, program, dataGroups, profileStore, functionGroup),
            blockEnvs: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) {
        if self.profileBuilder.f.body.is_none() {
            return;
        }
        //println!("Borrow checking function: {}", self.profileBuilder.f.name);
        //println!("Function profile {}", self.profileBuilder.f);
        self.profileBuilder.process();
        self.processBlock(BlockId::first());
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> Environment {
        let env = self.blockEnvs.entry(blockId).or_insert_with(|| Environment {
            deadPaths: Rc::new(RefCell::new(Vec::new())),
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
            //println!("   Instr: {} {:?}", i, varTypes);
            match &i.kind {
                InstructionKind::Ref(dest, arg) => {
                    let _argType = self.profileBuilder.getFinalVarType(arg);
                    let destType = self.profileBuilder.getFinalVarType(dest);
                    //println!("    Ref: {} -> {}", _argType, destType);
                    let refTyVar = destType.vars.first().expect("ref type must have a var");
                    let borrows = self
                        .borrows
                        .entry(refTyVar.clone())
                        .or_insert_with(|| BorrowSet { paths: BTreeSet::new() });
                    let path = Path { root: arg.name() };
                    //println!("    {} borrows: {}", refTyVar, path);
                    borrows.paths.insert(path);
                }
                InstructionKind::Assign(dest, src) => {
                    if src.getType().isNamed() {
                        // this is a struct or enum and it is being moved, mark its paths as dead
                        env.markPathDead(Path { root: src.name() });
                    }
                    env.revivePath(&Path { root: dest.name() });
                }
                InstructionKind::FieldAssign(dest, src, field) => {
                    if src.getType().isNamed() {
                        // this is a struct or enum and it is being moved, mark its paths as dead
                        env.markPathDead(Path { root: src.name() });
                    }
                }
                InstructionKind::FieldRef(dest, src, field) => {
                    env.revivePath(&Path { root: dest.name() });
                }
                _ => {
                    let mut usedVars = i.kind.collectVariables();
                    if let Some(v) = i.kind.getResultVar() {
                        usedVars.retain(|x| *x != v);
                    }
                    for usedVar in usedVars {
                        let varType = self.profileBuilder.getFinalVarType(&usedVar);
                        if varType.ty.isNamed() {
                            // this is a struct or enum and it is being moved, mark its paths as dead
                            env.markPathDead(Path { root: usedVar.name() });
                        }
                        for tyVar in &varType.vars {
                            if let Some(borrows) = self.borrows.get(tyVar) {
                                for path in &borrows.paths {
                                    if env.isPathDead(&path) {
                                        BorrowCheckerError::UseAfterMove(path.to_string(), usedVar.location())
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

#[derive(Clone)]
struct Environment {
    deadPaths: Rc<RefCell<Vec<Path>>>,
}

impl Environment {
    fn markPathDead(&self, path: Path) {
        //println!("    Marking path dead: {}", path);
        self.deadPaths.borrow_mut().push(path);
    }

    fn revivePath(&self, path: &Path) {
        //println!("    Reviving path: {}", path);
        self.deadPaths.borrow_mut().retain(|p| p != path);
    }

    fn isPathDead(&self, path: &Path) -> bool {
        self.deadPaths.borrow().iter().any(|p| p == path)
    }
}

enum BorrowCheckerError {
    UseAfterMove(String, Location),
}

impl BorrowCheckerError {
    pub fn report(&self, ctx: &ReportContext) -> ! {
        match &self {
            BorrowCheckerError::UseAfterMove(path, l) => {
                let slogan = format!(
                    "Trying to use borrow of {} but {} is already moved at this point",
                    ctx.yellow(&path.to_string()),
                    ctx.yellow(&path.to_string())
                );
                let r = Report::new(ctx, slogan, Some(l.clone()));
                r.print();
            }
        }
        std::process::exit(1);
    }
}
