#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    cfg::Builder::Builder,
    location::FileManager::FileManager,
    ownership::{dataflowprofile::Inference::infer, Borrowchecker::Borrowchecker},
    parser::Parser::*,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args};

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
    let (functions, classes, enums) = resolver.ir();
    let mut typedFunctions = BTreeMap::new();
    for (_, f) in &functions {
        let mut typechecker = Typechecker::new(&functions, &classes, &enums);
        let typedFn = typechecker.run(f);
        //typedFn.dump();
        typedFunctions.insert(typedFn.name.clone(), typedFn);
    }
    let mut borrowCheckedFunctions = BTreeMap::new();
    for (name, f) in typedFunctions {
        let borrowCheckedFn = if f.body.is_some() {
            let mut builder = Builder::new(f.name.to_string());
            builder.build(&f);
            let cfg = builder.getCFG();
            let mut borrowchecker = Borrowchecker::new(cfg);
            borrowchecker.check();
            let updatedFn = borrowchecker.update(&f);
            let cfg = borrowchecker.cfg();
            cfg.printDot();
            updatedFn.dump();
            updatedFn
        } else {
            f
        };
        borrowCheckedFunctions.insert(name, borrowCheckedFn);
    }

    infer(&borrowCheckedFunctions);
}
