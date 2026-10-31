//! The VM-facing half of the runner: upstream's `runConformance` harness
//! (luau-lang/luau `tests/Conformance.test.cpp`) rebuilt on the `luaur-rt`
//! handle API. This is the only module that names `luaur_rt`; pointing the
//! tool at another VM behind the same handle API means changing this file
//! (or the crate it imports) and nothing else.
//!
//! What upstream does, and what this does:
//!
//! - `luaL_openlibs`, then globals `collectgarbage`, `loadstring` and a silent
//!   `print` (upstream omits `print` with `--verbose`, falling back to the
//!   library one; the Rive profile ships no `print`, so `--verbose` installs
//!   one that writes `tostring` of each argument to stdout).
//! - `luaL_sandbox` + `luaL_sandboxthread`, then `_G` bound to the writable
//!   proxy environment. Here: `Lua::sandbox(true)`, which performs both.
//! - Per-script setup lambdas are not reproduced, with one exception: the
//!   scripts upstream gives `setupNativeHelpers` get `is_native` (always
//!   false) and `is_native_if_supported` (always true), which is exactly what
//!   upstream's helpers return when code generation is unavailable, as it is
//!   for every VM this tool drives. Every other setup (vector and userdata
//!   metatables, C yield helpers, allocator hooks, debugger callbacks) is
//!   absent, and the scope file says which scripts need it.
//! - The script is compiled with optimization level 1, debug level 1, type
//!   info level 1 and float vector precision (upstream `defaultOptions()` in a
//!   default `LUA_VECTOR_DOUBLE=0` build), chunk name `=<script>`, and run
//!   until it returns; it passes iff the final returned value is "OK".
//!
//! Known harness limitations (each is a property of the `luaur-rt` API, not
//! of the VM under test; the scope file cites them where they decide a
//! script's outcome):
//!
//! - **Main-thread resume is not reachable.** Upstream calls `lua_resume` on
//!   the main state. `luaur-rt` offers `pcall` on the main state or a
//!   coroutine, so the script runs on a fresh coroutine created from the
//!   sandboxed main state (same globals). Scripts that inspect the running
//!   thread (`coroutine.running`, `coroutine.isyieldable`) observe a
//!   non-main thread. Yield and break handlers that upstream passes for some
//!   scripts are not provided; a yield out of the script fails it as
//!   "thread yielded unexpectedly", like upstream without a handler.
//! - **Chunk names always get a `=` prefix.** `luaur-rt` passes
//!   `"=" + name` to `luau_load`. Upstream's `loadstring(s, name)` passes
//!   `name` (default: the source text itself) verbatim, which renders as
//!   `[string "..."]`. To keep `short_src` faithful, `loadstring` passes the
//!   upstream *rendering* of the chunk name (`luaO_chunkid`) as the name, so
//!   a VM that renders `=x` as `x` prints what upstream prints. Raw
//!   `debug.info(f, "s")` / `source` still differs (`=[string "..."]`).
//!   Syntax errors come back from `luaur-rt` without a chunk prefix; the
//!   rendered chunk id is prepended here, as upstream's `luau_load` does.
//! - **Sources are `String`, not bytes.** `luaur-rt`'s compiler entry
//!   (`Lua::load`) takes `&str`. Valid UTF-8 is passed unchanged. The
//!   three Latin-1 corpus scripts (`literals`, `pm`, `sort`) and any
//!   non-UTF-8 `loadstring` argument are transcoded byte-for-char (each byte
//!   becomes U+0000..U+00FF), so string literals with bytes >= 0x80 change
//!   length and content. Upstream compiles the bytes as they are.
//! - **`loadstring` does not clear `safeenv`.** Upstream calls
//!   `lua_setsafeenv(L, LUA_ENVIRONINDEX, false)`, which clears the flag on
//!   the C closure's environment (the pre-sandbox global table). Script
//!   closures run in the sandbox proxy, whose flag is untouched, so this is
//!   not observable to fastcalls; `luaur-rt` has no handle for it anyway.
//! - **`collectgarbage("setgoal"|"setstepmul"|"setstepsize")`** apply the
//!   value but return 0, not the previous setting (`luaur-rt` does not
//!   report it). No corpus script uses them.
//! - **No `luaC_validate`** after the run and no `limitedstack` global (only
//!   set upstream in ASAN/debug builds).

