#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    build::Build::BuildEngine,
    ir::{Function::Function, TraitMethodSelector::TraitMethodSelector},
    location::FileManager::FileManager,
    ownership::{BorrowChecker, DataGroups::createDataGroups},
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

fn typecheck(
    functions: BTreeMap<QualifiedName, Function>,
    classes: &BTreeMap<QualifiedName, siko::ir::Data::Class>,
    enums: &BTreeMap<QualifiedName, siko::ir::Data::Enum>,
    traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
) -> BTreeMap<QualifiedName, Function> {
    let mut result = BTreeMap::new();
    for (_, f) in &functions {
        let moduleName = f.name.module();
        let traitMethodSelector = traitMethodSelectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(&functions, classes, enums, &traitMethodSelector);
        let typedFn = typechecker.run(f);
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    result
}

fn borrowcheck(functions: BTreeMap<QualifiedName, Function>) {
    for (_, f) in &functions {
        let mut borrowchecker = BorrowChecker::BorrowChecker::new(&functions);
        borrowchecker.run(f);
        //typedFn.dump();
    }
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
    let functions = typecheck(functions, &classes, &enums, traitMethodSelectors);
    createDataGroups(&classes, &enums);
    let functions = borrowcheck(functions);

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
