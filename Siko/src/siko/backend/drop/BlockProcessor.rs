use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    backend::drop::{
        Error::Error,
        Event::{Event, EventSeries},
        Path::Path,
        SingleUseVariables::SingleUseVariableInfo,
        Usage::{Usage, UsageKind},
    },
    hir::{
        Function::Block,
        Instruction::InstructionKind,
        Variable::{Variable, VariableName},
    },
    location::Report::{Entry, Report, ReportContext},
};

pub struct Context {
    pub liveData: Vec<Path>,
    pub usages: BTreeMap<VariableName, EventSeries>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            liveData: Vec::new(),
            usages: BTreeMap::new(),
        }
    }

    pub fn addLive(&mut self, data: Path) {
        self.liveData.push(data);
    }

    pub fn addAssign(&mut self, path: Path) {
        self.usages
            .entry(path.root.value.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Assign(path));
    }

    pub fn addUsage(&mut self, usage: Usage) {
        self.usages
            .entry(usage.path.root.value.clone())
            .or_insert_with(EventSeries::new)
            .push(Event::Usage(usage));
    }

    pub fn useVar(&mut self, var: &Variable) {
        let ty = var.getType();
        //  println!("Using variable: {} {}", var.value.visibleName(), ty);
        if ty.isReference() {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location.clone()),
                kind: UsageKind::Ref,
            });
        } else {
            self.addUsage(Usage {
                path: Path::new(var.clone(), var.location.clone()),
                kind: UsageKind::Move,
            });
        }
    }

    pub fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        for (var_name, usages) in &self.usages {
            //println!("Validating usages for variable: {} {} usage(s)", var_name, usages.len());
            if let Err(errs) = usages.validate() {
                errors.extend(errs);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub struct BlockProcessor<'a> {
    singleUseVars: &'a SingleUseVariableInfo,
    receiverPaths: BTreeMap<Variable, Path>,
}

impl<'a> BlockProcessor<'a> {
    pub fn new(singleUseVars: &'a SingleUseVariableInfo) -> BlockProcessor<'a> {
        BlockProcessor {
            singleUseVars,
            receiverPaths: BTreeMap::new(),
        }
    }

    pub fn process(&mut self, block: &Block) {
        // Process the block here
        let mut context = Context::new();
        //println!("Processing block with id: {}", block.id);
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::DeclareVar(var, _) => {
                    context.addLive(Path::new(var.clone(), var.location.clone()));
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::FunctionCall(_, _, args) => {
                    for arg in args {
                        context.useVar(arg);
                    }
                }
                InstructionKind::Assign(dest, src) => {
                    context.useVar(src);
                    let path = Path::new(dest.clone(), dest.location.clone());
                    context.addAssign(path.clone());
                }
                InstructionKind::Return(_, arg) => {
                    context.useVar(arg);
                }
                InstructionKind::FieldRef(dest, receiver, name) => {
                    let destTy = dest.getType();
                    let mut path =
                        Path::new(receiver.clone(), receiver.location.clone()).add(name.clone(), dest.location.clone());
                    if self.singleUseVars.isSingleUse(&dest.value) && self.singleUseVars.isReceiver(&dest.value) {
                        self.receiverPaths.insert(dest.clone(), path.clone());
                    } else {
                        if let Some(origPath) = self.receiverPaths.get(receiver) {
                            path = origPath.add(name.clone(), dest.location.clone());
                        }
                        if destTy.isReference() {
                            context.addUsage(Usage {
                                path,
                                kind: UsageKind::Ref,
                            });
                        } else {
                            context.addUsage(Usage {
                                path,
                                kind: UsageKind::Move,
                            });
                        }
                    }
                }
                InstructionKind::Tuple(_, args) => {
                    for arg in args {
                        context.useVar(arg);
                    }
                }
                i => {
                    panic!("Unhandled instruction kind: {}", i);
                }
            }
        }

        let ctx = &ReportContext::new();
        let errors = context.validate();
        match errors {
            Ok(_) => {
                //println!("Block processed successfully.");
            }
            Err(errs) => {
                self.reportErrors(ctx, errs);
            }
        }
    }

    fn reportErrors(&self, ctx: &ReportContext, errors: Vec<Error>) {
        for error in errors {
            match error {
                Error::AlreadyMoved { path, prevMove } => {
                    let mut entries = Vec::new();
                    entries.push(Entry::new(None, path.location.clone()));
                    entries.push(Entry::new(
                        Some(format!("NOTE: previous move was here")),
                        prevMove.location.clone(),
                    ));
                    Report::build(
                        ctx,
                        format!("Value {} already moved", ctx.yellow(&path.userPath())),
                        entries,
                    )
                    .print();
                }
            }
        }
    }
}
