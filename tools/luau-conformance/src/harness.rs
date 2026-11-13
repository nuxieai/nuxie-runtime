//! The VM-facing half of the runner: upstream's `runConformance` harness
//! (luau-lang/luau `tests/Conformance.test.cpp`) on `nuxie-luau-vm` (plan
//! `docs/luau-safe-vm-plan.md`, Phase 2, lane 1).
//!
//! What upstream does, and what this does:
//!
//! - `luaL_openlibs`, then globals `collectgarbage`, `loadstring` and a
//!   silent `print` (`--verbose` prints `tostring` of each argument), and
//!   for the scripts upstream gives `setupNativeHelpers`, `is_native`
//!   (false) and `is_native_if_supported` (true), which is what upstream's
//!   helpers return without code generation. Every other per-script setup
//!   (vector and userdata metatables, C yield helpers, allocator hooks,
//!   debugger callbacks) is absent, and the scope file says which scripts
//!   need it.
//! - `luaL_sandbox` + `luaL_sandboxthread` (`Vm::sandbox`, which also binds
//!   `_G` to the writable proxy).
//! - The script is compiled by the vendored compiler with upstream's
//!   `defaultOptions()` (optimization 1, debug 1, type info 1, float vector
//!   precision, every Luau flag on), chunk name `=<script>`, and run on the
//!   main thread (`lua_resume(L, nullptr, 0)`, so the top level can yield:
//!   `Vm::start`) until it returns; it passes iff the final returned value
//!   is "OK". A yield out of the script fails it as "thread yielded
//!   unexpectedly", except for the scripts upstream gives a yield callback
//!   (`YIELD_SCRIPTS`), which are resumed. Upstream's callback for
//!   `ndebug_upvalues` also reads the yielding function's upvalues through
//!   `lua_getupvalue`; the host API has no upvalue access yet (lane 6), so
//!   that check is not run.
//! - **`collectgarbage`** is upstream's `lua_collectgarbage` over the VM's
//!   incremental collector: `step` runs `lua_gc(LUA_GCSTEP, n)` and
//!   reports whether a cycle finished, `count` is the KiB in use (the
//!   budget's count, not C++'s allocation sizes), and the tuning options
//!   return the previous setting, as upstream.
//! - **`setblockallocations(b)`** (gc.luau) makes every allocation fail
//!   while set, through `HostCtx::set_block_allocations`.
//! - **`loadstring` chunk names.** `loadstring(s, name)` passes upstream's
//!   rendering of the name (`luaO_chunkid`), prefixed with `=`, so
//!   `short_src` prints what upstream prints; raw `debug.info(f, "s")` /
//!   `source` still differs (`=[string "..."]`). Sources, and `loadstring`
//!   arguments, are compiled as bytes, as upstream compiles them.
//! - **No `luaC_validate`** after the run and no `limitedstack` global (only
//!   set upstream in ASAN/debug builds).

use luaur_ast::records::parse_options::ParseOptions;
use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
use luaur_compiler::functions::compile::compile_bytes as compiler_compile_bytes;
use luaur_compiler::records::compile_options::CompileOptions;
use nuxie_luau_vm::{GcTune, HostCtx, HostError, HostValue, RunState, Vm};

/// What a script did.
pub enum Outcome {
    /// Returned "OK".
    Pass,
    /// Failed to compile, raised, yielded, or returned something else. The
    /// message is the first line of the error.
    Fail(String),
}

/// Upstream's `LUA_IDSIZE`.
const LUA_IDSIZE: usize = 256;

/// The scripts whose upstream setup lambda calls `setupNativeHelpers`
/// (tests/Conformance.test.cpp at the pinned commit).
const NATIVE_HELPER_SCRIPTS: &[&str] = &[
    "buffers.luau",
    "integers.luau",
    "integers_regspill.luau",
    "iter.luau",
    "native.luau",
    "native_types.luau",
    "vector.luau",
    "vector_library.luau",
];

