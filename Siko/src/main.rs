#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    backend::{DeadCodeEliminator::eliminateDeadCode, Drop::checkDrops, RemoveTuples::removeTuples},
    hir::Program::Program,
    hir_lowering::Lowering::lowerProgram,
    location::{FileManager::FileManager, Report::ReportContext},
    minic::Generator::MiniCGenerator,
    monomorphizer::Monomorphizer::Monomorphizer,
    parser::Parser::*,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args, fs, path::Path};

use crate::siko::{
    ownership::{
        DataOwnershipVar::DataOwnershipVarInference, FunctionGroups, OwnershipProfileInference::ownershipInference,
    },
    util::DependencyProcessor::DependencyGroup,
};

fn typecheck(ctx: &ReportContext, mut program: Program) -> Program {
    let mut result = BTreeMap::new();
    for (_, f) in &program.functions {
        let moduleName = f.name.module();
        let traitMethodSelector = &program
            .traitMethodSelectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(ctx, &program, &traitMethodSelector, f);
        let typedFn = typechecker.run();
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    program.functions = result;
    program
}

// fn borrowcheck(program: &Program) {
//     let builder = DataFlowProfileBuilder::new(program);
//     let program = builder.process();
//     println!("{}", program);
//     for (_, f) in &program.functions {
//         if f.body.is_some() {
//             let mut borrowchecker = BorrowChecker::BorrowChecker::new(f);
//             borrowchecker.check();
//         }
//     }
// }

fn monomorphize(ctx: &ReportContext, program: Program) -> Program {
    let monomorphizer = Monomorphizer::new(ctx, program);
    monomorphizer.run()
}

fn collectFiles(input: &Path) -> Vec<String> {
    let mut allFiles = Vec::new();
    if input.is_dir() {
        for entry in fs::read_dir(input).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.is_dir() {
                allFiles.extend(collectFiles(&path));
            } else if let Some(extension) = path.extension() {
                if extension == "sk" {
                    allFiles.push(format!("{}", path.display()));
                }
            }
        }
    } else {
        allFiles.push(format!("{}", input.display()));
    }
    allFiles
}

fn main() {
    let ctx = ReportContext {};
    let fileManager = FileManager::new();
    let mut resolver = Resolver::new(&ctx);
    let mut parseOutput = false;
    let mut outputFile = "siko_main".to_string();
    let mut inputFiles = Vec::new();
    for arg in args().skip(1) {
        if arg == "-o" {
            parseOutput = true;
            continue;
        }
        if parseOutput {
            outputFile = arg.clone();
            parseOutput = false;
            continue;
        }
        inputFiles.push(arg.clone());
    }
    let mut allFiles = Vec::new();
    for inputFile in inputFiles {
        let files = collectFiles(Path::new(&inputFile));
        allFiles.extend(files);
    }
    for f in allFiles {
        let fileId = fileManager.add(f.clone());
        let mut parser = Parser::new(&ctx, fileId, f.to_string());
        parser.parse();
        let modules = parser.modules();
        for m in modules {
            resolver.addModule(m);
        }
    }
    resolver.process();
    let program = resolver.ir();
    //println!("after resolver\n{}", program);
    let program = typecheck(&ctx, program);
    //println!("after typchk\n{}", program);
    let program = eliminateDeadCode(&ctx, program);
    //println!("after dce\n{}", program);
    let program = checkDrops(&ctx, program);
    //println!("after dropcheck\n{}", program);
    let program = monomorphize(&ctx, program);
    //println!("after mono\n{}", program);
    let program = removeTuples(&program);
    //println!("after remove\n{}", program);
    //ownershipInference(program.clone());
    //println!("after backend\n {}", program);
    let mut mir_program = lowerProgram(&program);
    //println!("mir\n{}", mir_program);
    mir_program.process();
    let c_program = mir_program.toMiniC();
    let mut generator = MiniCGenerator::new(format!("{}.c", outputFile), c_program);
    generator.dump().expect("c generator failed");
    //println!("after data lifetime\n{}", program);
    //borrowcheck(&program);
    //dataflow(&functions);
}
