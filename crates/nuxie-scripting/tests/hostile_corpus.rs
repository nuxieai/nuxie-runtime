//! The hostile-input corpus gate (docs/luaur-client-path-safe.md, section 8,
//! lane F; evidence: docs/evidence/luaur-client-path-lane-h4-2026-11-14.md).
//!
//! Every case under `tests/hostile/` runs in its own child process, in a fresh
//! `ScriptVm` with the default `ScriptExecutionLimits`. The parent watches the
//! child's wall time and the resident memory of its whole process tree, and
//! kills it past the case's stated time or the memory ceiling. A case passes
//! when the child prints the outcome its header requires (an ordinary script
//! error or a normal return), then drops the VM and exits cleanly. Anything
//! else is a failure of that case: a crash, an abort, a hang, memory past the
//! ceiling, a host panic that a trampoline turned into a `rust panic:` error,
//! or the wrong outcome. The run never stops at the first failure.
//!
//! `tests/hostile/expected-failures.txt` lists the cases that fail today, per
//! panic strategy, with the failure kind and the owning lane. The gate fails
//! on any failure not listed and on any listed failure that no longer happens.
//!
//! Run it with `make luau-abort-gate` (both strategies), or alone with
//! `python3 tools/bazel/runtime.py test -p nuxie-scripting --test hostile_corpus`
//! (prefix `NUX_RUNTIME_PANIC=abort` for the abort build). The environment
//! variable `NUXIE_HOSTILE_FILTER` keeps only the cases whose path contains it.
#![cfg(feature = "luau-host")]

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::{
    File, NoopScriptHost, RuntimeFactoryHandle, ScriptInstance, ScriptMethod, ScriptValue,
};
use nuxie_scripting::luau::{Function, Table, Value};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

mod support;

/// Set in a child: the corpus-relative path of the one case it runs.
const CASE_ENV: &str = "NUXIE_HOSTILE_CASE";
const FILTER_ENV: &str = "NUXIE_HOSTILE_FILTER";
/// Set to print each case's whole result text in its child's output.
const VERBOSE_ENV: &str = "NUXIE_HOSTILE_VERBOSE";
const OUTCOME_PREFIX: &str = "HOSTILE-OUTCOME ";
const DROPPED_LINE: &str = "HOSTILE-DROPPED";
/// The case thread's stack: libtest's default for test threads, the stack
/// every other lane's hostile tests ran on.
const CASE_STACK_BYTES: usize = 2 * 1024 * 1024;
/// Resident memory the child's process tree may reach: the two 64 MiB
/// budgets (VM heap and host), twice over for allocator slack, plus the test
/// binary itself. A case that stays within its budgets stays far below it.
const MEMORY_CEILING_KIB: u64 = 512 * 1024;
const PARALLEL_CASES: usize = 4;
/// See `new_vm`.
const MODULE_TIMEOUT_MS: i32 = 20_000;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(50);
const KEPT_OUTPUT_BYTES: usize = 16 * 1024;

fn strategy() -> &'static str {
    if cfg!(panic = "abort") { "abort" } else { "unwind" }
}

