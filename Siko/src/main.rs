#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    build::Build::BuildEngine,
    cfg::Builder::Builder,
    ir::{Function::Function, TraitMethodSelector::TraitMethodSelector},
    location::FileManager::FileManager,
    ownership::{dataflowprofile::Inference::dataflow, Borrowchecker::Borrowchecker},
    parser::Parser::*,
    qualifiedname::QualifiedName,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{
    collections::BTreeMap,
    env::args,
    io::{self, BufRead, Write},
};

use crate::siko::build::Build::Key;

fn borrowcheck(functions: BTreeMap<QualifiedName, Function>) -> BTreeMap<QualifiedName, Function> {
    let mut result = BTreeMap::new();
    for (name, f) in functions {
        let borrowCheckedFn = if f.body.is_some() {
            let mut builder = Builder::new(f.name.to_string());
            builder.build(&f);
            let cfg = builder.getCFG();
            let mut borrowchecker = Borrowchecker::new(cfg);
            borrowchecker.check();
            let updatedFn = borrowchecker.update(&f);
            //let cfg = borrowchecker.cfg();
            //cfg.printDot();
            updatedFn
        } else {
            f
        };
        result.insert(name, borrowCheckedFn);
    }
    result
}

fn typecheck(
    functions: BTreeMap<QualifiedName, Function>,
    classes: BTreeMap<QualifiedName, siko::ir::Data::Class>,
    enums: BTreeMap<QualifiedName, siko::ir::Data::Enum>,
    traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
) -> BTreeMap<QualifiedName, Function> {
    let mut result = BTreeMap::new();
    for (_, f) in &functions {
        let moduleName = f.name.module();
        let traitMethodSelector = traitMethodSelectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(&functions, &classes, &enums, &traitMethodSelector);
        let typedFn = typechecker.run(f);
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    result
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
    let (functions, classes, enums, traitMethodSelectors) = resolver.ir();
    let functions = typecheck(functions, classes, enums, traitMethodSelectors);
    //let functions = borrowcheck(functions);
    //dataflow(&functions);
}

fn main2() {
    let mut run = true;
    let mut engine = BuildEngine::new();
    while run {
        let stdin = io::stdin();
        let mut line = String::new();
        line = line.trim_end().to_string();
        print!(">");
        io::stdout().flush().expect("flush failed");
        stdin.lock().read_line(&mut line).expect("read failed");
        line.remove(line.len() - 1);
        let subs: Vec<_> = line.split(" ").collect();
        match subs[0] {
            "quit" => {
                run = false;
            }
            "add" => {
                let filename = subs[1].to_string();
                engine.enqueue(Key::File(filename));
                engine.process();
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
