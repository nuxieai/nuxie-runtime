//! The same harness as `harness.rs`, for `nuxie-luau-vm` (`--vm nuxie`;
//! plan `docs/luau-safe-vm-plan.md`, Phase 2, lane 1).
//!
//! What it does, matching `harness.rs` wherever the new VM allows:
//!
//! - Base libraries, then globals `collectgarbage`, `loadstring` and a
//!   silent `print` (`--verbose` prints `tostring` of each argument), and
//!   for the scripts upstream gives `setupNativeHelpers`, `is_native`
//!   (false) and `is_native_if_supported` (true).
//! - `luaL_sandbox` + `luaL_sandboxthread` (`Vm::sandbox`, which also binds
//!   `_G` to the writable proxy).
//! - The script is compiled by the vendored compiler with the options
//!   `harness.rs` uses (optimization 1, debug 1, type info 1, float vector
//!   precision, every Luau flag on as `luaur-rt` sets them) and run on the
//!   main state
//!   until it returns; it passes iff the final returned value is "OK".
//!
//! Differences from `harness.rs`:
//!
//! - **The script runs on the main thread**, as upstream runs it
//!   (`lua_resume(L, nullptr, 0)`, so the top level can yield:
//!   `Vm::start`); the `luaur-rt` harness has to use a coroutine. A yield
//!   out of the script fails it as "thread yielded unexpectedly", except
//!   for the scripts upstream gives a yield callback (`YIELD_SCRIPTS`),
//!   which are resumed. Upstream's callback for `ndebug_upvalues` also
//!   reads the yielding function's upvalues through `lua_getupvalue`; the
//!   host API has no upvalue access yet (lane 6), so that check is not run.
//! - **`collectgarbage`** is upstream's `lua_collectgarbage` over the VM's
//!   incremental collector: `step` runs `lua_gc(LUA_GCSTEP, n)` and
//!   reports whether a cycle finished, `count` is the KiB in use (the
//!   budget's count, not C++'s allocation sizes), and the tuning options
//!   return the previous setting, as upstream (the luaur harness cannot).
//! - **`setblockallocations(b)`** (gc.luau) makes every allocation fail
//!   while set, through `HostCtx::set_block_allocations`.
//!
//! The chunk-name handling (`loadstring` passes upstream's rendering of
//! the name, prefixed with `=`) is the same as `harness.rs`, for the same
//! reasons. Sources, and `loadstring` arguments, are compiled as bytes, as
//! upstream compiles them.

use luaur_ast::records::parse_options::ParseOptions;
use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
use luaur_compiler::functions::compile::compile_bytes as compiler_compile_bytes;
use luaur_compiler::records::compile_options::CompileOptions;
use nuxie_luau_vm::{GcTune, HostCtx, HostError, HostValue, RunState, Vm};

use crate::harness::{NATIVE_HELPER_SCRIPTS, Outcome, chunkid, first_line};

/// Scripts upstream runs with a yield callback (`runConformance`'s
/// `yield`): resumed after each yield.
const YIELD_SCRIPTS: &[&str] = &["ndebug_upvalues.luau"];

/// Run one conformance script to completion on a fresh VM.
pub fn run_script(name: &str, source: &[u8], verbose: bool) -> Outcome {
    match run(name, source, verbose) {
        Ok(outcome) => outcome,
        Err(message) => Outcome::Fail(first_line(&message)),
    }
}

fn run(name: &str, source: &[u8], verbose: bool) -> Result<Outcome, String> {
    // `luaur_rt::Lua::new` does this before `harness.rs` compiles anything.
    luaur_common::set_all_flags(true);
    let mut vm = Vm::new().map_err(|e| e.message())?;
    install_globals(&mut vm, verbose).map_err(|e| e.message())?;
    if NATIVE_HELPER_SCRIPTS.contains(&name) {
        vm.register("is_native", |ctx| push(ctx, HostValue::Boolean(false)))
            .map_err(|e| e.message())?;
        vm.register("is_native_if_supported", |ctx| {
            push(ctx, HostValue::Boolean(true))
        })
        .map_err(|e| e.message())?;
    }
    vm.sandbox().map_err(|e| e.message())?;

    let chunkname = format!("={name}");
    let bytecode = match compile(source, 1) {
        Ok(bytecode) => bytecode,
        Err(message) => {
            return Ok(Outcome::Fail(first_line(&format!(
                "{}{message}",
                chunkid(chunkname.as_bytes())
            ))));
        }
    };
    let mut state = vm
        .start(chunkname.as_bytes(), &bytecode)
        .map_err(|e| e.message())?;
    let returned = loop {
        match state {
            RunState::Finished(values) => break values,
            RunState::Yielded(_) if YIELD_SCRIPTS.contains(&name) => {
                state = vm.resume(&[]).map_err(|e| e.message())?;
            }
            RunState::Yielded(_) => return Ok(Outcome::Fail("thread yielded unexpectedly".into())),
        }
    };
    Ok(match returned.last() {
        Some(HostValue::String(text)) if text == b"OK" => Outcome::Pass,
        Some(other) => Outcome::Fail(format!("returned {} instead of \"OK\"", describe(other))),
        None => Outcome::Fail("returned nothing instead of \"OK\"".into()),
    })
}