fn corpus_dir() -> PathBuf {
    Path::new(option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")))
        .join("tests/hostile")
}

// ---------------------------------------------------------------- headers

#[derive(Debug, Clone, PartialEq)]
enum Expect {
    Ok,
    Error(Option<String>),
    Either,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Harness {
    /// Execute the chunk as a module (`ScriptVm::run_bytecode`).
    Module,
    /// The chunk returns a generator; its table is driven as a scripted
    /// object: `init`, then two frames of `draw`, `advance(1/60)`, `update`.
    Instance,
    /// The chunk is compiled, then changed by the header's `bytecode:`
    /// directive; every variant is loaded and run in its own fresh VM.
    Bytecode,
}

#[derive(Debug, Clone)]
struct Header {
    attacks: String,
    expect: Expect,
    within: Duration,
    harness: Harness,
    view_models: Option<String>,
    bytecode: Option<String>,
}

fn parse_header(source: &str) -> Result<Header, String> {
    let line = source.lines().next().unwrap_or_default();
    let body = line
        .strip_prefix("--! ")
        .ok_or("first line must start with `--! `")?;
    let mut fields = BTreeMap::new();
    for field in body.split(';') {
        let (key, value) = field
            .split_once(':')
            .ok_or_else(|| format!("field without `:`: {field:?}"))?;
        if fields.insert(key.trim().to_owned(), value.trim().to_owned()).is_some() {
            return Err(format!("field {key:?} given twice"));
        }
    }
    let mut take = |key: &str| fields.remove(key);
    let attacks = take("attacks").ok_or("missing `attacks`")?;
    let expect = match take("expect").ok_or("missing `expect`")?.as_str() {
        "ok" => Expect::Ok,
        "either" => Expect::Either,
        "error" => Expect::Error(None),
        other => match other
            .strip_prefix("error(")
            .and_then(|rest| rest.strip_suffix(')'))
        {
            Some(text) if !text.is_empty() => Expect::Error(Some(text.to_owned())),
            _ => return Err(format!("unknown expect {other:?}")),
        },
    };
    let within = take("within").ok_or("missing `within`")?;
    let seconds: u64 = within
        .strip_suffix('s')
        .and_then(|n| n.parse().ok())
        .filter(|n| (1..=120).contains(n))
        .ok_or_else(|| format!("within must be 1s..120s, got {within:?}"))?;
    let harness = match take("harness").as_deref() {
        None | Some("module") => Harness::Module,
        Some("instance") => Harness::Instance,
        Some("bytecode") => Harness::Bytecode,
        Some(other) => return Err(format!("unknown harness {other:?}")),
    };
    let view_models = take("viewmodels");
    let bytecode = take("bytecode");
    if (harness == Harness::Bytecode) != bytecode.is_some() {
        return Err("`bytecode:` goes with `harness: bytecode` and only there".into());
    }
    if let Some(unknown) = fields.keys().next() {
        return Err(format!("unknown field {unknown:?}"));
    }
    Ok(Header {
        attacks,
        expect,
        within: Duration::from_secs(seconds),
        harness,
        view_models,
        bytecode,
    })
}

fn list_cases() -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
            .map(|entry| entry.expect("directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, out);
            } else if path.extension().is_some_and(|ext| ext == "luau") {
                let relative = path.strip_prefix(root).expect("under the corpus");
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let root = corpus_dir();
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out
}

// ------------------------------------------------------------- the child

fn limits() -> ScriptExecutionLimits {
    ScriptExecutionLimits::default()
}

fn new_vm(header: &Header) -> Result<(ScriptVm, PersistentFactory<NullFactory>), String> {
    let mut vm = ScriptVm::new_with_execution_limits(limits()).map_err(|e| format!("boot: {e}"))?;
    // The one limit that is not the default: the module's wall-clock
    // deadline. At 1 s it decides outcomes by machine load, not by the code
    // (a loaded host turned a dozen budget cases into timeouts). The
    // deterministic bounds (interrupt safepoints, VM and host memory) stay
    // at their defaults, and the case's `within` bounds its wall time.
    vm.set_timeout_ms(MODULE_TIMEOUT_MS);
    // A build with the editor's tools (a workspace test unifies them in)
    // starts a VM with editor playback off, which refuses every Audio.start
    // before its own checks. The corpus attacks a playing VM, as a player
    // runs it.
    #[cfg(feature = "tools")]
    vm.set_is_playing(true);
    let mut factory = PersistentFactory::new(NullFactory::new());
    vm.install_render_factory(&mut factory)
        .map_err(|e| format!("factory: {e}"))?;
    vm.install_rive_globals().map_err(|e| format!("globals: {e}"))?;
    if let Some(asset) = &header.view_models {
        let path = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets")
            .join(asset);
        let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let mut file_factory = PersistentFactory::new(NullFactory::new());
        let handle = RuntimeFactoryHandle::from_factory(&mut file_factory)
            .ok_or("no runtime factory handle")?;
        let file = File::import(&bytes, handle, None, None, None)
            .ok_or_else(|| format!("import {asset} failed"))?;
        vm.set_view_models(nuxie_runtime::script_view_models(&file))
            .map_err(|e| format!("view models: {e}"))?;
    }
    Ok((vm, factory))
}

/// The case's own outcome: `Ok` is a normal return, `Err` a script error.
type CaseResult = Result<String, String>;

fn run_module(vm: &ScriptVm, name: &str, bytecode: &[u8]) -> CaseResult {
    vm.run_bytecode::<Value>(name, bytecode)
        .map(|value| format!("{value:?}").chars().take(120).collect())
        .map_err(|e| e.to_string())
}

fn run_instance(
    vm: &ScriptVm,
    factory: &mut PersistentFactory<NullFactory>,
    name: &str,
    bytecode: &[u8],
) -> CaseResult {
    let generator: Function = vm.run_bytecode(name, bytecode).map_err(|e| e.to_string())?;
    let table: Table = generator.call(Value::Nil).map_err(|e| e.to_string())?;
    let mut instance = vm.script_instance_from_table(table);
    let mut host = NoopScriptHost;
    let text = |e: nuxie_runtime::ScriptError| e.to_string();
    instance
        .call_optional_method(ScriptMethod::Init, &[], &mut host)
        .map_err(text)?;
    let draws = instance.has_method(ScriptMethod::Draw).map_err(text)?;
    for _ in 0..2 {
        if draws {
            let mut renderer = factory.borrow().make_renderer();
            instance
                .call_draw(factory, &mut renderer, &mut host)
                .map_err(text)?;
        }
        instance
            .call_optional_method(ScriptMethod::Advance, &[ScriptValue::Number(1.0 / 60.0)], &mut host)
            .map_err(text)?;
        instance
            .call_optional_method(ScriptMethod::Update, &[], &mut host)
            .map_err(text)?;
    }
    Ok("instance ran two frames".into())
}

/// The variants a `bytecode:` directive makes from a valid chunk.
fn bytecode_variants(directive: &str, chunk: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let words: Vec<&str> = directive.split_whitespace().collect();
    let byte = |text: &str| -> Result<u8, String> {
        let digits = text.strip_prefix("0x").unwrap_or(text);
        u8::from_str_radix(digits, 16).map_err(|_| format!("bad byte {text:?}"))
    };
    let hex = |text: &str| -> Result<Vec<u8>, String> {
        let digits: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
        digits
            .chunks(2)
            .map(|pair| byte(&pair.iter().collect::<String>()))
            .collect()
    };
    let mut out = Vec::new();
    match words.as_slice() {
        // Every proper prefix, including the empty chunk.
        ["truncate-all"] => {
            for len in 0..chunk.len() {
                out.push((format!("prefix {len}"), chunk[..len].to_vec()));
            }
        }
        // Each offset in turn set to one value.
        ["every-byte", value] => {
            let value = byte(value)?;
            for at in 0..chunk.len() {
                let mut variant = chunk.to_vec();
                variant[at] = value;
                out.push((format!("byte {at} = {value:#04x}"), variant));
            }
        }
        // Each offset in turn with its bits flipped.
        ["every-byte-flipped"] => {
            for at in 0..chunk.len() {
                let mut variant = chunk.to_vec();
                variant[at] = !variant[at];
                out.push((format!("byte {at} flipped"), variant));
            }
        }
        // The version byte through every value.
        ["every-version"] => {
            for version in 0..=u8::MAX {
                let mut variant = chunk.to_vec();
                if let Some(first) = variant.first_mut() {
                    *first = version;
                }
                out.push((format!("version {version}"), variant));
            }
        }
        // The chunk with bytes appended.
        ["append", bytes] => {
            let mut variant = chunk.to_vec();
            variant.extend(hex(bytes)?);
            out.push(("appended".into(), variant));
        }
        // Each LEB128 varint in the chunk (a byte with the high bit set and
        // its continuation) replaced by the 5-byte encoding of u32::MAX, and
        // separately by an endless continuation run.
        ["every-varint-max"] => {
            let mut at = 0;
            while at < chunk.len() {
                let mut end = at;
                while end < chunk.len() && chunk[end] & 0x80 != 0 {
                    end += 1;
                }
                let end = (end + 1).min(chunk.len());
                for (label, replacement) in [
                    ("u32::MAX", vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
                    ("unterminated", vec![0xff; 16]),
                ] {
                    let mut variant = chunk[..at].to_vec();
                    variant.extend(&replacement);
                    variant.extend(&chunk[end..]);
                    out.push((format!("varint at {at} = {label}"), variant));
                }
                at = end;
            }
        }
        // A literal chunk, replacing the compiled one.
        ["hex", rest @ ..] => out.push(("literal".into(), hex(&rest.concat())?)),
        // A `.riv` ScriptAsset payload (signed-content envelope) as hex.
        ["payload-hex", rest @ ..] => out.push(("payload".into(), hex(&rest.concat())?)),
        // The compiled chunk behind a literal envelope prefix.
        ["payload-prefix", rest @ ..] => {
            let mut variant = hex(&rest.concat())?;
            variant.extend_from_slice(chunk);
            out.push(("prefixed payload".into(), variant));
        }
        _ => return Err(format!("unknown bytecode directive {directive:?}")),
    }
    Ok(out)
}

fn run_bytecode_case(header: &Header, name: &str, chunk: &[u8]) -> CaseResult {
    let directive = header.bytecode.as_deref().unwrap_or_default();
    let payload = directive.starts_with("payload-");
    let mut variants = bytecode_variants(directive, chunk).map_err(|e| format!("harness: {e}"))?;
    // The unchanged chunk goes first and must run, so the variants are
    // known to reach past the loader's first checks.
    let control = if payload { [&[0u8][..], chunk].concat() } else { chunk.to_vec() };
    variants.insert(0, ("control".into(), control));
    let (mut ok, mut errors) = (0usize, 0usize);
    let mut refusals: BTreeSet<String> = BTreeSet::new();
    for (index, (label, bytes)) in variants.iter().enumerate() {
        let (vm, _factory) = new_vm(header)?;
        let loaded = if payload {
            vm.load_script_asset_payload(name, bytes)
        } else {
            vm.load_module(name, bytes)
        };
        let result = loaded.and_then(|thread| vm.execute_module::<Value>(name, thread));
        match result {
            Ok(_) => ok += 1,
            Err(error) => {
                let text = error.to_string();
                if text.contains("rust panic") || index == 0 {
                    return Err(format!("{label}: {text}"));
                }
                errors += 1;
                if refusals.len() < 3 {
                    refusals.insert(text.chars().take(80).collect());
                }
            }
        }
        // Collection and drop of each variant's VM are part of the case.
        let _ = vm.lua().gc_collect();
        drop(vm);
    }
    // Each variant must end in a load error, a script error or a normal
    // return; which one is not the contract, so the case returns normally.
    let refusals: Vec<String> = refusals.into_iter().collect();
    Ok(format!(
        "{} variants with the control: {ok} ran, {errors} refused ({})",
        variants.len(),
        refusals.join(" / ")
    ))
}

fn run_child(case: &str) {
    let path = corpus_dir().join(case);
    let source = std::fs::read_to_string(&path).expect("case source");
    let header = parse_header(&source).expect("case header");
    let name = case.trim_end_matches(".luau").to_owned();
    let chunk = support::compile_source(&source).expect("the corpus compiles");
    let header_in_thread = header.clone();
    // The case runs on a thread of a stated stack size; its VM is built,
    // run, collected and dropped there, so a crash in any of them is the
    // case's.
    let worker = std::thread::Builder::new()
        .name("hostile-case".into())
        .stack_size(CASE_STACK_BYTES)
        .spawn(move || {
            let header = header_in_thread;
            let result = if header.harness == Harness::Bytecode {
                run_bytecode_case(&header, &name, &chunk)
            } else {
                match new_vm(&header) {
                    Err(error) => Err(format!("harness: {error}")),
                    Ok((vm, mut factory)) => {
                        let result = match header.harness {
                            Harness::Instance => run_instance(&vm, &mut factory, &name, &chunk),
                            _ => run_module(&vm, &name, &chunk),
                        };
                        let collected = vm.lua().gc_collect();
                        drop(vm);
                        drop(factory);
                        match (result, collected) {
                            (Ok(text), Err(e)) => Err(format!("{text}; collect: {e}")),
                            (result, _) => result,
                        }
                    }
                }
            };
            let line = match &result {
                Ok(text) => format!("ok {}", one_line(text)),
                Err(text) => format!("error {}", one_line(text)),
            };
            if std::env::var_os(VERBOSE_ENV).is_some() {
                println!("HOSTILE-FULL {result:?}");
            }
            println!("{OUTCOME_PREFIX}{line}");
            println!("{DROPPED_LINE}");
        })
        .expect("spawn the case thread");
    worker.join().expect("the case thread ran to its end");
}

fn one_line(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(400)
        .collect()
}

// ------------------------------------------------------------ the parent

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    /// The child died: a signal, an abort, or libtest reporting a panic.
    Crash,
    /// A host panic surfaced as a `rust panic:` script error.
    Panic,
    Timeout,
    Memory,
    /// A clean exit with the wrong outcome.
    Outcome,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Crash => "crash",
            Kind::Panic => "panic",
            Kind::Timeout => "timeout",
            Kind::Memory => "memory",
            Kind::Outcome => "outcome",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        [Kind::Crash, Kind::Panic, Kind::Timeout, Kind::Memory, Kind::Outcome]
            .into_iter()
            .find(|kind| kind.name() == text)
    }
}

struct Verdict {
    case: String,
    failure: Option<(Kind, String)>,
    outcome: String,
    elapsed: Duration,
    peak_kib: u64,
}

/// Pid, parent pid and resident KiB of every process, from `ps`.
fn process_table() -> Vec<(u32, u32, u64)> {
    let Ok(output) = Command::new("ps").args(["-A", "-o", "pid=,ppid=,rss="]).output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace().map(str::parse::<u64>);
            match (parts.next(), parts.next(), parts.next()) {
                (Some(Ok(pid)), Some(Ok(ppid)), Some(Ok(rss))) => {
                    Some((u32::try_from(pid).ok()?, u32::try_from(ppid).ok()?, rss))
                }
                _ => None,
            }
        })
        .collect()
}

