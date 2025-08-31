#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    hir_lowering::Lowering::lowerProgram,
    location::{FileManager::FileManager, Report::ReportContext},
    minic::Generator::MiniCGenerator,
    parser::Parser::*,
    resolver::Resolver::Resolver,
};

use std::{env::args, fs, path::Path, process::Command};

use crate::siko::{
    backend::{Backend, TypeVerifier::verifyTypes},
    typechecker::Typechecker::typecheck,
    util::Runner::Runner,
};

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

enum OptimizationLevel {
    None,
    O3,
}

fn runCommand(command: &str, args: &[&str]) {
    let status = Command::new(command)
        .args(args)
        .status()
        .expect("Failed to execute command");
    if !status.success() {
        eprintln!("Command {:?} failed with status: {}", command, status);
        std::process::exit(1);
    }
}

fn main() {
    let ctx = ReportContext {};
    let fileManager = FileManager::new();

    let mut outputFile = "siko_main".to_string();
    let mut inputFiles = Vec::new();
    let mut args: Vec<String> = args().collect();
    if args.len() < 2 {
        usage();
        return;
    }
    let mut sanitized = false;
    let phase;
    let mut pass_details = false;
    let mut optimization = OptimizationLevel::None;
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
            "-O3" => {
                optimization = OptimizationLevel::O3;
            }
            "--sanitize" => {
                sanitized = true;
            }
            "--pass-details" => {
                pass_details = true;
            }
            _ => {
                inputFiles.push(args[i].clone());
            }
        }
        i += 1;
    }

    let mut runner = Runner::new(pass_details);
    let mut resolver = stage!(runner, "Parsing", {
        let mut resolver = Resolver::new(&ctx);
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
        resolver
    });
    let program = stage!(runner, "Resolving names", {
        resolver.process();
        resolver.ir()
    });
    //println!("after resolver\n{}", program);
    let program = stage!(runner, "Type checking", { typecheck(&ctx, program) });
    let program = stage!(runner, "Verifying types", {
        verifyTypes(&program);
        program
    });
    //println!("after typechecker\n{}", program);
    let program = Backend::process(&ctx, &mut runner, program);
    //println!("after backend\n{}", program);
    let mut mir_program = stage!(runner, "Lowering to MIR", { lowerProgram(&program) });
    //println!("mir\n{}", mir_program);
    stage!(runner, "Processing MIR", { mir_program.process() });
    let c_program = stage!(runner, "Converting MIR to MiniC", { mir_program.toMiniC() });
    let c_output_path = format!("{}.c", outputFile);
    let object_path = format!("{}.o", outputFile);
    let bin_output_path = format!("./{}", outputFile);
    let mut generator = MiniCGenerator::new(c_output_path.clone(), c_program);
    stage!(runner, "Generating C code", {
        generator.dump().expect("c generator failed")
    });
    if phase == BuildPhase::BuildSource {
        // Only build the project
        return;
    }
    let mut compile_args = vec!["-g", "-c", &c_output_path, "-o", &object_path, "-Wno-pointer-sign"];
    let mut link_args = vec!["-g", &object_path, "-o", &bin_output_path];
    if sanitized {
        compile_args.push(CLANG_SANITIZE_FLAGS);
        link_args.push(CLANG_SANITIZE_FLAGS);
    }
    match optimization {
        OptimizationLevel::None => {
            compile_args.push("-O1");
            link_args.push("-O1");
        }
        OptimizationLevel::O3 => {
            compile_args.push("-O3");
            link_args.push("-O3");
        }
    }

    runCommand("clang", &compile_args);
    runCommand("clang", &link_args);
    if phase == BuildPhase::Run {
        // remove the c source and object
        fs::remove_file(c_output_path).expect("Failed to remove C source file");
        fs::remove_file(object_path).expect("Failed to remove object file");
        runCommand(&bin_output_path, &[]);
    }
    runner.report();
}
