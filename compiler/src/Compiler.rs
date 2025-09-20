use std::{env::args, fs, process::Command};

use crate::{
    siko::{
        backend::{Backend, TypeVerifier::verifyTypes},
        hir_lowering::Lowering::Lowering,
        location::{FileManager::FileManager, Report::ReportContext},
        minic::Generator::MiniCGenerator,
        parser::Parser::Parser,
        resolver::Resolver::Resolver,
        typechecker::Typechecker::typecheck,
        util::{
            Config::{BuildPhase, Config, OptimizationLevel},
            Runner::Runner,
        },
    },
    stage,
    PackageFinder::PackageFinder,
};

fn fatalError(message: &str) -> ! {
    eprintln!("Fatal error: {}", message);
    std::process::exit(1);
}

pub struct Compiler {
    config: Config,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { config: Config::new() }
    }

    fn parse<'a>(
        &self,
        runner: &mut Runner,
        config: &Config,
        ctx: &'a ReportContext,
        fileManager: &FileManager,
    ) -> Resolver<'a> {
        let resolver = stage!(runner, "Parsing", {
            let mut resolver = Resolver::new(&ctx);
            let mut packageFinder = PackageFinder::new();
            packageFinder.processPaths(config.inputFiles.clone(), true);
            packageFinder.processPaths(config.externalFiles.clone(), false);

            //packageFinder.dump();

            for package in packageFinder.packages {
                for f in package.files {
                    let fileId = fileManager.add(f.clone());
                    let mut parser = Parser::new(&ctx, fileId, f.to_string());
                    parser.parse();
                    let modules = parser.modules();
                    for m in modules {
                        resolver.addModule(m);
                    }
                }
            }
            resolver
        });
        resolver
    }

    fn parseConfig(&mut self) -> bool {
        let parentDir = std::env::current_exe()
            .expect("Failed to get current exe path")
            .parent()
            .expect("Failed to get parent dir")
            .to_path_buf();
        let mut stdLibPath = parentDir.join("std");
        match std::env::var("SIKO_STD_PATH") {
            Ok(val) => {
                stdLibPath = val.into();
            }
            Err(_) => {}
        }
        if !stdLibPath.exists() {
            fatalError(&format!(
                "ERROR: standard library path {} does not exist",
                stdLibPath.display()
            ));
        }
        let mut args: Vec<String> = args().collect();
        if args.len() < 2 {
            usage();
            return false;
        }
        match args[1].as_str() {
            "run" => {
                args.remove(1);
                self.config.buildPhase = BuildPhase::Run;
            }
            "build" => {
                args.remove(1);
                self.config.buildPhase = BuildPhase::Build;
            }
            "buildsource" => {
                args.remove(1);
                self.config.buildPhase = BuildPhase::BuildSource;
            }
            "test" => {
                args.remove(1);
                self.config.buildPhase = BuildPhase::Run;
                self.config.testOnly = true;
            }
            _ => {
                usage();
                return false;
            }
        }
        let mut i = 1;
        let mut external_mode = false;
        let mut nostd = false;
        while i < args.len() {
            match args[i].as_str() {
                "-o" => {
                    i += 1;
                    if i < args.len() {
                        self.config.outputFile = args[i].clone();
                    } else {
                        eprintln!("Error: -o option requires an argument");
                        return false;
                    }
                }
                "-O3" => {
                    self.config.optimization = OptimizationLevel::O3;
                }
                "--sanitize" => {
                    self.config.sanitized = true;
                }
                "--pass-details" => {
                    self.config.passDetails = true;
                }
                "--external" => {
                    external_mode = true;
                }
                "--nostd" => {
                    nostd = true;
                }
                _ => {
                    if external_mode {
                        self.config.externalFiles.push(args[i].clone());
                    } else {
                        self.config.inputFiles.push(args[i].clone());
                    }
                }
            }
            i += 1;
        }
        if !nostd {
            self.config.externalFiles.push(format!("{}", stdLibPath.display()));
        }
        true
    }

    fn compileC(
        &self,
        runner: &mut Runner,
        c_output_path: String,
        object_path: String,
        bin_output_path: String,
        mut generator: MiniCGenerator,
    ) {
        stage!(runner, "Generating C code", {
            generator.dump().expect("c generator failed");
        });
        if self.config.buildPhase == BuildPhase::BuildSource {
            // Only build the project
            runner.report();
            return;
        }
        let mut compile_args = vec!["-g", "-c", &c_output_path, "-o", &object_path, "-Wno-pointer-sign"];
        let mut link_args = vec!["-g", &object_path, "-o", &bin_output_path];
        if self.config.sanitized {
            compile_args.push(CLANG_SANITIZE_FLAGS);
            link_args.push(CLANG_SANITIZE_FLAGS);
        }
        match self.config.optimization {
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

        if self.config.buildPhase == BuildPhase::Run {
            // remove the c source and object
            //fs::remove_file(c_output_path).expect("Failed to remove C source file");
            fs::remove_file(object_path).expect("Failed to remove object file");
            runCommand(&bin_output_path, &[]);
            if self.config.testOnly {
                fs::remove_file(bin_output_path).expect("Failed to remove binary file");
            }
        }
    }

    pub fn run(&mut self) {
        let ctx = ReportContext {};
        let fileManager = FileManager::new();

        if !self.parseConfig() {
            return;
        }

        let mut runner = Runner::new(self.config.clone());
        let mut resolver = self.parse(&mut runner, &self.config, &ctx, &fileManager);
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
        let mut mir_program = stage!(runner, "Lowering to MIR", {
            let lowering = Lowering::new(program);
            lowering.lowerProgram()
        });
        //println!("mir\n{}", mir_program);
        stage!(runner, "Processing MIR", { mir_program.process() });
        let c_program = stage!(runner, "Converting MIR to MiniC", { mir_program.toMiniC() });
        let c_output_path = format!("{}.c", self.config.outputFile);
        let object_path = format!("{}.o", self.config.outputFile);
        let bin_output_path = format!("./{}", self.config.outputFile);
        let generator = MiniCGenerator::new(c_output_path.clone(), c_program);
        self.compileC(&mut runner, c_output_path, object_path, bin_output_path, generator);
    }
}

static CLANG_SANITIZE_FLAGS: &str = "-fsanitize=undefined,address,alignment,null,bounds,integer,enum,implicit-conversion,float-cast-overflow,float-divide-by-zero";

fn usage() {
    eprintln!("Usage: siko <command>");
    eprintln!("Commands:");
    eprintln!("  run       Compiles the source code into an executable and runs it, -o <output_file>s");
    eprintln!("  buildsource     Only compiles the source code into source code, -o <output_file>");
    eprintln!("  build     Compiles the source code into an executable, -o <output_file>");
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