/// The pids of `root` and all its descendants, and their summed RSS.
fn tree(root: u32) -> (Vec<u32>, u64) {
    let table = process_table();
    let mut pids = vec![root];
    let mut index = 0;
    while let Some(&parent) = pids.get(index) {
        pids.extend(table.iter().filter(|row| row.1 == parent).map(|row| row.0));
        index += 1;
    }
    let rss = table
        .iter()
        .filter(|row| pids.contains(&row.0))
        .map(|row| row.2)
        .sum();
    (pids, rss)
}

fn kill_tree(root: u32) {
    let (pids, _) = tree(root);
    let pids: Vec<String> = pids.iter().map(u32::to_string).collect();
    let _ = Command::new("kill").arg("-9").args(&pids).status();
}

fn drain(mut stream: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut buffer = [0u8; 8192];
        while let Ok(n) = stream.read(&mut buffer) {
            if n == 0 {
                break;
            }
            kept.extend_from_slice(&buffer[..n]);
            if kept.len() > 4 * KEPT_OUTPUT_BYTES {
                let cut = kept.len() - KEPT_OUTPUT_BYTES;
                kept.drain(..cut);
            }
        }
        kept
    })
}

/// The lines of a failed child's output that say what happened.
fn crash_summary(output: &str, status: &str) -> String {
    let mut lines: Vec<String> = output
        .lines()
        .filter(|line| {
            line.contains("panicked at")
                || line.contains("overflowed its stack")
                || line.contains("signal:")
                || line.contains("SIGSEGV")
                || line.contains("SIGBUS")
                || line.contains("SIGABRT")
                || line.contains("memory allocation of")
                || line.contains("fatal runtime error")
        })
        .map(|line| line.trim().to_owned())
        .collect();
    let message: Vec<String> = output
        .lines()
        .skip_while(|line| !line.contains("panicked at"))
        .skip(1)
        .take(2)
        .map(|line| line.trim().to_owned())
        .collect();
    lines.extend(message);
    // The first frames of our own crates name where the fault is.
    let frames: Vec<String> = output
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.chars().next().is_some_and(|c| c.is_ascii_digit())
                && (line.contains("nuxie_") || line.contains("luaur"))
                && !line.contains("hostile_corpus")
                && !line.contains("panicking")
        })
        .take(6)
        .map(|line| line.trim().to_owned())
        .collect();
    lines.extend(frames);
    lines.dedup();
    if lines.is_empty() {
        format!("{status}; no panic message")
    } else {
        format!("{status}; {}", lines.join(" | "))
    }
}

