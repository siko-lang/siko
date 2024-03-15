#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    cfg::Builder::Builder, location::FileManager::FileManager, parser::Parser::*,
    resolver::Resolver::Resolver, typechecker::Typechecker::Typechecker,
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
        if typedFn.body.is_some() {
            let mut builder = Builder::new(typedFn.name.to_string());
            builder.build(&typedFn);
            let cfg = builder.getCFG();
            cfg.printDot();
        }
        typedFunctions.insert(typedFn.name.clone(), typedFn);
    }
}
