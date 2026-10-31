//! Runs the upstream Luau conformance corpus against the vendored luaur VM,
//! scored by a scope file (see `scope.rs`), as a one-way ratchet.
//!
//! ```text
//! luau-conformance run --scope <file> --corpus <dir> [--filter <substr>]
//!                      [--timeout <secs>] [--jobs <n>] [--stack-mib <n>] [--verbose]
//! ```
//!
//! Each script runs in its own child process (this binary, `exec-one`), so
//! a panic, abort, stack overflow or runaway loop costs one script, never the
//! run. The parent prints one line per script in corpus order and a summary,
//! and exits 1 when a `required` script does not pass or an `expected-fail`
//! script passes (promote it), 2 on usage or scope errors.
//!
//! Only `harness.rs` touches the VM.

mod harness;
mod scope;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use scope::{Kind, Scope};

const USAGE: &str = "usage:
  luau-conformance run --scope <file> --corpus <dir> [--filter <substr>]
                       [--timeout <secs>] [--jobs <n>] [--stack-mib <n>] [--verbose]
  luau-conformance exec-one <script> [--stack-mib <n>] [--verbose]";

/// Marks the child's result line on stdout, after anything the script printed.
const RESULT_MARKER: &str = "\u{1e}luau-conformance-result\t";

const DEFAULT_TIMEOUT_SECS: u64 = 120;
/// The script thread's stack. Deep-recursion scripts (calls, errors, pcall)
/// recurse through Rust frames in the transliterated VM; unoptimized builds
/// need far more than the 8 MiB main-thread default.
const DEFAULT_STACK_MIB: usize = 256;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => RunOptions::parse(&args[1..]).and_then(|options| run(&options)),
        Some("exec-one") => exec_one(&args[1..]),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("luau-conformance: {message}");
            ExitCode::from(2)
        }
    }
}

struct RunOptions {
    scope: PathBuf,
    corpus: PathBuf,
    filter: Option<String>,
    timeout: Duration,
    jobs: usize,
    stack_mib: usize,
    verbose: bool,
}

impl RunOptions {
    fn parse(args: &[String]) -> Result<RunOptions, String> {
        let mut scope = None;
        let mut corpus = None;
        let mut options = RunOptions {
            scope: PathBuf::new(),
            corpus: PathBuf::new(),
            filter: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            jobs: std::thread::available_parallelism()
                .map_or(4, |n| n.get())
                .min(8),
            stack_mib: DEFAULT_STACK_MIB,
            verbose: false,
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let mut value = || {
                args.next()
                    .cloned()
                    .ok_or_else(|| format!("{arg} needs a value\n{USAGE}"))
            };
            match arg.as_str() {
                "--scope" => scope = Some(PathBuf::from(value()?)),
                "--corpus" => corpus = Some(PathBuf::from(value()?)),
                "--filter" => options.filter = Some(value()?),
                "--timeout" => {
                    options.timeout = Duration::from_secs(number(&value()?, "--timeout")?)
                }
                "--jobs" => options.jobs = number::<usize>(&value()?, "--jobs")?.max(1),
                "--stack-mib" => options.stack_mib = number(&value()?, "--stack-mib")?,
                "--verbose" => options.verbose = true,
                other => return Err(format!("unknown argument {other}\n{USAGE}")),
            }
        }
        options.scope = scope.ok_or_else(|| format!("--scope is required\n{USAGE}"))?;
        options.corpus = corpus.ok_or_else(|| format!("--corpus is required\n{USAGE}"))?;
        Ok(options)
    }
}

fn number<T: std::str::FromStr>(text: &str, flag: &str) -> Result<T, String> {
    text.parse()
        .map_err(|_| format!("{flag} expects a number, got '{text}'"))
}

#[derive(Clone, Debug)]
enum Status {
    Ok,
    Fail(String),
    Panic(String),
    Timeout,
}

struct ScriptResult {
    status: Status,
    elapsed: Duration,
    output: String,
}

