use crate::siko::{
    backend::simplification::{CompileTimeEvaluator, DeadCodeEliminator, JumpSimplifier, VarSimplifier},
    hir::Program::Program,
};

pub fn simplify(program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut simplifiedFunc = f.clone();
        let mut simplified = true;
        while simplified {
            simplified = false;
            if let Some(f) = VarSimplifier::simplifyFunction(&simplifiedFunc) {
                simplifiedFunc = f;
                simplified = true;
            }
            if let Some(f) = JumpSimplifier::simplifyFunction(&simplifiedFunc) {
                simplifiedFunc = f;
                simplified = true;
            }
            if let Some(f) = CompileTimeEvaluator::simplifyFunction(&simplifiedFunc) {
                simplifiedFunc = f;
                simplified = true;
            }
            if let Some(f) = DeadCodeEliminator::eliminateDeadCode(&simplifiedFunc) {
                simplifiedFunc = f;
                simplified = true;
            }
        }
        result.functions.insert(name.clone(), simplifiedFunc);
    }
    result
}
