use std::collections::BTreeMap;

use crate::siko::{
    backend::{
        borrowcheck::{
            functionprofiles::{
                FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
            },
            BorrowVarMap::{BorrowVarMap, BorrowVarMapBuilder},
            DataGroups::DataGroups,
            Error::BorrowCheckerError,
        },
        path::{Path::Path, Util::buildFieldPath},
    },
    hir::{
        BlockBuilder::InstructionRef, BlockGroupBuilder::BlockGroupBuilder, Function::Function,
        Instruction::InstructionKind, Program::Program, Type::Type,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::QualifiedName,
    util::Runner::Runner,
};

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
        }
        for link in &self.profileBuilder.profile.links {
            self.links.insert(link.from.clone(), link.to.clone());
        }
        let blockGroupBuilder = BlockGroupBuilder::new(self.profileBuilder.f);
        let groupInfo = blockGroupBuilder.process();
        if self.traceEnabled {
            println!("Links:");
            for (from, to) in &self.links {
                println!("  {} -> {}", from, to);
            }
            println!("Block groups: {:?}", groupInfo.groups);
        }
        let borrowVarMapBuilder = BorrowVarMapBuilder::new(&self.profileBuilder, self.traceEnabled);
        let borrowVarMap = borrowVarMapBuilder.buildBorrowVarMap(&self.links, &groupInfo);
        self.buildBorrows();
        self.checkBorrows(&borrowVarMap);
    }

    fn movePath(&mut self, path: Path, liveBorrowVars: &Vec<Type>) {
        // this is a struct or enum and it is being moved, mark its paths as dead
        for tyVar in liveBorrowVars {
            if let Some(borrows) = self.borrows.get(tyVar) {
                if self.traceEnabled {
                    for p in borrows.paths.keys() {
                        println!("     Borrows for type {}: {}", tyVar, p);
                    }
                }
                for (borrowPath, info) in &borrows.paths {
                    if path == *borrowPath || borrowPath.contains(&path) {
                        if path.root.kind().isDrop() {
                            BorrowCheckerError::UseAfterDrop(
                                path.userPath(),
                                path.root.location(),
                                info.location.clone(),
                            )
                            .report(self.ctx);
                        } else {
                            BorrowCheckerError::UseAfterMove(
                                path.userPath(),
                                path.root.location(),
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
        let borrows = self
            .borrows
            .entry(refTyVar.clone())
            .or_insert_with(|| BorrowSet { paths: BTreeMap::new() });
        if self.traceEnabled {
            println!("    {} borrows: {}", refTyVar, path);
        }
        borrows.paths.insert(path, BorrowInfo { location });
    }

    fn buildBorrows(&mut self) {
        let body = self.profileBuilder.f.body.as_ref().unwrap();
        for (_, block) in &body.blocks {
            let inner = block.getInner();
            let b = inner.borrow();
            for instruction in &b.instructions {
                match &instruction.kind {
                    InstructionKind::Ref(dest, arg) => {
                        let _argType = self.profileBuilder.getFinalVarType(arg);
                        let destType = self.profileBuilder.getFinalVarType(dest);
                        if self.traceEnabled {
                            println!("    Ref: {} -> {}", _argType, destType);
                        }
                        let refTyVar = destType.vars.first().expect("ref type must have a var");
                        self.borrowPath(arg.toPath(), refTyVar, arg.location().clone());
                    }
                    InstructionKind::AddressOfField(dest, receiver, fields, isRaw) => {
                        if *isRaw {
                        } else {
                            let path = buildFieldPath(receiver, fields);
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
                    _ => {}
                }
            }
        }
    }

    fn checkBorrows(&mut self, borrowVarMap: &BorrowVarMap) {
        let body = self.profileBuilder.f.body.as_ref().unwrap();
        for (blockId, block) in &body.blocks {
            let inner = block.getInner();
            let b = inner.borrow();
            for (index, instruction) in b.instructions.iter().enumerate() {
                let instrRef = InstructionRef {
                    blockId: blockId.clone(),
                    instructionId: index as u32,
                };
                if let Some(liveBorrowVars) = borrowVarMap.borrowVarMap.get(&instrRef) {
                    match &instruction.kind {
                        InstructionKind::Ref(_, _) => {}
                        InstructionKind::PtrOf(_, _) => {}
                        InstructionKind::FieldAccess(_, info) => {
                            if info.isRef {
                                // this is a borrow, do nothing
                            } else {
                                let path = buildFieldPath(&info.receiver, &info.fields);
                                if self.traceEnabled {
                                    println!("    FieldRef: {} -> {}", info.receiver.name(), path);
                                }
                                self.movePath(path, liveBorrowVars);
                            }
                        }
                        InstructionKind::AddressOfField(_, _, _, _) => {}
                        _ => {
                            let vars = instruction.kind.collectVariables();
                            for v in vars {
                                self.movePath(v.toPath(), liveBorrowVars);
                            }
                        }
                    }
                }
            }
        }
    }
}
