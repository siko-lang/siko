#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    location::Location::FileId, parser::Parser::*, resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args};

fn main() {
    let mut resolver = Resolver::new();
    for arg in args().skip(1) {
        let mut parser = Parser::new(FileId::new(0), arg.to_string());
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
}
