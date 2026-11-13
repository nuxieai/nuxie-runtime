//! Everything that touches a Luau VM lives here, so another VM can be added
//! by writing one more `BenchVm` implementation and one more arm in `by_name`.
//! The vendored luaur VM the suite first measured was deleted in Phase 3; its
//! last numbers stay in `tests/luau-bench/baseline-luaur.json`.
//!
//! The contract for an implementation: run one merged bench script (upstream
//! script with `bench_support.lua` inlined, see `tests/luau-bench/merge.py`)
//! in a fresh VM state and return every line the script printed. The state
//! must be sandboxed the way the C++ `luau` CLI sandboxes it
//! (`luaL_sandbox` + `luaL_sandboxthread`): builtin fastcalls only engage
//! under `safeenv`, so an unsandboxed run measures a slower VM than the one
//! that ships. The script needs `print`, `os.clock` and, if available,
//! `collectgarbage()` (bench_support calls it between timed runs). A VM
//! also installs `__bench_atomic_load(index)` for nuxie's audio-block benches
//! (`tests/luau-bench/nuxie`): a host function reading [`SHARED_WORDS`].

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// The words `__bench_atomic_load` reads: the controls an audio processor
/// reads from a shared region each block (frequency in Hz, gain in
/// thousandths, then fourteen unused controls).
pub static SHARED_WORDS: [AtomicU32; 16] = [
    AtomicU32::new(440),
    AtomicU32::new(250),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

/// `__bench_atomic_load(index)`: word `index`, 0 when out of range.
fn shared_word(index: f64) -> f64 {
    let word = if index >= 0.0 { SHARED_WORDS.get(index as usize) } else { None };
    word.map_or(0.0, |w| f64::from(w.load(Ordering::SeqCst)))
}

/// What the scripts' `os.clock` reads. With `LUAU_BENCH_CLOCK=instructions`
/// the VM gets a host `os.clock` returning this process's retired
/// instructions divided by 1e9, so every reported "ms" is a count of
/// millions of retired instructions (user and kernel, macOS
/// `proc_pid_rusage`). Measurement only: robust to a loaded host, where wall
/// time is not. Unset, `os.clock` is wall seconds since the state was
/// created.
pub fn instruction_clock() -> bool {
    std::env::var("LUAU_BENCH_CLOCK").is_ok_and(|clock| clock == "instructions")
}

/// Retired instructions for this process so far; 0 where unavailable.
///
/// macOS `proc_pid_rusage(getpid(), RUSAGE_INFO_V4, ...)`, declared here
/// rather than through the `libc` crate so the bench tool keeps its
/// dependency set (and the Bazel lockfiles) unchanged. `rusage_info_v4` is a
/// 16-byte uuid followed by 35 `u64` counters; `ri_instructions` is the
/// 30th counter, so word 31 of the buffer.
pub fn process_instructions() -> u64 {
    #[cfg(target_os = "macos")]
    {
        const RUSAGE_INFO_V4: i32 = 4;
        const WORDS: usize = 2 + 35;
        const RI_INSTRUCTIONS: usize = 2 + 29;
        unsafe extern "C" {
            fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut u64) -> i32;
        }
        let mut info = [0u64; WORDS];
        let Ok(pid) = i32::try_from(std::process::id()) else {
            return 0;
        };
        // SAFETY: RUSAGE_INFO_V4 fills exactly a rusage_info_v4, which is
        // WORDS u64s (8-byte aligned, 296 bytes); `info` is that large.
        let status = unsafe { proc_pid_rusage(pid, RUSAGE_INFO_V4, info.as_mut_ptr()) };
        if status == 0 {
            return info[RI_INSTRUCTIONS];
        }
    }
    0
}

fn instruction_seconds() -> f64 {
    process_instructions() as f64 / 1e9
}

pub trait BenchVm {
    /// Short identifier recorded in result files, for example `nuxie`.
    fn name(&self) -> &'static str;

    /// Runs one merged bench script in a fresh, sandboxed state and returns
    /// the lines it printed. An `Err` carries the VM's error message.
    fn run_script(&self, chunk_name: &str, source: &str) -> Result<Vec<String>, String>;
}

/// Names accepted by `--vm`.
pub const VM_NAMES: &[&str] = &["nuxie"];

pub fn by_name(name: &str) -> Option<Box<dyn BenchVm>> {
    match name {
        "nuxie" => Some(Box::new(Nuxie)),
        _ => None,
    }
}