fn judge(case: &str, header: &Header) -> Verdict {
    let started = Instant::now();
    let mut child = Command::new(std::env::current_exe().expect("test binary"))
        .args(["hostile_corpus", "--exact", "--nocapture", "--test-threads=1"])
        .env(CASE_ENV, case)
        .env("RUST_BACKTRACE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the case child");
    let pid = child.id();
    let out = drain(child.stdout.take().expect("stdout"));
    let err = drain(child.stderr.take().expect("stderr"));
    let mut peak_kib = 0;
    let mut killed = None;
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait for the case") {
            break Some(status);
        }
        let (_, rss) = tree(pid);
        peak_kib = peak_kib.max(rss);
        if rss > MEMORY_CEILING_KIB {
            killed = Some(Kind::Memory);
        } else if started.elapsed() > header.within {
            killed = Some(Kind::Timeout);
        }
        if killed.is_some() {
            kill_tree(pid);
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(SAMPLE_INTERVAL);
    };
    let elapsed = started.elapsed();
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.join().unwrap_or_default()),
        String::from_utf8_lossy(&err.join().unwrap_or_default())
    );
    if std::env::var_os(VERBOSE_ENV).is_some() {
        println!("---- {case} child output ----\n{output}\n---- end {case} ----");
    }
    let outcome = output
        .lines()
        // libtest's `test hostile_corpus ... ` can precede it on its line.
        .find_map(|line| line.find(OUTCOME_PREFIX).map(|at| &line[at + OUTCOME_PREFIX.len()..]))
        .unwrap_or("")
        .to_owned();
    let verdict = |failure| Verdict {
        case: case.to_owned(),
        failure,
        outcome: outcome.clone(),
        elapsed,
        peak_kib,
    };
    match (killed, status) {
        (Some(Kind::Memory), _) => {
            return verdict(Some((Kind::Memory, format!("RSS {peak_kib} KiB past {MEMORY_CEILING_KIB} KiB"))));
        }
        (Some(kind), _) => {
            return verdict(Some((kind, format!("killed after {:.1}s", elapsed.as_secs_f64()))));
        }
        (None, Some(status)) if !status.success() || !output.contains(DROPPED_LINE) || outcome.is_empty() => {
            return verdict(Some((Kind::Crash, crash_summary(&output, &status.to_string()))));
        }
        _ => {}
    }
    // A host panic is a failure even when a script's pcall swallowed the
    // error the trampoline made of it: the panic hook still printed it.
    if outcome.contains("rust panic") || output.contains("panicked at") {
        let detail = if outcome.contains("rust panic") {
            outcome.clone()
        } else {
            crash_summary(&output, "caught host panic")
        };
        return verdict(Some((Kind::Panic, detail)));
    }
    let (is_ok, text) = match outcome.split_once(' ') {
        Some(("ok", text)) => (true, text),
        Some(("error", text)) => (false, text),
        _ => (outcome.starts_with("ok"), ""),
    };
    let matches = match &header.expect {
        Expect::Either => true,
        Expect::Ok => is_ok,
        Expect::Error(None) => !is_ok,
        Expect::Error(Some(needle)) => !is_ok && text.contains(needle.as_str()),
    };
    if matches {
        verdict(None)
    } else {
        verdict(Some((Kind::Outcome, format!("expected {:?}, got: {outcome}", header.expect))))
    }
}

