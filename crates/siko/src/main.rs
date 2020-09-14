use colored::*;
use siko_compiler::compiler::Compiler;
use siko_compiler::compiler::CompilerInput;
use siko_compiler::config::Config;
use std::env;
use std::path::Path;
use walkdir::WalkDir;

fn process_dir(arg: String, inputs: &mut Vec<CompilerInput>) -> bool {
    let path = Path::new(&arg);
    if !path.exists() {
        let path_str = format!("{}", path.display());
        eprintln!(
            "{} path {} does not exist",
            "ERROR:".red(),
            path_str.yellow()
        );
        return false;
    }
    if path.is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry.unwrap();
            if let Some(ext) = entry.path().extension() {
                if ext == "sk" {
                    let input = CompilerInput::File {
                        name: format!("{}", entry.path().display()),
                    };
                    inputs.push(input);
                }
            }
        }
    } else if path.is_file() {
        let input = CompilerInput::File { name: arg };
        inputs.push(input);
    }
    true
}

fn print_usage() {
    println!("arguments: OPTIONS FILENAME... ");
    println!("Options:");
    println!("\t-c <path> compile");
    println!("\t-m measure durations");
    println!("\t-i visualize");
    println!("\t-s <path> path to std");
}

fn process_args(args: Vec<String>) -> (Config, Vec<CompilerInput>, bool) {
    let mut inputs = Vec::new();
    let mut config = Config::new();
    let mut success = true;
    let mut std_path = format!("std");
    let mut file_given = false;
    let arg_len = args.len();
    let mut index = 0;
    while index < arg_len {
        let arg = args[index].as_ref();
        match arg {
            "-c" => {
                if index + 1 >= args.len() {
                    eprintln!("{} missing path after -c", "ERROR:".red(),);
                    success = false;
                    break;
                } else {
                    let output_file = args[index + 1].to_string();
                    config.compile = Some(output_file);
                    index += 1;
                }
            }
            "-m" => {
                config.measure_durations = true;
            }
            "-i" => {
                config.visualize = true;
            }
            "-s" => {
                if index + 1 >= args.len() {
                    eprintln!("{} missing path after -s", "ERROR:".red(),);
                    success = false;
                    break;
                } else {
                    std_path = args[index + 1].to_string();
                    index += 1;
                }
            }
            "-h" => {
                success = false;
            }
            "--" => {
                break;
            }
            _ => {
                file_given = true;
                if !process_dir(arg.to_string(), &mut inputs) {
                    success = false;
                    break;
                }
            }
        }
        index += 1;
    }
    if !file_given {
        if success {
            eprintln!("no file given to compile");
        }
        success = false;
    }
    if success {
        if !process_dir(std_path, &mut inputs) {
            success = false;
        }
    }
    if !success {
        print_usage();
    }
    //println!("Compiling {} file(s)", inputs.len());
    (config, inputs, success)
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    let (config, inputs, success) = process_args(args);

    if !success {
        std::process::exit(1);
    }

    let mut compiler = Compiler::new(config);

    if let Err(e) = compiler.compile(inputs) {
        compiler.report_error(e);
        std::process::exit(1);
    }
}
