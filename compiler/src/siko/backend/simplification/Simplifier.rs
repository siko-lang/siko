use crate::siko::{
    backend::simplification::{
        BlockMerger, CompileTimeEvaluator, DeadCodeEliminator, Inliner::Inliner, JumpSimplifier, SwitchSimplifier,
        UnusedAssignmentEliminator, UnusedVariableEliminator, VarSimplifier,
    },
    hir::{Function::Function, FunctionGroupBuilder::FunctionGroupBuilder, Program::Program},
    qualifiedname::QualifiedName,
};

pub struct Config {
    pub enableInliner: bool,
}

pub fn simplify(mut program: Program, config: Config) -> Program {
    let functionGroupBuilder = FunctionGroupBuilder::new(&program);
    let functionGroupInfo = functionGroupBuilder.process();
    let mut inliner = Inliner::new(config.enableInliner, &functionGroupInfo);
    for group in &functionGroupInfo.groups {
        //println!("Simplifying function group: {:?}", group.items);
        for fnName in &group.items {
            let mut simplifiedFunc = program.functions.get(&fnName).unwrap().clone();
            simplifiedFunc = simplifyFunction(&program, simplifiedFunc, &group.items, &mut inliner);
            program.functions.insert(fnName.clone(), simplifiedFunc);
        }
    }
    if config.enableInliner {
        // Remove inline functions that were inlined
        program.functions.retain(|name, _| {
            let keep = !inliner.wasInlined.contains(name) || inliner.wasNotInlined.contains(name);
            if !keep {
                //println!("Removing inlined function: {}", name);
            }
            keep
        });
    }
    program
}

pub fn simplifyFunction(
    program: &Program,
    mut simplifiedFunc: Function,
    groupItems: &Vec<QualifiedName>,
    inliner: &mut Inliner,
) -> Function {
    let trace = false;
    let mut simplified = true;
    while simplified {
        simplified = false;
        if trace {
            println!("Running simplification passes for function: {}", simplifiedFunc.name);
            println!("starting VarSimplifier");
        }
        if let Some(f) = VarSimplifier::simplifyFunction(&simplifiedFunc) {
            //println!("VarSimplifier made changes to function: {}", name);
            if trace {
                println!("VarSimplifier made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting JumpSimplifier");
        }
        if let Some(f) = JumpSimplifier::simplifyFunction(&simplifiedFunc) {
            //println!("JumpSimplifier made changes to function: {}", name);
            if trace {
                println!("JumpSimplifier made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting SwitchSimplifier");
        }
        if let Some(f) = SwitchSimplifier::simplifyFunction(&simplifiedFunc) {
            //println!("SwitchSimplifier made changes to function: {}", name);
            if trace {
                println!("SwitchSimplifier made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting BlockMerger");
        }
        if let Some(f) = BlockMerger::simplifyFunction(&simplifiedFunc) {
            //println!("BlockMerger made changes to function: {}", name);
            if trace {
                println!("BlockMerger made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting CompileTimeEvaluator");
        }
        if let Some(f) = CompileTimeEvaluator::simplifyFunction(&simplifiedFunc) {
            //println!("CompileTimeEvaluator made changes to function: {}", name);
            if trace {
                println!("CompileTimeEvaluator made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting DeadCodeEliminator");
        }
        if let Some(f) = DeadCodeEliminator::eliminateDeadCode(&simplifiedFunc) {
            //println!("DeadCodeEliminator made changes to function: {}", name);
            if trace {
                println!("DeadCodeEliminator made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting UnusedVariableEliminator");
        }
        if let Some(f) = UnusedVariableEliminator::eliminateUnusedVariable(&simplifiedFunc, program) {
            //println!("UnusedVariableEliminator made changes to function: {}", name);
            if trace {
                println!(
                    "UnusedVariableEliminator made changes to function: {}",
                    simplifiedFunc.name
                );
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting UnusedAssignmentEliminator");
        }
        if let Some(f) = UnusedAssignmentEliminator::simplifyFunction(&simplifiedFunc, program) {
            //println!("UnusedAssignmentEliminator made changes to function: {}", name);
            if trace {
                println!(
                    "UnusedAssignmentEliminator made changes to function: {}",
                    simplifiedFunc.name
                );
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
        if trace {
            println!("starting Inliner");
        }
        if let Some(f) = inliner.process(&simplifiedFunc, program, groupItems) {
            if trace {
                println!("Inliner made changes to function: {}", simplifiedFunc.name);
                println!("{}", f);
            }
            simplifiedFunc = f;
            simplified = true;
        }
    }
    simplifiedFunc
}
