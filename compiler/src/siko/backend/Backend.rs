use crate::siko::{
    backend::{
        drop::Drop::checkDrops, simplification::Simplifier, DeadCodeEliminator::eliminateDeadCode, FieldRefMerger,
        RemoveTuples::removeTuples,
    },
    hir::Program::Program,
    location::Report::ReportContext,
    monomorphizer::Monomorphizer::Monomorphizer,
};

fn monomorphize(ctx: &ReportContext, program: Program) -> Program {
    let monomorphizer = Monomorphizer::new(ctx, program);
    monomorphizer.run()
}

pub fn process(ctx: &ReportContext, program: Program) -> Program {
    let program = eliminateDeadCode(&ctx, program);
    //println!("after dce\n{}", program);
    let program = FieldRefMerger::mergeFieldRefs(program);
    //println!("after field ref merge\n{}", program);
    let program = checkDrops(&ctx, program);
    //println!("after dropcheck\n{}", program);
    // program
    //     .dumpToFile("hirdump/afterdropcheck")
    //     .expect("Failed to dump HIR");
    let program = monomorphize(&ctx, program);
    // println!("after mono\n{}", program);
    let program = removeTuples(&program);
    //println!("after remove tuples\n{}", program);
    let program = Simplifier::simplify(program);
    program
}