/// `luaO_chunkid` with upstream's `LUA_IDSIZE` buffer.
fn chunkid(source: &[u8]) -> String {
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    match source.first() {
        Some(b'=') => text(&source[1..source.len().min(LUA_IDSIZE)]),
        Some(b'@') if source.len() <= LUA_IDSIZE => text(&source[1..]),
        Some(b'@') => format!("...{}", text(&source[source.len() - (LUA_IDSIZE - 4)..])),
        _ => {
            let budget = LUA_IDSIZE - "[string \"...\"]".len() - 1;
            let line = source
                .iter()
                .position(|&byte| byte == b'\n' || byte == b'\r')
                .unwrap_or(source.len());
            let len = line.min(budget);
            if len < source.len() {
                format!("[string \"{}...\"]", text(&source[..len]))
            } else {
                format!("[string \"{}\"]", text(source))
            }
        }
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().to_owned()
}

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
    // Every Luau flag on, as upstream's conformance build has them.
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
    crate::nuxie_helpers::install(&mut vm, name).map_err(|e| e.message())?;
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

/// The vendored compiler with upstream's `defaultOptions()` (`loadstring` passes
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunkid_matches_upstream_rendering() {
        assert_eq!(chunkid(b"=basic.luau"), "basic.luau");
        assert_eq!(chunkid(b"@file.luau"), "file.luau");
        assert_eq!(chunkid(b"return 1"), "[string \"return 1\"]");
        assert_eq!(chunkid(b"local x\nreturn x"), "[string \"local x...\"]");
        let long = vec![b'a'; 300];
        let rendered = chunkid(&long);
        assert_eq!(rendered.len(), LUA_IDSIZE - 1);
        assert!(rendered.ends_with("...\"]"));
    }

    #[test]
    fn passes_ok_and_reports_failures() {
        assert!(matches!(
            run_script("t.luau", b"return 'OK'", false),
            Outcome::Pass
        ));
        let Outcome::Fail(message) = run_script("t.luau", b"assert(false)", false) else {
            panic!()
        };
        assert!(message.contains("assertion failed"), "{message}");
        let Outcome::Fail(message) = run_script("t.luau", b"return 1", false) else {
            panic!()
        };
        assert_eq!(message, "returned 1 instead of \"OK\"");
        let Outcome::Fail(message) = run_script("t.luau", b"coroutine.yield() return 'OK'", false)
        else {
            panic!()
        };
        assert_eq!(message, "thread yielded unexpectedly");
    }

    #[test]
    fn harness_globals_follow_upstream() {
        let script = br#"
            assert(_G == getfenv(1))
            assert(type(collectgarbage("count")) == "number")
            assert(collectgarbage("isrunning") == true)
            collectgarbage("stop"); assert(collectgarbage("isrunning") == false)
            collectgarbage("restart")
            assert(type(collectgarbage("step", 1)) == "boolean")
            assert(not pcall(collectgarbage, "nooption"))
            local f, e = loadstring("x =")
            assert(f == nil and string.find(e, '^%[string "x ="%]:1:'), e)
            assert(loadstring("return 7")() == 7)
            print("silent")
            return "OK"
        "#;
        let outcome = run_script("t.luau", script, false);
        if let Outcome::Fail(message) = outcome {
            panic!("{message}");
        }
    }

    #[test]
    fn byte_sources_compile_as_bytes() {
        // A Latin-1 byte in a literal keeps its length; one in a broken
        // token reaches the syntax-error text without assuming UTF-8.
        let script = b"assert(#\"\xe1\" == 1 and string.byte(\"\xe1\") == 225)\n\
            local f, e = loadstring(\"local x = \\\"\\255\xe1\")\n\
            assert(f == nil and type(e) == \"string\", e)\n\
            return \"OK\"";
        if let Outcome::Fail(message) = run_script("t.luau", script, false) {
            panic!("{message}");
        }
    }
}
