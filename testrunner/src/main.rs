use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const COMPILER: &str = "./siko.bin";

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const TIMEOUT_SECS: u64 = 45;

fn main() {
    let siko_bin = Path::new(COMPILER);

    let bless = std::env::args().any(|a| a == "--bless");
    let valgrind = std::env::args().any(|a| a == "--valgrind");

    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--bless" && a != "--re" && a != "--valgrind")
        .collect();

    if bless {
        println!("blessing snapshots...");
    }

    let mut total = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut failed_names: Vec<String> = Vec::new();

    let success_cases_nostd = discover_cases(Path::new("test/success/nostd"), &filters);
    println!(
        "=== success ({} cases) nostd ===",
        success_cases_nostd.len()
    );
    for case in &success_cases_nostd {
        if case.join("SKIP").exists() {
            println!("  SKIP  {} (SKIP file present)", case.to_string_lossy());
            skipped += 1;
            continue;
        }
        if bless {
            bless_success_case(siko_bin, case, true);
        } else {
            run_success_case(
                siko_bin,
                case,
                &mut total,
                &mut failed,
                &mut failed_names,
                true,
                valgrind,
            );
        }
    }

    let success_cases_std = discover_cases(Path::new("test/success/std"), &filters);
    println!("=== success ({} cases) std ===", success_cases_std.len());
    for case in &success_cases_std {
        if case.join("SKIP").exists() {
            println!("  SKIP  {} (SKIP file present)", case.to_string_lossy());
            skipped += 1;
            continue;
        }
        if bless {
            bless_success_case(siko_bin, case, false);
        } else {
            run_success_case(
                siko_bin,
                case,
                &mut total,
                &mut failed,
                &mut failed_names,
                false,
                valgrind,
            );
        }
    }

    let failure_dir = Path::new("test/failure");
    if failure_dir.exists() {
        let failure_cases = discover_cases(failure_dir, &filters);
        println!("\n=== failure ({} cases) ===", failure_cases.len());
        for case in &failure_cases {
            if case.join("SKIP").exists() {
                println!("  SKIP  {} (SKIP file present)", case.to_string_lossy());
                skipped += 1;
                continue;
            }
            if bless {
                bless_failure_case(siko_bin, case);
            } else {
                run_failure_case(siko_bin, case, &mut total, &mut failed, &mut failed_names);
            }
        }
    }

    if !bless {
        let passed = total - failed;
        if skipped > 0 {
            println!("\n{skipped} skipped");
        }
        println!("{passed}/{total} passed");

        if failed > 0 {
            println!("\nfailed tests:");
            for name in &failed_names {
                println!("  {name}");
            }
            std::process::exit(1);
        }
    }
}

fn format_duration(d: Duration) -> String {
    let total_ms = d.as_millis();
    let mins = total_ms / 60_000;
    let secs = (total_ms % 60_000) / 1_000;
    let ms = total_ms % 1_000;
    if mins > 0 {
        format!("{}m {}s {}ms", mins, secs, ms)
    } else if secs > 0 {
        format!("{}s {}ms", secs, ms)
    } else {
        format!("{}ms", ms)
    }
}

// ─── Discovery ────────────────────────────────────────────────────────────────

fn discover_cases(root: &Path, filters: &Vec<String>) -> Vec<PathBuf> {
    let mut cases = Vec::new();
    collect_cases(root, &mut cases, filters);
    cases.sort();
    cases
}

fn collect_cases(dir: &Path, out: &mut Vec<PathBuf>, filters: &Vec<String>) {
    let mut subdirs: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    subdirs.sort();

    for subdir in subdirs {
        if subdir.join("main.sk").exists() {
            if filters.is_empty() || filters.iter().any(|f| subdir.to_string_lossy().contains(f)) {
                out.push(subdir);
            }
        } else {
            collect_cases(&subdir, out, filters);
        }
    }
}

// ─── Bless ────────────────────────────────────────────────────────────────────

