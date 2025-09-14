use crate::siko::{
    backend::simplification::{
        BlockMerger, CompileTimeEvaluator, DeadCodeEliminator, Inliner::Inliner, JumpSimplifier,
        UnusedAssignmentEliminator, UnusedVariableEliminator, VarSimplifier,
    },
    hir::{Function::Function, FunctionGroupBuilder::FunctionGroupBuilder, Program::Program},
    qualifiedname::QualifiedName,
};

pub fn simplify(mut program: Program) -> Program {
    let functionGroupBuilder = FunctionGroupBuilder::new(&program);
    let functionGroups = functionGroupBuilder.process();
    let mut inliner = Inliner::new();
    for group in functionGroups {
        for fnName in &group.items {
            let mut simplifiedFunc = program.functions.get(&fnName).unwrap().clone();
            simplifiedFunc = simplifyFunction(&program, simplifiedFunc, &group.items, &mut inliner);
            program.functions.insert(fnName.clone(), simplifiedFunc);
        }
    }
    program
        .functions
        .retain(|name, f| !f.isInline() || inliner.savedInlineFn.contains(name));
    program
}

pub fn simplifyFunction(
    program: &Program,
    mut simplifiedFunc: Function,
    groupItems: &Vec<QualifiedName>,
    inliner: &mut Inliner,
) -> Function {
    let mut simplified = true;
    while simplified {
        simplified = false;
        //println!("Running simplification passes for function: {}", name);
        if let Some(f) = VarSimplifier::simplifyFunction(&simplifiedFunc) {
            //println!("VarSimplifier made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = JumpSimplifier::simplifyFunction(&simplifiedFunc) {
            //println!("JumpSimplifier made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = BlockMerger::simplifyFunction(&simplifiedFunc) {
            //println!("BlockMerger made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = CompileTimeEvaluator::simplifyFunction(&simplifiedFunc) {
            //println!("CompileTimeEvaluator made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = DeadCodeEliminator::eliminateDeadCode(&simplifiedFunc) {
            //println!("DeadCodeEliminator made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = UnusedVariableEliminator::eliminateUnusedVariable(&simplifiedFunc, program) {
            //println!("UnusedVariableEliminator made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }
        if let Some(f) = UnusedAssignmentEliminator::simplifyFunction(&simplifiedFunc, program) {
            //println!("UnusedAssignmentEliminator made changes to function: {}", name);
            simplifiedFunc = f;
            simplified = true;
        }

        simplifiedFunc = inliner.process(&simplifiedFunc, program, groupItems);
    }
    simplifiedFunc
}