fn run(options: &RunOptions) -> Result<ExitCode, String> {
    let scope_text = std::fs::read_to_string(&options.scope).map_err(|error| {
        format!(
            "cannot read scope file {}: {error}",
            options.scope.display()
        )
    })?;
    let scope = Scope::parse(&scope_text)
        .map_err(|error| format!("{}:\n{error}", options.scope.display()))?;
    let corpus = corpus_scripts(&options.corpus)?;
    scope.check_against(&corpus).map_err(|error| {
        format!(
            "scope file {} does not match the corpus:\n{error}",
            options.scope.display()
        )
    })?;
    let selected: Vec<String> = corpus
        .into_iter()
        .filter(|name| {
            options
                .filter
                .as_deref()
                .is_none_or(|filter| name.contains(filter))
        })
        .collect();
    if selected.is_empty() {
        return Err("--filter selected no scripts".into());
    }

    let exe = std::env::current_exe()
        .map_err(|error| format!("cannot locate own executable: {error}"))?;
    let next = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel::<(usize, ScriptResult)>();
    let selected = Arc::new(selected);
    let workers: Vec<_> = (0..options.jobs.min(selected.len()))
        .map(|_| {
            let (next, sender, selected, exe) =
                (next.clone(), sender.clone(), selected.clone(), exe.clone());
            let (corpus, timeout, stack_mib, verbose) = (
                options.corpus.clone(),
                options.timeout,
                options.stack_mib,
                options.verbose,
            );
            std::thread::spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(name) = selected.get(index) else {
                        break;
                    };
                    let result = run_child(&exe, &corpus.join(name), timeout, stack_mib, verbose);
                    if sender.send((index, result)).is_err() {
                        break;
                    }
                }
            })
        })
        .collect();
    drop(sender);

    println!(
        "luau-conformance: {} of {} scripts from {} (timeout {}s, {} jobs)",
        selected.len(),
        scope.entries.len(),
        options.corpus.display(),
        options.timeout.as_secs(),
        workers.len()
    );
    let mut pending: BTreeMap<usize, ScriptResult> = BTreeMap::new();
    let mut printed = 0;
    let mut tally = Tally::default();
    for (index, result) in receiver {
        pending.insert(index, result);
        while let Some(result) = pending.remove(&printed) {
            let name = &selected[printed];
            let entry = &scope.entries[name];
            report(name, entry.kind, &result, options.verbose, &mut tally);
            printed += 1;
        }
    }
    for worker in workers {
        let _ = worker.join();
    }
    if printed != selected.len() {
        return Err(format!(
            "only {printed} of {} scripts reported",
            selected.len()
        ));
    }
    tally.summarize(&scope);
    Ok(
        if tally.regressions.is_empty() && tally.unexpected_passes.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    )
}

fn corpus_scripts(dir: &Path) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("cannot read corpus {}: {error}", dir.display()))?;
    let mut scripts: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".luau"))
        .collect();
    scripts.sort();
    if scripts.is_empty() {
        return Err(format!("no .luau scripts in {}", dir.display()));
    }
    Ok(scripts)
}

fn run_child(
    exe: &Path,
    script: &Path,
    timeout: Duration,
    stack_mib: usize,
    verbose: bool,
) -> ScriptResult {
    let started = Instant::now();
    let mut command = Command::new(exe);
    command
        .arg("exec-one")
        .arg(script)
        .arg("--stack-mib")
        .arg(stack_mib.to_string());
    if verbose {
        command.arg("--verbose");
    }
    let spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            return ScriptResult {
                status: Status::Panic(format!("cannot spawn runner: {error}")),
                elapsed: started.elapsed(),
                output: String::new(),
            };
        }
    };
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let exit = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(_) => break None,
        }
    };
    let elapsed = started.elapsed();
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    let (printed, verdict) = match stdout.rfind(RESULT_MARKER) {
        Some(at) => (
            &stdout[..at],
            Some(stdout[at + RESULT_MARKER.len()..].trim_end()),
        ),
        None => (stdout.as_str(), None),
    };
    let status = match (exit, verdict) {
        (None, _) => Status::Timeout,
        (Some(_), Some(verdict)) => parse_verdict(verdict),
        (Some(exit), None) => {
            let last = stderr
                .lines()
                .rev()
                .find(|line| !line.trim().is_empty())
                .unwrap_or_default();
            Status::Panic(format!(
                "runner died ({}){}",
                describe_exit(exit),
                if last.is_empty() {
                    String::new()
                } else {
                    format!(": {last}")
                }
            ))
        }
    };
    let mut output = printed.to_owned();
    output.push_str(&stderr);
    ScriptResult {
        status,
        elapsed,
        output,
    }
}

fn drain<R: Read + Send + 'static>(stream: Option<R>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

fn describe_exit(status: std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }
    match status.code() {
        Some(code) => format!("exit {code}"),
        None => "unknown exit".into(),
    }
}

fn parse_verdict(verdict: &str) -> Status {
    let (word, detail) = verdict.split_once(' ').unwrap_or((verdict, ""));
    match word {
        "OK" => Status::Ok,
        "FAIL" => Status::Fail(detail.to_owned()),
        "PANIC" => Status::Panic(detail.to_owned()),
        _ => Status::Panic(format!("unreadable runner verdict: {verdict}")),
    }
}

#[derive(Default)]
struct Tally {
    ok: usize,
    fail: usize,
    panic: usize,
    timeout: usize,
    regressions: Vec<String>,
    unexpected_passes: Vec<String>,
}

