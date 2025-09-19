use crate::{
    siko::{
        backend::{
            borrowcheck::Check::Check,
            closurelowering::ClosureLowering,
            coroutinelowering,
            drop::Drop::checkDrops,
            recursivedatahandler::RecursiveDataHandler,
            simplification::Simplifier::{self, Config},
            DeadCodeEliminator::eliminateDeadCode,
            FieldRefMerger,
            RemoveTuples::removeTuples,
            SafetyChecker::SafetyChecker,
        },
        hir::Program::Program,
        location::Report::ReportContext,
        monomorphizer::Monomorphizer::Monomorphizer,
        util::Runner::Runner,
    },
    stage,
};

pub fn process(ctx: &ReportContext, runner: &mut Runner, program: Program) -> Program {
    let safetyChecker = SafetyChecker::new(&program);
    safetyChecker.check(ctx);
    let program = stage!(runner, "Eliminating dead code", { eliminateDeadCode(&ctx, program) });
    //println!("after dce\n{}", program);
    let program = stage!(runner, "Handling recursive data", {
        RecursiveDataHandler::process(ctx, program)
    });
    //println!("after recursive data handling\n{}", program);
    let program = stage!(runner, "Merging field references", {
        FieldRefMerger::mergeFieldRefs(program)
    });
    //println!("after field ref merge\n{}", program);
    let program = stage!(runner, "Checking drops", { checkDrops(&ctx, program) });
    //println!("after dropcheck\n{}", program);
    // program
    //     .dumpToFile("hirdump/afterdropcheck")
    //     .expect("Failed to dump HIR");
    let cfg = runner.config.clone();
    let program = stage!(runner, "Monomorphizing", {
        let monomorphizer = Monomorphizer::new(cfg, ctx, program);
        monomorphizer.run()
    });
    //println!("after mono\n{}", program);
    //verifyTypes(&program);
    let program = stage!(runner, "Removing tuples", { removeTuples(&program) });
    //println!("after remove tuples\n{}", program);
    let program = stage!(runner, "Simplifying", {
        Simplifier::simplify(program, Config { enableInliner: false })
    });
    //println!("after simplification\n{}", program);
    let program = ClosureLowering::process(program);
    //println!("after closure lowering\n{}", program);
    let program = stage!(runner, "Coroutine lowering", {
        let mut coroutineStore = coroutinelowering::CoroutineLowering::CoroutineStore::new();
        coroutineStore.process(program)
    });
    //println!("after coroutine lowering\n{}", program);
    Check::new(&program).process(ctx);
    let program = stage!(runner, "Simplifying2", {
        Simplifier::simplify(program, Config { enableInliner: true })
    });
    //println!("Final program:\n{}", program);
    program
}