fn build_run_selfhost(
    siko_bin: &Path,
    case: &Path,
    source: &Path,
    nostd: bool,
    bless: bool,
    valgrind: bool,
) -> Result<(Duration, Duration), String> {
    let bin = case.join("test_selfhost.bin");
    let bin_str = bin.to_str().unwrap();
    let source_str = source.to_str().unwrap();

    let mut siko_args = vec!["build"];
    if nostd {
        siko_args.push("--no-std");
    }
    siko_args.push(source_str);
    siko_args.push("-o");
    siko_args.push(bin_str);

    let build_start = Instant::now();

    let out = std::process::Command::new(siko_bin)
        .args(&siko_args)
        .output()
        .map_err(|e| format!("failed to spawn siko.bin: {e}"))?;

    let build_dur = build_start.elapsed();

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        return Err(format!("siko.bin exited non-zero:\n{stdout}{stderr}"));
    }

    let exec_start = Instant::now();
    let abs_bin = fs::canonicalize(&bin).map_err(|e| format!("binary not found {}: {e}", bin.display()))?;
    let actual = invoke_success(&abs_bin, &[], case, valgrind);
    let exec_dur = exec_start.elapsed();
    let _ = fs::remove_file(&bin);
    let actual = actual?;

    let snapshot = case.join("output.txt");
    if bless {
        write_if_changed(&snapshot, &actual)
            .map_err(|e| format!("cannot write {}: {e}", snapshot.display()))?;
    } else {
        let expected = fs::read_to_string(&snapshot)
            .map_err(|e| format!("cannot read {}: {e}", snapshot.display()))?;
        if actual != expected {
            return Err(format!(
                "[selfhost] output differs from snapshot ({})\n--- expected ---\n{}--- actual ---\n{}",
                snapshot.display(),
                expected,
                actual,
            ));
        }
    }
    Ok((build_dur, exec_dur))
}

fn bless_success_case(compiler: &Path, case: &Path, nostd: bool) {
    let label = case.to_string_lossy();
    let source = case.join("main.sk");

    print!("  {label} ...");
    std::io::stdout().flush().ok();

    let mut ok = true;
    if let Err(msg) = build_run_selfhost(compiler, case, &source, nostd, true, false) {
        eprintln!("  [build] failed for {label}:\n{msg}");
        ok = false;
    }

    if ok {
        println!(" {GREEN}BLESSED{RESET}");
    } else {
        println!(" {RED}FAILED{RESET}");
    }
}

fn bless_failure_case(compiler: &Path, case: &Path) {
    let label = case.to_string_lossy();
    let source = case.join("main.sk");

    print!("  {label} ...");
    std::io::stdout().flush().ok();

    match invoke_failure_stdout(compiler, &["build", source.to_str().unwrap()]) {
        Ok(output) => {
            let path = case.join("output.txt");
            if let Err(e) = write_if_changed(&path, &output) {
                eprintln!("  error writing {}: {e}", path.display());
                println!(" {RED}FAILED{RESET}");
            } else {
                println!(" {GREEN}BLESSED{RESET}");
            }
        }
        Err(msg) => {
            eprintln!("  [build] failed for {label}:\n{msg}");
            println!(" {RED}FAILED{RESET}");
        }
    }
}

// ─── Success cases ────────────────────────────────────────────────────────────

fn run_success_case(
    siko_bin: &Path,
    case: &Path,
    total: &mut usize,
    failed: &mut usize,
    failed_names: &mut Vec<String>,
    nostd: bool,
    valgrind: bool,
) {
    *total += 1;
    let label = case.to_string_lossy();
    let source = case.join("main.sk");

    print!("  {label} ...");
    std::io::stdout().flush().ok();

    let mut failures: Vec<(&str, String)> = Vec::new();
    let mut build = Duration::ZERO;
    let mut exec = Duration::ZERO;

    if siko_bin.exists() {
        match build_run_selfhost(siko_bin, case, &source, nostd, false, valgrind) {
            Ok((b, e)) => {
                build = b;
                exec = e;
            }
            Err(msg) => failures.push(("self-build", msg)),
        }
    }

    let timing = format!(
        "(build: {} exec: {})",
        format_duration(build),
        format_duration(exec),
    );
    if failures.is_empty() {
        println!(" {GREEN}PASS{RESET} {timing}");
    } else {
        *failed += 1;
        failed_names.push(label.into_owned());
        println!(" {RED}FAIL{RESET} {timing}");
        for (name, msg) in &failures {
            println!("        [{name}]");
            for line in msg.lines() {
                println!("          {line}");
            }
        }
    }
}

// ─── Failure cases ────────────────────────────────────────────────────────────

