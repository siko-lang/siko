use crate::siko::{
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
};

pub fn process(ctx: &ReportContext, runner: &mut Runner, program: Program) -> Program {
    let safetyChecker = SafetyChecker::new(&program);
    safetyChecker.check(ctx);
    let program = runner
        .child("dead_code_elimination")
        .run(|| eliminateDeadCode(&ctx, program));
    //println!("after dce\n{}", program);
    let program = runner
        .child("recursive_data_handling")
        .run(|| RecursiveDataHandler::process(ctx, program));
    //println!("after recursive data handling\n{}", program);
    let program = runner
        .child("field_ref_merging")
        .run(|| FieldRefMerger::mergeFieldRefs(program));
    //println!("after field ref merge\n{}", program);
    let dropCheckRunner = runner.child("drop_check");
    let program = dropCheckRunner
        .clone()
        .run(|| checkDrops(&ctx, program, dropCheckRunner));
    //println!("after dropcheck\n{}", program);
    // program
    //     .dumpToFile("hirdump/afterdropcheck")
    //     .expect("Failed to dump HIR");
    let cfg = runner.getConfig();
    let monoRunner = runner.child("monomorphizer");
    let program = monoRunner.clone().run(|| {
        let monomorphizer = Monomorphizer::new(cfg, ctx, program, monoRunner);
        monomorphizer.run()
    });
    //println!("after mono\n{}", program);
    //verifyTypes(&program);

    //println!("after remove tuples\n{}", program);
    let program = runner
        .child("simplification")
        .run(|| Simplifier::simplify(program, Config { enableInliner: false }));
    //println!("after simplification\n{}", program);
    let program = ClosureLowering::process(program);
    //println!("after closure lowering\n{}", program);
    let program = runner.child("coroutine_lowering").run(|| {
        let mut program = program;
        let coroutineStore = coroutinelowering::CoroutineLowering::CoroutineStore::new(&mut program);
        coroutineStore.process();
        program
    });
    //println!("after coroutine lowering\n{}", program);
    let program = runner.child("tuple_removal").run(|| removeTuples(&program));
    //println!("after tuple removal\n{}", program);
    let borrowCheckRunner = runner.child("borrow_check");
    borrowCheckRunner
        .clone()
        .run(|| Check::new(&program).process(ctx, borrowCheckRunner));
    let program = runner
        .child("simplification2")
        .run(|| Simplifier::simplify(program, Config { enableInliner: true }));
    //println!("Final program:\n{}", program);
    program
}