use luaur_rt::{
    Compiler, Error, Function, Lua, LuaString, MultiValue, Result, ThreadStatus, Value,
};

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

/// Run one conformance script to completion on a fresh VM.
pub fn run_script(name: &str, source: &[u8], verbose: bool) -> Outcome {
    match run(name, source, verbose) {
        Ok(outcome) => outcome,
        Err(error) => Outcome::Fail(first_line(&message(&error))),
    }
}

fn run(name: &str, source: &[u8], verbose: bool) -> Result<Outcome> {
    let lua = Lua::new();
    install_globals(&lua, verbose)?;
    if NATIVE_HELPER_SCRIPTS.contains(&name) {
        // Upstream setupNativeHelpers without code generation.
        let globals = lua.globals();
        globals.set("is_native", lua.create_function(|_, ()| Ok(false))?)?;
        globals.set(
            "is_native_if_supported",
            lua.create_function(|_, ()| Ok(true))?,
        )?;
    }
    lua.sandbox(true)?;
    let env = lua.globals();
    env.set("_G", env.clone())?;

    let compiler = Compiler::new()
        .set_optimization_level(1)
        .set_debug_level(1)
        .set_type_info_level(1)
        .set_vector_precision(0);
    let chunkname = format!("={name}");
    let function = match lua
        .load(source_text(source))
        .set_name(name)
        .set_compiler(compiler)
        .into_function()
    {
        Ok(function) => function,
        Err(error) => return Ok(Outcome::Fail(first_line(&load_error(&chunkname, &error)))),
    };
    let thread = lua.create_thread(function)?;
    let returned: MultiValue = thread.resume(())?;
    if thread.status() == ThreadStatus::Resumable {
        return Ok(Outcome::Fail("thread yielded unexpectedly".into()));
    }
    Ok(match returned.into_vec().pop() {
        Some(Value::String(text)) if text.as_bytes() == b"OK" => Outcome::Pass,
        Some(other) => Outcome::Fail(format!("returned {} instead of \"OK\"", describe(&other))),
        None => Outcome::Fail("returned nothing instead of \"OK\"".into()),
    })
}

fn install_globals(lua: &Lua, verbose: bool) -> Result<()> {
    let globals = lua.globals();
    globals.set("collectgarbage", lua.create_function(collectgarbage)?)?;
    globals.set("loadstring", lua.create_function(loadstring)?)?;
    globals.set(
        "print",
        lua.create_function(move |lua, args: MultiValue| {
            if verbose {
                let tostring: Function = lua.globals().get("tostring")?;
                let mut line = Vec::new();
                for (index, value) in args.into_vec().into_iter().enumerate() {
                    if index > 0 {
                        line.push(b'\t');
                    }
                    line.extend(tostring.call::<LuaString>(value)?.as_bytes());
                }
                line.push(b'\n');
                use std::io::Write;
                let _ = std::io::stdout().lock().write_all(&line);
            }
            Ok(())
        })?,
    )?;
    Ok(())
}

/// Upstream `lua_collectgarbage`: `luaL_checkoption(L, 1, "collect", ...)`,
/// `luaL_optinteger(L, 2, 0)`, then `lua_gc`; `step` and `isrunning` return
/// booleans, everything else the number `lua_gc` returned.
fn collectgarbage(lua: &Lua, (option, extra): (Value, Value)) -> Result<Value> {
    let option = match &option {
        Value::Nil => "collect".to_owned(),
        Value::String(text) => text.to_string_lossy(),
        Value::Integer(_) | Value::Number(_) => lua_tostring(lua, &option)?,
        other => {
            return Err(arg_error(
                "collectgarbage",
                1,
                &format!("string expected, got {}", other.type_name()),
            ));
        }
    };
    let extra = match &extra {
        Value::Nil => 0,
        Value::Integer(value) => *value as i32,
        Value::Number(value) => *value as i32,
        Value::String(_) => match lua.coerce_number(extra.clone())? {
            Some(value) => value as i32,
            None => {
                return Err(arg_error(
                    "collectgarbage",
                    2,
                    "number expected, got string",
                ));
            }
        },
        other => {
            return Err(arg_error(
                "collectgarbage",
                2,
                &format!("number expected, got {}", other.type_name()),
            ));
        }
    };
    Ok(match option.as_str() {
        "stop" => {
            lua.gc_stop();
            Value::Number(0.0)
        }
        "restart" => {
            lua.gc_restart();
            Value::Number(0.0)
        }
        "collect" => {
            lua.gc_collect()?;
            Value::Number(0.0)
        }
        "count" => Value::Number(lua.gc_count() as f64),
        "isrunning" => Value::Boolean(lua.gc_is_running()),
        "step" => Value::Boolean(lua.gc_step_kbytes(extra)?),
        "setgoal" => {
            lua.gc_inc(extra, 0, 0);
            Value::Number(0.0)
        }
        "setstepmul" => {
            lua.gc_inc(0, extra, 0);
            Value::Number(0.0)
        }
        "setstepsize" => {
            lua.gc_inc(0, 0, extra);
            Value::Number(0.0)
        }
        other => {
            return Err(arg_error(
                "collectgarbage",
                1,
                &format!("invalid option '{other}'"),
            ));
        }
    })
}