fn report(name: &str, kind: Kind, result: &ScriptResult, verbose: bool, tally: &mut Tally) {
    let passed = matches!(result.status, Status::Ok);
    match &result.status {
        Status::Ok => tally.ok += 1,
        Status::Fail(_) => tally.fail += 1,
        Status::Panic(_) => tally.panic += 1,
        Status::Timeout => tally.timeout += 1,
    }
    let verdict = match &result.status {
        Status::Ok => "OK".to_owned(),
        Status::Fail(message) => format!("FAIL {message}"),
        Status::Panic(message) => format!("PANIC {message}"),
        Status::Timeout => "TIMEOUT".to_owned(),
    };
    let note = match (kind, passed) {
        (Kind::Required, false) => {
            tally.regressions.push(name.to_owned());
            "  <-- REGRESSION: required script did not pass"
        }
        (Kind::ExpectedFail, true) => {
            tally.unexpected_passes.push(name.to_owned());
            "  <-- UNEXPECTED PASS: promote to required"
        }
        _ => "",
    };
    println!(
        "{name:<34} {:<7} {:>7.2}s  {verdict}{note}",
        kind.label(),
        result.elapsed.as_secs_f64()
    );
    if verbose && !result.output.is_empty() {
        for line in result.output.lines() {
            println!("    | {line}");
        }
    }
    let _ = std::io::stdout().flush();
}

impl Tally {
    fn summarize(&self, scope: &Scope) {
        let total = self.ok + self.fail + self.panic + self.timeout;
        let verdict = if self.regressions.is_empty() && self.unexpected_passes.is_empty() {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "summary: {total} scripts: {} OK, {} FAIL, {} PANIC, {} TIMEOUT | scope: {} required, {} expected-fail, {} permanent-non-goal | required not passing: {} | expected-fail passing: {} | {verdict}",
            self.ok,
            self.fail,
            self.panic,
            self.timeout,
            scope.count(Kind::Required),
            scope.count(Kind::ExpectedFail),
            scope.count(Kind::PermanentNonGoal),
            self.regressions.len(),
            self.unexpected_passes.len(),
        );
        if !self.regressions.is_empty() {
            println!(
                "required scripts not passing: {}",
                self.regressions.join(" ")
            );
        }
        if !self.unexpected_passes.is_empty() {
            println!(
                "expected-fail scripts now passing (move them to required): {}",
                self.unexpected_passes.join(" ")
            );
        }
    }
}

/// Child mode: run one script in this process on a large-stack thread and
/// print the verdict line last. Panics are caught here; anything that kills
/// the process (abort, stack overflow) is reported by the parent.
fn exec_one(args: &[String]) -> Result<ExitCode, String> {
    let mut script = None;
    let mut stack_mib = DEFAULT_STACK_MIB;
    let mut verbose = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--verbose" => verbose = true,
            "--stack-mib" => {
                stack_mib = number(
                    args.next().ok_or("--stack-mib needs a value")?,
                    "--stack-mib",
                )?;
            }
            path if script.is_none() => script = Some(PathBuf::from(path)),
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    let script = script.ok_or_else(|| USAGE.to_owned())?;
    let source = std::fs::read(&script)
        .map_err(|error| format!("cannot read {}: {error}", script.display()))?;
    let name = script
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    let panic_message = Arc::new(Mutex::new(None::<String>));
    let hook_message = panic_message.clone();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|text| (*text).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".into());
        let location = info
            .location()
            .map(|at| format!(" at {}:{}", at.file(), at.line()))
            .unwrap_or_default();
        // Keep the latest: luaur-rt turns panics inside callbacks into Lua
        // errors, so only the last panic can be the one that escaped.
        if let Ok(mut slot) = hook_message.lock() {
            *slot = Some(format!("{payload}{location}"));
        }
    }));

    let worker = std::thread::Builder::new()
        .name(name.clone())
        .stack_size(stack_mib.saturating_mul(1024 * 1024))
        .spawn(move || harness::run_script(&name, &source, verbose))
        .map_err(|error| format!("cannot start script thread: {error}"))?;
    let verdict = match worker.join() {
        Ok(harness::Outcome::Pass) => "OK".to_owned(),
        Ok(harness::Outcome::Fail(message)) => format!("FAIL {}", one_line(&message)),
        Err(_) => {
            let message = panic_message
                .lock()
                .ok()
                .and_then(|slot| slot.clone())
                .unwrap_or_default();
            format!("PANIC {}", one_line(&message))
        }
    };
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "\n{RESULT_MARKER}{verdict}");
    let _ = stdout.flush();
    Ok(ExitCode::SUCCESS)
}

fn one_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().to_owned()
}
