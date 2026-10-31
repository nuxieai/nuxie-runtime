//! Everything that touches a Luau VM lives here, so another VM can be added
//! by writing one more `BenchVm` implementation and one more arm in `by_name`.
//!
//! The contract for an implementation: run one merged bench script (upstream
//! script with `bench_support.lua` inlined, see `tests/luau-bench/merge.py`)
//! in a fresh VM state and return every line the script printed. The state
//! must be sandboxed the way the C++ `luau` CLI sandboxes it
//! (`luaL_sandbox` + `luaL_sandboxthread`): builtin fastcalls only engage
//! under `safeenv`, so an unsandboxed run measures a slower VM than the one
//! that ships. The script needs `print`, `os.clock` and, if available,
//! `collectgarbage()` (bench_support calls it between timed runs).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

pub trait BenchVm {
    /// Short identifier recorded in result files, for example `luaur`.
    fn name(&self) -> &'static str;

    /// Runs one merged bench script in a fresh, sandboxed state and returns
    /// the lines it printed. An `Err` carries the VM's error message.
    fn run_script(&self, chunk_name: &str, source: &str) -> Result<Vec<String>, String>;
}

/// Names accepted by `--vm`.
pub const VM_NAMES: &[&str] = &["luaur"];

pub fn by_name(name: &str) -> Option<Box<dyn BenchVm>> {
    match name {
        "luaur" => Some(Box::new(Luaur)),
        _ => None,
    }
}

/// The vendored luaur fork, driven through `luaur-rt` exactly as the
/// 2026-10-31 bake-off runner drove it.
pub struct Luaur;

impl BenchVm for Luaur {
    fn name(&self) -> &'static str {
        "luaur"
    }

    fn run_script(&self, chunk_name: &str, source: &str) -> Result<Vec<String>, String> {
        let output = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&output);
        let outcome = catch_unwind(AssertUnwindSafe(|| luaur_run(chunk_name, source, captured)));
        let lines = output.lock().map(|lines| lines.clone()).unwrap_or_default();
        match outcome {
            Ok(Ok(())) => Ok(lines),
            Ok(Err(error)) => Err(error
                .to_string()
                .lines()
                .next()
                .unwrap_or("error")
                .to_string()),
            Err(_) => Err("host panic while running the script".to_string()),
        }
    }
}

fn luaur_run(
    chunk_name: &str,
    source: &str,
    output: Arc<Mutex<Vec<String>>>,
) -> luaur_rt::Result<()> {
    use luaur_rt::{Error, Lua, MultiValue, Value};

    let lua = Lua::new();
    let globals = lua.globals();
    // The Rive profile registers neither `collectgarbage` nor `print`.
    globals.set(
        "collectgarbage",
        lua.create_function(
            |lua, (option, arg): (Option<String>, Option<f64>)| -> luaur_rt::Result<Value> {
                match option.as_deref().unwrap_or("collect") {
                    "collect" => {
                        lua.gc_collect()?;
                        Ok(Value::Number(0.0))
                    }
                    "count" => Ok(Value::Number(lua.used_memory() as f64 / 1024.0)),
                    "step" => Ok(Value::Boolean(match arg {
                        Some(kbytes) if kbytes > 0.0 => lua.gc_step_kbytes(kbytes as i32)?,
                        _ => lua.gc_step()?,
                    })),
                    other => Err(Error::runtime(format!(
                        "collectgarbage: unsupported option '{other}'"
                    ))),
                }
            },
        )?,
    )?;
    globals.set(
        "print",
        lua.create_function(move |_, args: MultiValue| {
            let parts: Vec<String> = args
                .into_vec()
                .iter()
                .map(|value| match value {
                    Value::String(text) => text
                        .to_str()
                        .map(|text| text.to_string())
                        .unwrap_or_default(),
                    Value::Number(number) => format!("{number}"),
                    Value::Integer(integer) => format!("{integer}"),
                    Value::Boolean(flag) => format!("{flag}"),
                    Value::Nil => "nil".to_string(),
                    other => other.type_name().to_string(),
                })
                .collect();
            if let Ok(mut lines) = output.lock() {
                lines.push(parts.join("\t"));
            }
            Ok(())
        })?,
    )?;
    lua.sandbox(true)?;
    // Upstream runs each script on a fresh thread (luaL_sandboxthread), so
    // the chunk sees a yieldable coroutine.
    let function = lua.load(source).set_name(chunk_name).into_function()?;
    let thread = lua.create_thread(function)?;
    thread.resume::<Value>(())?;
    Ok(())
}
