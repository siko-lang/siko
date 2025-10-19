use crate::siko::{
    backend::simplification::{
        BlockMerger, CompileTimeEvaluator, DeadCodeEliminator, Inliner::Inliner, JumpSimplifier, SwitchSimplifier,
        UnusedAssignmentEliminator, UnusedVariableEliminator, VarSimplifier,
    },
    hir::{Function::Function, FunctionGroupBuilder::FunctionGroupBuilder, Program::Program},
    qualifiedname::QualifiedName,
    util::Runner::Runner,
};

pub struct Config {
    pub enableInliner: bool,
}

pub fn simplify(mut program: Program, config: Config, traceEnabled: bool, runner: Runner) -> Program {
    let functionGroupInfo = runner.child("function_group_builder").run(|| {
        let functionGroupBuilder = FunctionGroupBuilder::new(&program);
        functionGroupBuilder.process()
    });
    let mut inliner = Inliner::new(config.enableInliner, &functionGroupInfo);
    let functionRunner = runner.child("function");
    for group in &functionGroupInfo.groups {
        //println!("Simplifying function group: {:?}", group.items);
        for fnName in &group.items {
            let mut simplifiedFunc = program.functions.get(&fnName).unwrap().clone();
            simplifiedFunc = simplifyFunction(
                &program,
                simplifiedFunc,
                &group.items,
                &mut inliner,
                traceEnabled,
                functionRunner.clone(),
            );
            program.functions.insert(fnName.clone(), simplifiedFunc);
        }
    }
    if config.enableInliner {
        // Remove inline functions that were inlined
        runner.child("remove_inlined_functions").run(|| {
            program.functions.retain(|name, _| {
                let keep = !inliner.wasInlined.contains(name) || inliner.wasNotInlined.contains(name);
                if !keep {
                    //println!("Removing inlined function: {}", name);
                }
                keep
            });
        });
    }
    program
}

pub fn simplifyFunction(
    program: &Program,
    mut simplifiedFunc: Function,
    groupItems: &Vec<QualifiedName>,
    inliner: &mut Inliner,
    traceEnabled: bool,
    runner: Runner,
) -> Function {
    let trace = traceEnabled;
    let varSimplifierRunner = runner.child("var_simplifier");
    let jumpSimplifierRunner = runner.child("jump_simplifier");
    let switchSimplifierRunner = runner.child("switch_simplifier");
    let blockMergerRunner = runner.child("block_merger");
    let compileTimeEvaluatorRunner = runner.child("compile_time_evaluator");
    let deadCodeEliminatorRunner = runner.child("dead_code_eliminator");
    let unusedVariableEliminatorRunner = runner.child("unused_variable_eliminator");
    let unusedAssignmentEliminatorRunner = runner.child("unused_assignment_eliminator");
    let inlinerRunner = runner.child("inliner");
    let mut simplified = true;
    while simplified {
        simplified = false;
        if trace {
            println!("Running simplification passes for function: {}", simplifiedFunc.name);
            println!("start state {}", simplifiedFunc);
            println!("starting VarSimplifier");
        }
        if let Some(f) = varSimplifierRunner.run(|| VarSimplifier::simplifyFunction(&simplifiedFunc)) {
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
        if let Some(f) = jumpSimplifierRunner.run(|| JumpSimplifier::simplifyFunction(&simplifiedFunc)) {
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
        if let Some(f) = switchSimplifierRunner.run(|| SwitchSimplifier::simplifyFunction(&simplifiedFunc)) {
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
        if let Some(f) = blockMergerRunner.run(|| BlockMerger::simplifyFunction(&simplifiedFunc)) {
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
        if let Some(f) = compileTimeEvaluatorRunner.run(|| CompileTimeEvaluator::simplifyFunction(&simplifiedFunc)) {
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
        if let Some(f) = deadCodeEliminatorRunner.run(|| DeadCodeEliminator::eliminateDeadCode(&simplifiedFunc)) {
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
        if let Some(f) = unusedVariableEliminatorRunner
            .run(|| UnusedVariableEliminator::eliminateUnusedVariable(&simplifiedFunc, program))
        {
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
        if let Some(f) = unusedAssignmentEliminatorRunner.run(|| {
            UnusedAssignmentEliminator::simplifyFunction(
                &simplifiedFunc,
                program,
                unusedAssignmentEliminatorRunner.clone(),
            )
        }) {
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
        if let Some(f) = inlinerRunner.run(|| inliner.process(&simplifiedFunc, program, groupItems)) {
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
