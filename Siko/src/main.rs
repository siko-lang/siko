#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    cfg::Builder::Builder,
    ir::Program::Program,
    location::FileManager::FileManager,
    monomorphizer::Monomorphizer::Monomorphizer,
    ownership::{
        BorrowChecker, DataFlow::DataFlowProfileBuilder::DataFlowProfileBuilder,
        DataLifetime::DataLifeTimeInference,
    },
    parser::Parser::*,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args};

fn typecheck(mut program: Program) -> Program {
    let mut result = BTreeMap::new();
    for (_, f) in &program.functions {
        let moduleName = f.name.module();
        let traitMethodSelector = &program
            .traitMethodSelectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(&program, &traitMethodSelector);
        let typedFn = typechecker.run(f);
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    program.functions = result;
    program
}

fn borrowcheck(program: &Program) {
    let builder = DataFlowProfileBuilder::new(program);
    let program = builder.process();
    println!("{}", program);
    for (_, f) in &program.functions {
        if f.body.is_some() {
            let mut cfgBuilder = Builder::new(f.name.toString(), f.result.clone());
            cfgBuilder.build(f);
            let controlFlowGraph = cfgBuilder.getCFG();
            controlFlowGraph.printDot();
            let mut borrowchecker = BorrowChecker::BorrowChecker::new(&program.functions);
            borrowchecker.run(f);
        }
    }
}

fn monomorphize(program: Program) -> Program {
    let monomorphizer = Monomorphizer::new(program);
    monomorphizer.run()
}

fn main() {
    let fileManager = FileManager::new();
    let mut resolver = Resolver::new();
    for arg in args().skip(1) {
        let fileId = fileManager.add(arg.clone());
        let mut parser = Parser::new(fileId, arg.to_string());
        parser.parse();
        let modules = parser.modules();
        for m in modules {
            resolver.addModule(m);
        }
    }
    resolver.process();
    let program = resolver.ir();
    let program = typecheck(program);
    let program = monomorphize(program);
    //println!("after mono\n{}", program);
    let data_lifetime_inferer = DataLifeTimeInference::new(program);
    let program = data_lifetime_inferer.process();
    //println!("after data lifetime\n{}", program);
    borrowcheck(&program);
    //dataflow(&functions);
}