struct ExpectedFailure {
    kind: Kind,
    owner: String,
}

/// `case | strategy | kind | owner | reason`, one per line; `#` comments.
fn expected_failures() -> BTreeMap<String, ExpectedFailure> {
    let path = corpus_dir().join("expected-failures.txt");
    let text = std::fs::read_to_string(&path).expect("expected-failures.txt");
    let mut out = BTreeMap::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.splitn(5, '|').map(str::trim).collect();
        let [case, strategy_field, kind, owner, reason] = fields.as_slice() else {
            panic!("expected-failures.txt:{}: want 5 `|` fields", number + 1);
        };
        assert!(
            ["both", "abort", "unwind"].contains(strategy_field),
            "expected-failures.txt:{}: strategy is both, abort or unwind",
            number + 1
        );
        assert!(
            ["H1", "H2", "H3"].contains(owner),
            "expected-failures.txt:{}: owner is H1, H2 or H3",
            number + 1
        );
        assert!(!reason.is_empty(), "expected-failures.txt:{}: no reason", number + 1);
        let kind = Kind::parse(kind)
            .unwrap_or_else(|| panic!("expected-failures.txt:{}: unknown kind {kind}", number + 1));
        if *strategy_field == "both" || *strategy_field == strategy() {
            let previous = out.insert(
                (*case).to_owned(),
                ExpectedFailure { kind, owner: (*owner).to_owned() },
            );
            assert!(previous.is_none(), "expected-failures.txt:{}: {case} listed twice", number + 1);
        }
    }
    out
}

