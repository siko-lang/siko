#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    hir::Program::Program,
    hir_lowering::Lowering::lowerProgram,
    location::{FileManager::FileManager, Report::ReportContext},
    minic::Generator::MiniCGenerator,
    parser::Parser::*,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args, fs, path::Path, process::Command};

use crate::siko::backend::Backend;

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

static CLANG_SANITIZE_FLAGS: &str = "-fsanitize=undefined,address,alignment,null,bounds,integer,enum,implicit-conversion,float-cast-overflow,float-divide-by-zero";

fn usage() {
    eprintln!("Usage: siko <command>");
    eprintln!("Commands:");
    eprintln!("  run       Compiles the source code into an executable and runs it, -o <output_file>s");
    eprintln!("  buildsource     Only compiles the source code into source code, -o <output_file>");
    eprintln!("  build     Compiles the source code into an executable, -o <output_file>");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildPhase {
    Run,
    BuildSource,
    Build,
}

fn main() {
    let ctx = ReportContext {};
    let fileManager = FileManager::new();
    let mut resolver = Resolver::new(&ctx);
    let mut outputFile = "siko_main".to_string();
    let mut inputFiles = Vec::new();
    let mut args: Vec<String> = args().collect();
    if args.len() < 2 {
        usage();
        return;
    }
    let mut sanitized = false;
    let phase;
    match args[1].as_str() {
        "run" => {
            args.remove(1);
            phase = BuildPhase::Run;
        }
        "build" => {
            args.remove(1);
            phase = BuildPhase::Build;
        }
        "buildsource" => {
            args.remove(1);
            phase = BuildPhase::BuildSource;
        }
        _ => {
            usage();
            return;
        }
    }
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i < args.len() {
                    outputFile = args[i].clone();
                } else {
                    eprintln!("Error: -o option requires an argument");
                    return;
                }
            }
            "--sanitize" => {
                sanitized = true;
            }
            _ => {
                inputFiles.push(args[i].clone());
            }
        }
        i += 1;
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
    let program = Backend::process(&ctx, program);
    //println!("after backend\n{}", program);
    let mut mir_program = lowerProgram(&program);
    //println!("mir\n{}", mir_program);
    mir_program.process();
    let c_program = mir_program.toMiniC();
    let c_output_path = format!("{}.c", outputFile);
    let object_path = format!("{}.o", outputFile);
    let bin_output_path = format!("./{}", outputFile);
    let mut generator = MiniCGenerator::new(c_output_path.clone(), c_program);
    generator.dump().expect("c generator failed");
    if phase == BuildPhase::BuildSource {
        // Only build the project
        return;
    }
    let mut compile_args = vec!["-g", "-O1", "-c", &c_output_path, "-o", &object_path];
    let mut link_args = vec!["-g", "-O1", &object_path, "-o", &bin_output_path];
    if sanitized {
        compile_args.push(CLANG_SANITIZE_FLAGS);
        link_args.push(CLANG_SANITIZE_FLAGS);
    }
    Command::new("clang")
        .args(&compile_args)
        .status()
        .expect("Failed to execute clang");
    Command::new("clang")
        .args(&link_args)
        .status()
        .expect("Failed to execute clang");
    if phase == BuildPhase::Run {
        // remove the c source and object
        fs::remove_file(c_output_path).expect("Failed to remove C source file");
        fs::remove_file(object_path).expect("Failed to remove object file");
        Command::new(&bin_output_path)
            .status()
            .expect("Failed to execute binary");
    }
}
