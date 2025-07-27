use core::{error, panic};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::drop::{
        BlockProcessor::BlockProcessor,
        Context::Context,
        Error::{reportErrors, Error},
        Path::Path,
        SingleUseVariables::{SingleUseVariableInfo, SingleUseVariables},
        Usage::{Usage, UsageKind},
    },
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Program::Program,
        Variable::{Variable, VariableName},
    },
    location::Report::{Entry, Report, ReportContext},
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut s = SingleUseVariables::new(f);
        let singleUseInfo = s.process();
        let mut checker = DropChecker::new(f, ctx, &program, singleUseInfo);
        //println!("Checking drops for {}", name);
        checker.process();
        // let f = checker.process();
        // result.functions.insert(name.clone(), f);
    }
    result
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
    singleUseInfo: SingleUseVariableInfo,
}

impl<'a> DropChecker<'a> {
    pub fn new(
        f: &'a Function,
        ctx: &'a ReportContext,
        program: &'a Program,
        singleUseInfo: SingleUseVariableInfo,
    ) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            visited: BTreeMap::new(),
            singleUseInfo: singleUseInfo,
        }
    }

    fn process(&mut self) {
        if self.function.body.is_none() {
            return;
        }
        //println!("Processing function: {}", self.function.name);
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Case {
            blockId: BlockId,
            context: Context,
        }

        let mut visited = BTreeSet::new();
        let mut queue = Vec::new();
        queue.push(Case {
            blockId: BlockId::first(),
            context: Context::new(),
        });
        let mut allCollisions = Vec::new();
        loop {
            let Some(case) = queue.pop() else { break };
            if !visited.insert(case.clone()) {
                continue;
            }
            let block = self.function.getBlockById(case.blockId);
            let mut blockProcessor = BlockProcessor::new(&self.singleUseInfo);
            let (context, jumpTargets) = blockProcessor.process(&block, case.context);
            let collisions = context.validate();
            allCollisions.extend(collisions);
            let jumpContext = context.compress();
            for jumpTarget in jumpTargets {
                queue.push(Case {
                    blockId: jumpTarget,
                    context: jumpContext.clone(),
                });
            }
        }
        let mut errors = Vec::new();
        for c in allCollisions {
            let err = Error::AlreadyMoved {
                path: c.path,
                prevMove: c.prev,
            };
            errors.push(err);
        }
        reportErrors(self.ctx, errors);
        // for blockId in allblocksIds {
        //     let block = self.function.getBlockById(blockId);
        //     let mut blockProcessor = BlockProcessor::new(&self.singleUseInfo);
        //     let mut context = Context::new();
        //     context = blockProcessor.process(&block, context);
        //     let collisions = context.validate();
        //     let mut errors = Vec::new();
        //     for c in collisions {
        //         let err = Error::AlreadyMoved {
        //             path: c.path,
        //             prevMove: c.prev,
        //         };
        //         errors.push(err);
        //     }
        //     reportErrors(self.ctx, errors);
        // }
        // let groups = DependencyProcessor::processDependencies(&mut blockDeps);
        // //println!("all deps {:?}", blockDeps);
        // //println!("groups {:?}", groups);
        // for g in &groups {
        //     //println!("groups {:?}", g);
        //     self.processGroup(&g.items, &blockDeps);
        // }
    }
}