/// `nuxie-luau-vm` (plan Phase 2, lane 1): the script is compiled by the
/// vendored compiler with the options the vendored VM's runtime used (every
/// Luau flag on, optimization 1, debug 1) and runs on the sandboxed main state. Lane 1
/// has no `os` library, so `os.clock` is a host function installed in an
/// `os` table before the sandbox seals the globals. Numbers are recorded,
/// not enforced: the plan budgets speed, it does not gate on it.
pub struct Nuxie;

impl BenchVm for Nuxie {
    fn name(&self) -> &'static str {
        "nuxie"
    }

    fn run_script(&self, chunk_name: &str, source: &str) -> Result<Vec<String>, String> {
        let output = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&output);
        let outcome = catch_unwind(AssertUnwindSafe(|| nuxie_run(chunk_name, source, captured)));
        let lines = output.lock().map(|lines| lines.clone()).unwrap_or_default();
        match outcome {
            Ok(Ok(())) => Ok(lines),
            Ok(Err(error)) => Err(error.lines().next().unwrap_or("error").to_string()),
            Err(_) => Err("host panic while running the script".to_string()),
        }
    }
}

struct NoopEncoder;

impl luaur_bytecode::records::bytecode_encoder::BytecodeEncoder for NoopEncoder {
    fn encode(&mut self, _data: &mut [u32]) {}
}

fn nuxie_compile(source: &str) -> Result<Vec<u8>, String> {
    use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
    luaur_common::set_all_flags(true);
    let options = luaur_compiler::records::compile_options::CompileOptions::default();
    let mut encoder = NoopEncoder;
    let bytes = luaur_compiler::functions::compile::compile(
        &source.to_owned(),
        &options,
        &luaur_ast::records::parse_options::ParseOptions::default(),
        &mut encoder as *mut dyn BytecodeEncoder,
    )
    .into_bytes();
    match bytes.split_first() {
        Some((0, message)) => Err(String::from_utf8_lossy(message).into_owned()),
        Some(_) => Ok(bytes),
        None => Err("the compiler returned nothing".to_string()),
    }
}

fn nuxie_run(chunk_name: &str, source: &str, output: Arc<Mutex<Vec<String>>>) -> Result<(), String> {
    use nuxie_luau_vm::{HostError, HostValue, Vm};

    let message = |error: nuxie_luau_vm::Error| error.message();
    let mut vm = Vm::new().map_err(message)?;
    vm.register("collectgarbage", |ctx| {
        let option = ctx
            .opt_string(1)?
            .map(|o| String::from_utf8_lossy(&o).into_owned())
            .unwrap_or_else(|| "collect".to_string());
        let value = match option.as_str() {
            "collect" => {
                ctx.collect();
                HostValue::Number(0.0)
            }
            "count" => HostValue::Number(ctx.memory_bytes() as f64 / 1024.0),
            "step" => {
                ctx.collect();
                HostValue::Boolean(true)
            }
            other => {
                return Err(HostError::new(format!(
                    "collectgarbage: unsupported option '{other}'"
                )));
            }
        };
        ctx.push(value)?;
        Ok(1)
    })
    .map_err(message)?;
    vm.register("print", move |ctx| {
        let mut parts = Vec::new();
        for n in 1..=ctx.arg_count() {
            parts.push(String::from_utf8_lossy(&ctx.tostring_value(n)?).into_owned());
        }
        if let Ok(mut lines) = output.lock() {
            lines.push(parts.join("\t"));
        }
        Ok(0)
    })
    .map_err(message)?;
    let started = std::time::Instant::now();
    let instructions = instruction_clock();
    vm.register("__nuxie_os_clock", move |ctx| {
        let now = if instructions {
            instruction_seconds()
        } else {
            started.elapsed().as_secs_f64()
        };
        ctx.push(HostValue::Number(now))?;
        Ok(1)
    })
    .map_err(message)?;
    vm.register("__bench_atomic_load", |ctx| {
        let index = ctx.check_number(1)?;
        ctx.push(HostValue::Number(shared_word(index)))?;
        Ok(1)
    })
    .map_err(message)?;
    let prelude = nuxie_compile("os = { clock = __nuxie_os_clock }\n__nuxie_os_clock = nil")?;
    vm.run(b"=prelude", &prelude).map_err(message)?;
    vm.sandbox().map_err(message)?;
    let bytecode = nuxie_compile(source)?;
    vm.run(chunk_name.as_bytes(), &bytecode).map_err(message)?;
    Ok(())
}