#[test]
fn hostile_corpus() {
    if let Ok(case) = std::env::var(CASE_ENV) {
        run_child(&case);
        return;
    }
    let all = list_cases();
    assert!(!all.is_empty(), "no cases under {}", corpus_dir().display());
    let expected = expected_failures();
    let known: BTreeSet<&String> = all.iter().collect();
    let missing: Vec<&String> = expected.keys().filter(|case| !known.contains(case)).collect();
    assert!(missing.is_empty(), "expected-failures.txt names cases that do not exist: {missing:?}");

    let filter = std::env::var(FILTER_ENV).ok();
    let cases: Vec<(String, Header)> = all
        .iter()
        .filter(|case| filter.as_deref().is_none_or(|f| case.contains(f)))
        .map(|case| {
            let source = std::fs::read_to_string(corpus_dir().join(case)).expect("case source");
            let header =
                parse_header(&source).unwrap_or_else(|error| panic!("{case}: header: {error}"));
            assert!(!header.attacks.is_empty(), "{case}: empty `attacks`");
            (case.clone(), header)
        })
        .collect();
    assert!(
        !cases.is_empty(),
        "{FILTER_ENV}={} selects none of the {} cases",
        filter.as_deref().unwrap_or(""),
        all.len()
    );
    println!(
        "hostile corpus: {} of {} cases, panic={}, {} at a time",
        cases.len(),
        all.len(),
        strategy(),
        PARALLEL_CASES
    );

    let queue = Arc::new(Mutex::new(cases.into_iter().rev().collect::<Vec<_>>()));
    let verdicts = Arc::new(Mutex::new(Vec::new()));
    let workers: Vec<_> = (0..PARALLEL_CASES)
        .map(|_| {
            let queue = Arc::clone(&queue);
            let verdicts = Arc::clone(&verdicts);
            std::thread::spawn(move || {
                loop {
                    let next = queue.lock().expect("queue").pop();
                    let Some((case, header)) = next else { break };
                    let verdict = judge(&case, &header);
                    verdicts.lock().expect("verdicts").push(verdict);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker");
    }
    let mut verdicts = std::mem::take(&mut *verdicts.lock().expect("verdicts"));
    verdicts.sort_by(|a, b| a.case.cmp(&b.case));

    let mut unexpected = Vec::new();
    let mut fixed = Vec::new();
    let mut wrong_kind = Vec::new();
    for verdict in &verdicts {
        let listed = expected.get(&verdict.case);
        let status = match (&verdict.failure, listed) {
            (None, None) => "pass".to_owned(),
            (None, Some(entry)) => {
                fixed.push(format!("{} (was {} for {})", verdict.case, entry.kind.name(), entry.owner));
                "PASSES BUT LISTED".to_owned()
            }
            (Some((kind, detail)), Some(entry)) if *kind == entry.kind => {
                format!("expected {} ({}): {detail}", kind.name(), entry.owner)
            }
            (Some((kind, detail)), Some(entry)) => {
                wrong_kind.push(format!(
                    "{}: listed as {}, now {}: {detail}",
                    verdict.case,
                    entry.kind.name(),
                    kind.name()
                ));
                format!("WRONG KIND {}: {detail}", kind.name())
            }
            (Some((kind, detail)), None) => {
                unexpected.push(format!("{}: {}: {detail}", verdict.case, kind.name()));
                format!("FAIL {}: {detail}", kind.name())
            }
        };
        println!(
            "{:<58} {:>6.2}s {:>7} KiB  {status}  [{}]",
            verdict.case,
            verdict.elapsed.as_secs_f64(),
            verdict.peak_kib,
            verdict.outcome.chars().take(90).collect::<String>()
        );
    }
    let peak = verdicts.iter().map(|v| v.peak_kib).max().unwrap_or(0);
    let slowest = verdicts
        .iter()
        .max_by_key(|v| v.elapsed)
        .map(|v| format!("{} {:.2}s", v.case, v.elapsed.as_secs_f64()))
        .unwrap_or_default();
    println!(
        "hostile corpus (panic={}): {} cases, {} expected failures seen, {} unexpected, {} listed but passing, {} wrong kind; peak RSS {peak} KiB; slowest {slowest}",
        strategy(),
        verdicts.len(),
        verdicts.iter().filter(|v| v.failure.is_some() && expected.contains_key(&v.case)).count(),
        unexpected.len(),
        fixed.len(),
        wrong_kind.len(),
    );
    let mut problems = Vec::new();
    problems.extend(unexpected.iter().map(|line| format!("unexpected failure: {line}")));
    problems.extend(fixed.iter().map(|line| format!("listed failure now passes, remove its line: {line}")));
    problems.extend(wrong_kind.iter().map(|line| format!("listed failure changed kind: {line}")));
    assert!(problems.is_empty(), "hostile corpus gate:\n{}", problems.join("\n"));
}
