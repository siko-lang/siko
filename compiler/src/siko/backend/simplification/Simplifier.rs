use crate::siko::{
    backend::simplification::{
        BlockMerger, CompileTimeEvaluator, DeadCodeEliminator, JumpSimplifier, UnusedAssignmentEliminator,
        UnusedVariableEliminator, VarSimplifier,
    },
    hir::Program::Program,
};

pub fn simplify(program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut simplifiedFunc = f.clone();
        let mut simplified = true;
        //println!("Simplifying function: {}", name);
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
            if let Some(f) = UnusedVariableEliminator::eliminateUnusedVariable(&simplifiedFunc, &program) {
                //println!("UnusedVariableEliminator made changes to function: {}", name);
                simplifiedFunc = f;
                simplified = true;
            }
            if let Some(f) = UnusedAssignmentEliminator::simplifyFunction(&simplifiedFunc, &program) {
                //println!("UnusedAssignmentEliminator made changes to function: {}", name);
                simplifiedFunc = f;
                simplified = true;
            }
        }
        //println!("Finished simplifying function: {}", name);
        result.functions.insert(name.clone(), simplifiedFunc);
    }
    result
}