/// Upstream `lua_loadstring`: compile `s` with default options under
/// `chunkname` (default `s`), return the function or `nil, message`.
fn loadstring(lua: &Lua, (source, chunkname): (Value, Value)) -> Result<MultiValue> {
    let source = match &source {
        Value::String(text) => text.as_bytes(),
        Value::Integer(_) | Value::Number(_) => lua_tostring(lua, &source)?.into_bytes(),
        other => {
            return Err(arg_error(
                "loadstring",
                1,
                &format!("string expected, got {}", other.type_name()),
            ));
        }
    };
    let chunkname = match &chunkname {
        Value::Nil => source.clone(),
        Value::String(text) => text.as_bytes(),
        Value::Integer(_) | Value::Number(_) => lua_tostring(lua, &chunkname)?.into_bytes(),
        other => {
            return Err(arg_error(
                "loadstring",
                2,
                &format!("string expected, got {}", other.type_name()),
            ));
        }
    };
    let rendered = chunkid(&chunkname);
    // luaur-rt prefixes "=" to the name; pass upstream's rendering so the
    // displayed short_src matches (see the module comment).
    let chunk = lua.load(source_text(&source)).set_name(rendered.clone());
    Ok(match chunk.into_function() {
        Ok(function) => MultiValue::from_vec(vec![Value::Function(function)]),
        Err(error) => {
            let text = load_error(&format!("={rendered}"), &error);
            MultiValue::from_vec(vec![Value::Nil, Value::String(lua.create_string(text))])
        }
    })
}

/// The message a failed `luau_load` leaves upstream: the rendered chunk id
/// followed by the compiler's `:line: message`. Other errors pass through.
fn load_error(chunkname: &str, error: &Error) -> String {
    match error {
        Error::SyntaxError { message, .. } => format!("{}{message}", chunkid(chunkname.as_bytes())),
        other => message(other),
    }
}

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

/// See the module comment: luaur-rt compiles `&str`, so non-UTF-8 bytes are
/// transcoded byte-for-char.
fn source_text(source: &[u8]) -> String {
    match std::str::from_utf8(source) {
        Ok(text) => text.to_owned(),
        Err(_) => source.iter().map(|&byte| char::from(byte)).collect(),
    }
}

fn lua_tostring(lua: &Lua, value: &Value) -> Result<String> {
    let tostring: Function = lua.globals().get("tostring")?;
    Ok(tostring.call::<LuaString>(value.clone())?.to_string_lossy())
}

fn arg_error(function: &str, index: usize, detail: &str) -> Error {
    Error::runtime(format!(
        "invalid argument #{index} to '{function}' ({detail})"
    ))
}

/// The VM's own message, without luaur-rt's "runtime error: " framing.
fn message(error: &Error) -> String {
    match error {
        Error::RuntimeError(text) | Error::LuaLRuntimeError(text) | Error::MemoryError(text) => {
            text.clone()
        }
        Error::SyntaxError { message, .. } => message.clone(),
        other => other.to_string(),
    }
}

fn describe(value: &Value) -> String {
    match value {
        Value::String(text) => format!("\"{}\"", text.to_string_lossy()),
        Value::Boolean(flag) => flag.to_string(),
        Value::Integer(number) => number.to_string(),
        Value::Number(number) => number.to_string(),
        other => other.type_name().to_owned(),
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().to_owned()
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
}