struct NoopEncoder;

impl BytecodeEncoder for NoopEncoder {
    fn encode(&mut self, _data: &mut [u32]) {}
}

/// The vendored compiler with `harness.rs`'s options (`loadstring` passes
/// optimization level 1 too: upstream compiles it with default options).
fn compile(source: &[u8], optimization_level: core::ffi::c_int) -> Result<Vec<u8>, String> {
    let options = CompileOptions {
        optimization_level,
        debug_level: 1,
        type_info_level: 1,
        vector_precision: 0,
        ..CompileOptions::default()
    };
    let mut encoder = NoopEncoder;
    let bytes = compiler_compile_bytes(
        source,
        &options,
        &ParseOptions::default(),
        &mut encoder as *mut dyn BytecodeEncoder,
    )
    .into_bytes();
    match bytes.split_first() {
        Some((0, message)) => Err(String::from_utf8_lossy(message).into_owned()),
        Some(_) => Ok(bytes),
        None => Err("the compiler returned nothing".to_owned()),
    }
}

fn push(ctx: &mut HostCtx<'_>, value: HostValue) -> Result<usize, HostError> {
    ctx.push(value)?;
    Ok(1)
}

fn install_globals(vm: &mut Vm, verbose: bool) -> Result<(), nuxie_luau_vm::Error> {
    vm.register("collectgarbage", collectgarbage)?;
    vm.register("setblockallocations", |ctx| {
        let blocked = matches!(ctx.arg(1), Some(HostValue::Boolean(true)));
        ctx.set_block_allocations(blocked);
        Ok(0)
    })?;
    vm.register("loadstring", loadstring)?;
    vm.register("print", move |ctx| {
        if verbose {
            let mut line = Vec::new();
            for n in 1..=ctx.arg_count() {
                if n > 1 {
                    line.push(b'\t');
                }
                line.extend(ctx.tostring_value(n)?);
            }
            line.push(b'\n');
            use std::io::Write;
            let _ = std::io::stdout().lock().write_all(&line);
        }
        Ok(0)
    })?;
    Ok(())
}

/// Upstream `lua_collectgarbage`; see the module comment for how the
/// options map onto a stop-the-world collector.
fn collectgarbage(ctx: &mut HostCtx<'_>) -> Result<usize, HostError> {
    let option = ctx
        .opt_string(1)?
        .map(|o| String::from_utf8_lossy(&o).into_owned())
        .unwrap_or_else(|| "collect".to_owned());
    let extra = ctx.opt_integer(2, 0)?;
    let value = match option.as_str() {
        "stop" => {
            ctx.set_gc_running(false);
            HostValue::Number(0.0)
        }
        "restart" => {
            ctx.set_gc_running(true);
            HostValue::Number(0.0)
        }
        "collect" => {
            ctx.collect();
            HostValue::Number(0.0)
        }
        "count" => HostValue::Number((ctx.memory_bytes() >> 10) as f64),
        "isrunning" => HostValue::Boolean(ctx.gc_running()),
        "step" => HostValue::Boolean(ctx.gc_step(extra)),
        "setgoal" => HostValue::Number(f64::from(ctx.gc_tune(GcTune::Goal, extra))),
        "setstepmul" => HostValue::Number(f64::from(ctx.gc_tune(GcTune::StepMul, extra))),
        "setstepsize" => HostValue::Number(f64::from(ctx.gc_tune(GcTune::StepSize, extra))),
        other => {
            return Err(HostError::new(format!(
                "invalid argument #1 to 'collectgarbage' (invalid option '{other}')"
            )));
        }
    };
    push(ctx, value)
}

/// Upstream `lua_loadstring`: compile `s` under `chunkname` (default `s`),
/// return the function or `nil, message`.
fn loadstring(ctx: &mut HostCtx<'_>) -> Result<usize, HostError> {
    let source = ctx.check_string(1)?;
    let chunkname = ctx.opt_string(2)?.unwrap_or_else(|| source.clone());
    let rendered = chunkid(&chunkname);
    match compile(&source, 1) {
        Ok(bytecode) => match ctx.load(format!("={rendered}").as_bytes(), &bytecode)? {
            Ok(()) => Ok(1),
            Err(error) => {
                ctx.push(HostValue::Nil)?;
                ctx.push(HostValue::String(error.to_string().into_bytes()))?;
                Ok(2)
            }
        },
        Err(message) => {
            ctx.push(HostValue::Nil)?;
            ctx.push(HostValue::String(
                format!("{rendered}{message}").into_bytes(),
            ))?;
            Ok(2)
        }
    }
}

fn describe(value: &HostValue) -> String {
    match value {
        HostValue::String(text) => format!("\"{}\"", String::from_utf8_lossy(text)),
        HostValue::Boolean(flag) => flag.to_string(),
        HostValue::Number(number) => nuxie_luau_vm::format_number(*number),
        other => other.type_name().to_owned(),
    }
}
