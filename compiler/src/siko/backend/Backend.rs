use crate::{
    siko::{
        backend::{
            closurelowering::ClosureLowering, drop::Drop::checkDrops, recursivedatahandler::RecursiveDataHandler,
            simplification::Simplifier, DeadCodeEliminator::eliminateDeadCode, FieldRefMerger,
            RemoveTuples::removeTuples,
        },
        hir::Program::Program,
        location::Report::ReportContext,
        monomorphizer::Monomorphizer::Monomorphizer,
        util::Runner::Runner,
    },
    stage,
};

fn monomorphize(ctx: &ReportContext, program: Program) -> Program {
    let monomorphizer = Monomorphizer::new(ctx, program);
    monomorphizer.run()
}

pub fn process(ctx: &ReportContext, runner: &mut Runner, program: Program) -> Program {
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
    let program = ClosureLowering::process(program);
    //println!("after closure lowering\n{}", program);
    let program = stage!(runner, "Checking drops", { checkDrops(&ctx, program) });
    //println!("after dropcheck\n{}", program);
    // program
    //     .dumpToFile("hirdump/afterdropcheck")
    //     .expect("Failed to dump HIR");
    let program = stage!(runner, "Monomorphizing", { monomorphize(&ctx, program) });
    //println!("after mono\n{}", program);
    //verifyTypes(&program);
    let program = stage!(runner, "Removing tuples", { removeTuples(&program) });
    //println!("after remove tuples\n{}", program);
    let program = stage!(runner, "Simplifying", { Simplifier::simplify(program) });
    //println!("after simplification\n{}", program);
    program
}