fn run_failure_case(
    siko_bin: &Path,
    case: &Path,
    total: &mut usize,
    failed: &mut usize,
    failed_names: &mut Vec<String>,
) {
    *total += 1;
    let label = case.to_string_lossy();
    let source = case.join("main.sk");
    let snapshot = case.join("output.txt");

    print!("  {label} ...");
    std::io::stdout().flush().ok();

    let mut failures: Vec<(&str, String)> = Vec::new();
    let self_dur;
    if siko_bin.exists() {
        let self_start = Instant::now();
        if let Err(msg) = failure_snapshot_test_stdout(
            siko_bin,
            &["build", source.to_str().unwrap()],
            &snapshot,
        ) {
            failures.push(("self", msg));
        }
        self_dur = self_start.elapsed();
    } else {
        self_dur = Duration::ZERO;
    }

    let timing = format!("(time: {})", format_duration(self_dur));
    if failures.is_empty() {
        println!(" {GREEN}PASS{RESET} {timing}");
    } else {
        *failed += 1;
        failed_names.push(label.into_owned());
        println!(" {RED}FAIL{RESET} {timing}");
        for (name, msg) in &failures {
            println!("        [{name}]");
            for line in msg.lines() {
                println!("          {line}");
            }
        }
    }
}

fn failure_snapshot_test_stdout(
    compiler: &Path,
    args: &[&str],
    snapshot: &Path,
) -> Result<(), String> {
    failure_snapshot_test_impl(invoke_failure_stdout(compiler, args)?, snapshot)
}

fn failure_snapshot_test_impl(actual: String, snapshot: &Path) -> Result<(), String> {
    let expected = fs::read_to_string(snapshot)
        .map_err(|e| format!("cannot read {}: {e}", snapshot.display()))?;

    if actual != expected {
        return Err(format!(
            "output differs from snapshot ({})\n--- expected ---\n{}--- actual ---\n{}",
            snapshot.display(),
            expected,
            actual,
        ));
    }
    Ok(())
}

// ─── File helpers ─────────────────────────────────────────────────────────────

fn write_if_changed(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(());
        }
    }
    fs::write(path, content)
}

// ─── Invocation helpers ───────────────────────────────────────────────────────

fn invoke_success(compiler: &Path, args: &[&str], cwd: &Path, valgrind: bool) -> Result<String, String> {
    use std::io::Read as _;

    // --leak-check=no: Boehm GC holds memory intentionally in ways valgrind
    // would flag as leaks. We care about invalid accesses, not GC-held memory.
    // --suppressions: silences Boehm GC false positives (conservative stack scan).
    let mut child = if valgrind {
        let mut c = Command::new("valgrind");
        c.args(["--error-exitcode=1", "--leak-check=no", "--suppressions=.github/valgrind.supp"]);
        c.arg(compiler);
        c.args(args);
        c
    } else {
        let mut c = Command::new(compiler);
        c.args(args);
        c
    };
    let mut child = child
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", compiler.display()))?;

    // Drain stdout and stderr on background threads so the pipe buffers never
    // fill up and block the child. try_wait + kill still works on the main thread.
    let mut stdout_pipe = child.stdout.take().unwrap();
    let mut stderr_pipe = child.stderr.take().unwrap();

    let stdout_thread = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).ok();
        buf
    });
    let stderr_thread = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).ok();
        buf
    });

    let deadline = Instant::now() + Duration::from_secs(TIMEOUT_SECS);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();
                    return Err(format!("timed out after {TIMEOUT_SECS}s"));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("wait error: {e}")),
        }
    };

    let stdout_bytes = stdout_thread.join().unwrap_or_default();
    let stderr_bytes = stderr_thread.join().unwrap_or_default();

    if !status.success() {
        let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
        let stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();
        return Err(format!("compiler exited non-zero:\n{stdout}\n{stderr}"));
    }

    Ok(String::from_utf8_lossy(&stdout_bytes).into_owned())
}

fn invoke_failure_stdout(compiler: &Path, args: &[&str]) -> Result<String, String> {
    invoke_failure_impl(compiler, args, true)
}

fn invoke_failure_impl(compiler: &Path, args: &[&str], use_stdout: bool) -> Result<String, String> {
    let out = Command::new(compiler)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {e}", compiler.display()))?;

    if out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        return Err(format!(
            "expected compiler to fail but it succeeded\nstdout:\n{stdout}"
        ));
    }

    if use_stdout {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Ok(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}
