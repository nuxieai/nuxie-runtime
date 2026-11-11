//! Differential harness for `nuxie-luau-vm` (plan
//! `docs/luau-safe-vm-plan.md`, Phase 2, lane 1).
//!
//! One Luau source file is compiled once per [`Profile`] by the vendored
//! compiler, and the same bytecode runs on the vendored VM (`luaur-rt`) and
//! on `nuxie-luau-vm`. The harness compares three things:
//!
//! - the values the chunk returned, each rendered as `tostring` would
//!   (objects as their type name only, since addresses differ);
//! - every line `print` wrote (each argument through the VM's own
//!   `tostring`, tab separated);
//! - the error that escaped, if any, as message text.
//!
//! Both states are prepared the same way: base libraries open, `print` and
//! `loadstring` installed, then `luaL_sandbox` + `luaL_sandboxthread` with
//! `_G` bound to the writable proxy. `loadstring(source [, name])` compiles
//! with the case's profile and loads the chunk as `=name` (default
//! `=loadstring`); a compile error returns `nil, message`.
//!
//! The corpus is `tests/luau-vm-diff/*.luau`: adding a case is adding a
//! file. Opcode coverage is measured over every bytecode the corpus loads,
//! including chunks built with `loadstring`.
//!
//! This is a tool, so the reference side uses `luaur-rt`'s `unsafe`
//! bytecode entry; the inputs are the vendored compiler's own output.

use std::sync::{Arc, Mutex};

use luaur_ast::records::parse_options::ParseOptions;
use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
use luaur_common::ScopedAllFlags;
use luaur_compiler::functions::compile::compile as compiler_compile;
use luaur_compiler::records::compile_options::CompileOptions;

/// The compiler configurations every case runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// Every Luau flag on (the product profile): bytecode version 11 with
    /// call feedback (`CALLFB`), optimization level 1.
    Product,
    /// As `Product` at optimization level 2 (inlining, unrolling).
    ProductO2,
    /// Every Luau flag off: the older bytecode version, optimization 1.
    Base,
    /// As `Product` with statement and expression coverage (`COVERAGE`).
    Coverage,
}

impl Profile {
    pub const ALL: [Profile; 4] = [
        Profile::Product,
        Profile::ProductO2,
        Profile::Base,
        Profile::Coverage,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Profile::Product => "product-O1",
            Profile::ProductO2 => "product-O2",
            Profile::Base => "base-O1",
            Profile::Coverage => "coverage",
        }
    }

    fn flags_on(self) -> bool {
        !matches!(self, Profile::Base)
    }

    fn optimization_level(self) -> core::ffi::c_int {
        match self {
            Profile::ProductO2 => 2,
            Profile::Product | Profile::Base | Profile::Coverage => 1,
        }
    }

    fn coverage_level(self) -> core::ffi::c_int {
        match self {
            Profile::Coverage => 2,
            Profile::Product | Profile::ProductO2 | Profile::Base => 0,
        }
    }
}

struct NoopEncoder;

impl BytecodeEncoder for NoopEncoder {
    fn encode(&mut self, _data: &mut [u32]) {}
}

/// Compiles `source` with the vendored compiler under `profile`: debug
/// level 1, type info level 1. `Err` carries the compiler's message.
pub fn compile(source: &str, profile: Profile) -> Result<Vec<u8>, String> {
    let _flags = ScopedAllFlags::enter(profile.flags_on());
    let options = CompileOptions {
        optimization_level: profile.optimization_level(),
        debug_level: 1,
        type_info_level: 1,
        coverage_level: profile.coverage_level(),
        ..CompileOptions::default()
    };
    let mut encoder = NoopEncoder;
    let blob = compiler_compile(
        &source.to_owned(),
        &options,
        &ParseOptions::default(),
        &mut encoder as *mut dyn BytecodeEncoder,
    );
    let bytes = blob.into_bytes();
    match bytes.split_first() {
        Some((0, message)) => Err(String::from_utf8_lossy(message).into_owned()),
        Some(_) => Ok(bytes),
        None => Err("the compiler returned nothing".to_owned()),
    }
}

/// What one VM did with one bytecode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub returns: Vec<String>,
    pub output: Vec<String>,
    pub error: Option<String>,
}

/// One case under one profile.
pub struct Run {
    pub profile: Profile,
    /// The bytecode version byte the compiler produced.
    pub version: u8,
    pub reference: Outcome,
    pub nuxie: Outcome,
    /// Every bytecode the case loaded (the chunk, then `loadstring` chunks).
    pub bytecodes: Vec<Vec<u8>>,
}

impl Run {
    pub fn agrees(&self) -> bool {
        self.reference == self.nuxie
    }
}

/// The vendored VM's thread stack: its transliterated interpreter recurses
/// through Rust frames (the conformance tool gives it the same 256 MiB).
pub const REFERENCE_STACK_BYTES: usize = 256 << 20;

/// `nuxie-luau-vm`'s thread stack: the usual 8 MiB main-thread size, so the
/// corpus also checks that the VM reaches its re-entry limits
/// (`Limits::max_c_calls`, `Limits::max_reentry`) without overflowing an
/// ordinary stack. `limit_tests` checks them on 1 MiB.
pub const NUXIE_STACK_BYTES: usize = 8 << 20;

fn on_thread<T: Send + 'static>(
    stack_bytes: usize,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    std::thread::Builder::new()
        .stack_size(stack_bytes)
        .spawn(work)
        .map_err(|e| format!("cannot start a VM thread: {e}"))?
        .join()
        .map_err(|_| "the VM thread panicked".to_owned())
}

/// Runs `source` under every profile, each VM on its own thread.
pub fn run_case(source: &str) -> Result<Vec<Run>, String> {
    Profile::ALL
        .iter()
        .map(|&profile| {
            let bytecode =
                compile(source, profile).map_err(|e| format!("{}: {e}", profile.name()))?;
            let shared = bytecode.clone();
            let reference = on_thread(REFERENCE_STACK_BYTES, move || {
                run_reference(&shared, profile)
            })?;
            let shared = bytecode.clone();
            let (nuxie, bytecodes) = on_thread(NUXIE_STACK_BYTES, move || {
                let mut bytecodes = vec![shared.clone()];
                let nuxie = run_nuxie(&shared, profile, &mut bytecodes);
                (nuxie, bytecodes)
            })?;
            Ok(Run {
                profile,
                version: bytecode.first().copied().unwrap_or(0),
                reference,
                nuxie,
                bytecodes,
            })
        })
        .collect()
}

// ---- the vendored VM ------------------------------------------------------

/// Runs `bytecode` on the vendored VM.
///
/// `print` and the rendering of returned values go through the raw stack
/// (`luaL_tolstring`), as upstream's `print` does: `luaur-rt`'s value
/// conversion turns `-0.0` into the integer 0, which would hide the sign.
pub fn run_reference(bytecode: &[u8], profile: Profile) -> Outcome {
    PRINTED.with(|lines| lines.borrow_mut().clear());
    let result = reference_inner(bytecode, profile);
    let output = PRINTED.with(|lines| std::mem::take(&mut *lines.borrow_mut()));
    match result {
        Ok(returns) => Outcome {
            returns,
            output,
            error: None,
        },
        Err(error) => Outcome {
            returns: Vec::new(),
            output,
            error: Some(reference_message(&error)),
        },
    }
}

thread_local! {
    static PRINTED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

type LuaState = luaur_vm::type_aliases::lua_state::lua_State;
type RawResult<T> = luaur_vm::records::lua_exception::LuaResult<T>;

/// `luaL_tolstring` of stack slot `index`, popped again.
///
/// # Safety
/// `l` is a live state and `index` an occupied slot.
unsafe fn raw_tostring(l: *mut LuaState, index: core::ffi::c_int) -> RawResult<Vec<u8>> {
    use luaur_vm::functions::{lua_l_tolstring::lua_l_tolstring, lua_settop::lua_settop};
    let mut len = 0usize;
    // SAFETY: per the contract; the returned pointer addresses `len` bytes
    // of a string kept alive by the stack slot until it is popped.
    unsafe {
        let text = lua_l_tolstring(l, index, &mut len)?;
        let bytes = std::slice::from_raw_parts(text.cast::<u8>(), len).to_vec();
        lua_settop(l, -2)?;
        Ok(bytes)
    }
}

/// Upstream's `print`, writing to [`PRINTED`].
///
/// # Safety
/// Called by the VM as a C function.
unsafe fn raw_print(l: *mut LuaState) -> RawResult<core::ffi::c_int> {
    use luaur_vm::functions::lua_gettop::lua_gettop;
    // SAFETY: called by the VM with a live state; slots 1..=n are the args.
    unsafe {
        let n = lua_gettop(l);
        let mut line = Vec::new();
        for index in 1..=n {
            if index > 1 {
                line.push(b'\t');
            }
            line.extend(raw_tostring(l, index)?);
        }
        PRINTED.with(|lines| lines.borrow_mut().push(exact(&line)));
    }
    Ok(0)
}

fn reference_inner(bytecode: &[u8], profile: Profile) -> luaur_rt::Result<Vec<String>> {
    use luaur_rt::{Lua, LuaString, MultiValue, Value};

    let lua = Lua::new();
    let globals = lua.globals();
    // SAFETY: `raw_print` follows the luaur C-function convention.
    globals.set("print", unsafe { lua.create_c_function(Some(raw_print)) }?)?;
    globals.set(
        "loadstring",
        lua.create_function(
            move |lua,
                  (source, name): (LuaString, Option<LuaString>)|
                  -> luaur_rt::Result<MultiValue> {
                let source = String::from_utf8_lossy(&source.as_bytes()).into_owned();
                let name = name
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| "loadstring".to_owned());
                Ok(match compile(&source, profile) {
                    // SAFETY: the bytes are the vendored compiler's output.
                    Ok(bytecode) => MultiValue::from_vec(vec![Value::Function(unsafe {
                        lua.load_bytecode(&name, &bytecode)
                    }?)]),
                    Err(message) => MultiValue::from_vec(vec![
                        Value::Nil,
                        Value::String(lua.create_string(message)),
                    ]),
                })
            },
        )?,
    )?;
    globals.set(
        "collectgarbage",
        lua.create_function(|lua, option: Option<LuaString>| -> luaur_rt::Result<f64> {
            match option.map(|o| o.to_string_lossy()).as_deref() {
                None | Some("collect") => {
                    lua.gc_collect()?;
                    Ok(0.0)
                }
                Some(other) => Err(luaur_rt::Error::RuntimeError(format!(
                    "collectgarbage: the diff harness has no option '{other}'"
                ))),
            }
        })?,
    )?;
    lua.sandbox(true)?;
    let env = lua.globals();
    env.set("_G", env.clone())?;
    // SAFETY: the bytes are the vendored compiler's output.
    let function = unsafe { lua.load_bytecode("case", bytecode) }?;
    let mut returns = Vec::new();
    // SAFETY: the closure calls the function in slot 1 and reads only the
    // slots the call produced, then clears the frame.
    unsafe {
        lua.exec_raw::<(), _>(function, |l| {
            use luaur_vm::functions::{
                lua_call::lua_call, lua_gettop::lua_gettop, lua_settop::lua_settop,
                lua_type::lua_type, lua_typename::lua_typename,
            };
            lua_call(l, 0, -1)?;
            for index in 1..=lua_gettop(l) {
                let kind = lua_type(l, index);
                let name = std::ffi::CStr::from_ptr(lua_typename(l, kind))
                    .to_string_lossy()
                    .into_owned();
                returns.push(match name.as_str() {
                    "nil" | "boolean" | "number" | "string" => {
                        format!("{name} {}", exact(&raw_tostring(l, index)?))
                    }
                    _ => name,
                });
            }
            lua_settop(l, 0)
        })?;
    }
    Ok(returns)
}

fn reference_message(error: &luaur_rt::Error) -> String {
    use luaur_rt::Error;
    match error {
        Error::RuntimeError(text) | Error::LuaLRuntimeError(text) | Error::MemoryError(text) => {
            text.clone()
        }
        Error::SyntaxError { message, .. } => message.clone(),
        other => other.to_string(),
    }
}

// ---- nuxie-luau-vm ------------------------------------------------------------

/// Runs `bytecode` on `nuxie-luau-vm`; every chunk `loadstring` compiles is
/// appended to `bytecodes`.
pub fn run_nuxie(bytecode: &[u8], profile: Profile, bytecodes: &mut Vec<Vec<u8>>) -> Outcome {
    use nuxie_luau_vm::{HostError, HostValue, Vm};

    let output = Arc::new(Mutex::new(Vec::new()));
    let loaded = Arc::new(Mutex::new(Vec::new()));
    let result = (|| -> Result<Vec<HostValue>, nuxie_luau_vm::Error> {
        // The reference runs the chunk from a C function (`exec_raw` calls
        // it with `lua_call`), which spends one C-call level a direct host
        // run does not; charge this VM the same level so re-entry limits
        // trip at the same depth.
        let defaults = nuxie_luau_vm::Limits::default();
        let mut vm = Vm::with_limits(nuxie_luau_vm::Limits {
            max_c_calls: defaults.max_c_calls.saturating_sub(1),
            ..defaults
        })?;
        let lines = Arc::clone(&output);
        vm.register("print", move |ctx| {
            let mut line = Vec::new();
            for n in 1..=ctx.arg_count() {
                if n > 1 {
                    line.push(b'\t');
                }
                line.extend(ctx.tostring_value(n)?);
            }
            if let Ok(mut lines) = lines.lock() {
                lines.push(exact(&line));
            }
            Ok(0)
        })?;
        let chunks = Arc::clone(&loaded);
        vm.register("loadstring", move |ctx| {
            let source = String::from_utf8_lossy(&ctx.check_string(1)?).into_owned();
            let name = ctx
                .opt_string(2)?
                .map(|n| String::from_utf8_lossy(&n).into_owned())
                .unwrap_or_else(|| "loadstring".to_owned());
            match compile(&source, profile) {
                Ok(bytecode) => {
                    let chunk_name = format!("={name}");
                    let loaded = ctx.load(chunk_name.as_bytes(), &bytecode)?;
                    if let Ok(mut chunks) = chunks.lock() {
                        chunks.push(bytecode);
                    }
                    match loaded {
                        Ok(()) => Ok(1),
                        Err(error) => Err(HostError::raw(error.to_string())),
                    }
                }
                Err(message) => {
                    ctx.push(HostValue::Nil)?;
                    ctx.push(HostValue::String(message.into_bytes()))?;
                    Ok(2)
                }
            }
        })?;
        vm.register("collectgarbage", |ctx| {
            match ctx.opt_string(1)?.as_deref() {
                None | Some(b"collect") => {
                    ctx.collect();
                    ctx.push(HostValue::Number(0.0))?;
                    Ok(1)
                }
                Some(other) => Err(HostError::raw(format!(
                    "collectgarbage: the diff harness has no option '{}'",
                    String::from_utf8_lossy(other)
                ))),
            }
        })?;
        vm.sandbox()?;
        let returned = vm.run(b"=case", bytecode);
        accounting_holds(&mut vm)?;
        returned
    })();
    bytecodes.extend(loaded.lock().map(|l| l.clone()).unwrap_or_default());
    let output = output.lock().map(|o| o.clone()).unwrap_or_default();
    match result {
        Ok(values) => Outcome {
            returns: values.iter().map(render_host).collect(),
            output,
            error: None,
        },
        Err(error) => Outcome {
            returns: Vec::new(),
            output,
            error: Some(error.message()),
        },
    }
}

/// The memory budget's count matches the arenas (budget.rs in the crate):
/// after the run, and again after a full collection. A difference means
/// something allocated around the budget, reported as the case's error so
/// it shows as a disagreement.
fn accounting_holds(vm: &mut nuxie_luau_vm::Vm) -> Result<(), nuxie_luau_vm::Error> {
    let after_run = vm.accounting_drift();
    vm.collect();
    let after_collect = vm.accounting_drift();
    if after_run == 0 && after_collect == 0 {
        Ok(())
    } else {
        Err(nuxie_luau_vm::Error::Host(format!(
            "memory accounting drift: {after_run} bytes after the run, {after_collect} after a collection"
        )))
    }
}

/// A returned value as `type text` for scalars (so a number and the string
/// of its digits differ) and the type name for objects, whose identities
/// differ between VMs by design.
fn render_host(value: &nuxie_luau_vm::HostValue) -> String {
    use nuxie_luau_vm::HostValue;
    match value {
        HostValue::Nil => "nil nil".to_owned(),
        HostValue::Boolean(b) => format!("boolean {b}"),
        HostValue::Number(n) => format!("number {}", nuxie_luau_vm::format_number(*n)),
        HostValue::String(bytes) => format!("string {}", exact(bytes)),
        other => other.type_name().to_owned(),
    }
}

/// Bytes as text without loss: ASCII escapes for everything that is not
/// printable ASCII, so distinct byte strings never compare equal.
fn exact(bytes: &[u8]) -> String {
    bytes.escape_ascii().to_string()
}

/// The opcodes of every prototype in `bytecode`, as `nuxie-luau-vm`'s
/// loader decodes them.
pub fn opcodes(bytecode: &[u8]) -> Result<Vec<u8>, String> {
    let mut vm = nuxie_luau_vm::Vm::new().map_err(|e| e.message())?;
    vm.opcodes(bytecode).map_err(|e| e.message())
}

/// Opcode names in `LuauOpcode` order.
pub const OPCODE_NAMES: [&str; 90] = [
    "NOP",
    "BREAK",
    "LOADNIL",
    "LOADB",
    "LOADN",
    "LOADK",
    "MOVE",
    "GETGLOBAL",
    "SETGLOBAL",
    "GETUPVAL",
    "SETUPVAL",
    "CLOSEUPVALS",
    "GETIMPORT",
    "GETTABLE",
    "SETTABLE",
    "GETTABLEKS",
    "SETTABLEKS",
    "GETTABLEN",
    "SETTABLEN",
    "NEWCLOSURE",
    "NAMECALL",
    "CALL",
    "RETURN",
    "JUMP",
    "JUMPBACK",
    "JUMPIF",
    "JUMPIFNOT",
    "JUMPIFEQ",
    "JUMPIFLE",
    "JUMPIFLT",
    "JUMPIFNOTEQ",
    "JUMPIFNOTLE",
    "JUMPIFNOTLT",
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "MOD",
    "POW",
    "ADDK",
    "SUBK",
    "MULK",
    "DIVK",
    "MODK",
    "POWK",
    "AND",
    "OR",
    "ANDK",
    "ORK",
    "CONCAT",
    "NOT",
    "MINUS",
    "LENGTH",
    "NEWTABLE",
    "DUPTABLE",
    "SETLIST",
    "FORNPREP",
    "FORNLOOP",
    "FORGLOOP",
    "FORGPREP_INEXT",
    "FASTCALL3",
    "FORGPREP_NEXT",
    "NATIVECALL",
    "GETVARARGS",
    "DUPCLOSURE",
    "PREPVARARGS",
    "LOADKX",
    "JUMPX",
    "FASTCALL",
    "COVERAGE",
    "CAPTURE",
    "SUBRK",
    "DIVRK",
    "FASTCALL1",
    "FASTCALL2",
    "FASTCALL2K",
    "FORGPREP",
    "JUMPXEQKNIL",
    "JUMPXEQKB",
    "JUMPXEQKN",
    "JUMPXEQKS",
    "IDIV",
    "IDIVK",
    "GETUDATAKS",
    "SETUDATAKS",
    "NAMECALLUDATA",
    "NEWCLASSMEMBER",
    "CALLFB",
    "CMPPROTO",
    "NEWCLASS",
];

/// Opcodes the loader rejects, so the corpus need not reach them: the
/// debugger break, native-code entry, the userdata-access and class
/// opcodes (bytecode versions this VM does not admit).
pub const REJECTED_OPCODES: [&str; 8] = [
    "BREAK",
    "NATIVECALL",
    "GETUDATAKS",
    "SETUDATAKS",
    "NAMECALLUDATA",
    "NEWCLASSMEMBER",
    "CMPPROTO",
    "NEWCLASS",
];

/// Opcodes the vendored compiler never emits (it names `NOP` only as a
/// placeholder), so no source can reach them.
pub const UNEMITTED_OPCODES: [&str; 1] = ["NOP"];

/// `nuxie-luau-vm`'s limits under real scripts (the crate has no compiler
/// of its own). The vendored VM has no per-state equivalents to compare.
#[cfg(test)]
mod budget_suite;

#[cfg(test)]
mod collector_tests;

#[cfg(test)]
mod limit_tests {
    use super::{Profile, compile};
    use nuxie_luau_vm::{Error, HostValue, Limits, Vm};

    fn run(vm: &mut Vm, source: &str) -> Result<Vec<HostValue>, Error> {
        let bytecode = compile(source, Profile::Product).expect("the source compiles");
        vm.run(b"=limits", &bytecode)
    }

    fn budget(bytes: usize) -> Limits {
        Limits {
            max_memory_bytes: Some(bytes),
            ..Limits::default()
        }
    }

    /// Every re-entry path to its limit, on a 1 MiB thread in this
    /// unoptimized test build (lane 4): metamethods, `pcall`, sort
    /// comparators, gsub callbacks and coroutine resumes run as frames, so
    /// they reach the C-call limit (or 10,000 nested `pcall`s) without Rust
    /// recursion; the host callback (`print` rendering a `__tostring` that
    /// prints), `string.format`'s `%s` and the `xpcall` handler recurse and
    /// stop at `Limits::max_reentry`.
    #[test]
    fn reentry_reaches_its_limits_on_a_one_mib_stack() {
        use nuxie_luau_vm::RunState;
        let source = r##"
            local out = {}
            local function stackover() return pcall(stackover) end
            out[#out + 1] = select("#", stackover())
            local function depth_of(f, ...)
                local n = 0
                local ok, e = pcall(f, function() n += 1 end, ...)
                return n, e
            end
            local deep = setmetatable({}, {__index = function(t, k) return t[k] end})
            out[#out + 1] = select(2, pcall(function() return deep.x end))
            out[#out + 1] = select(2, depth_of(function(tick)
                local function cmp(a, b) tick() table.sort({2, 1}, cmp) return a < b end
                table.sort({2, 1}, cmp)
            end))
            out[#out + 1] = select(2, depth_of(function(tick)
                local function rep() tick() return (string.gsub("x", "x", rep)) end
                string.gsub("x", "x", rep)
            end))
            local loud = setmetatable({}, {__tostring = function(self) print(self) return "" end})
            out[#out + 1] = select(2, pcall(print, loud))
            local fmt = setmetatable({}, {__tostring = function(self) return string.format("%*", self) end})
            out[#out + 1] = select(2, pcall(string.format, "%*", fmt))
            local a = function(a) coroutine.wrap(a)(a) end
            out[#out + 1] = select(2, pcall(a, a))
            local function handler(m) return select(2, xpcall(error, handler, m)) end
            out[#out + 1] = select(2, xpcall(error, handler, "e"))
            local function recurse(n) if n <= 1 then return 1 end return recurse(n - 1) + 1 end
            out[#out + 1] = recurse(19000)
            return table.unpack(out)
        "##;
        let bytecode = compile(source, Profile::Product).expect("the source compiles");
        let returned = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || {
                let mut vm = Vm::new().expect("vm");
                vm.register("print", |ctx| {
                    for n in 1..=ctx.arg_count() {
                        ctx.tostring_value(n)?;
                    }
                    Ok(0)
                })
                .expect("register");
                vm.start(b"=limits", &bytecode).map_err(|e| e.message())
            })
            .expect("a 1 MiB thread")
            .join()
            .expect("the VM thread did not overflow its stack");
        let Ok(RunState::Finished(mut values)) = returned else {
            panic!("{returned:?}");
        };
        // Each `coroutine.wrap` rethrow prepends its caller's position, as
        // C++ `auxwrapfinish` does: one per nested resume.
        let wrapped = values.remove(6);
        let HostValue::String(wrapped) = wrapped else {
            panic!("{wrapped:?}");
        };
        let prefix = b"=limits:24: ";
        let levels = wrapped
            .windows(prefix.len())
            .filter(|w| w == prefix)
            .count();
        assert!(
            wrapped.ends_with(b"C stack overflow"),
            "{}",
            wrapped.escape_ascii()
        );
        assert!(levels > 150, "{levels} nested resumes");
        let text = |s: &str| HostValue::String(s.as_bytes().to_vec());
        assert_eq!(
            values,
            vec![
                HostValue::Number(10000.0),
                text("=limits:10: C stack overflow"),
                text(":: C stack overflow"),
                text(":: C stack overflow"),
                text(":: C stack overflow"),
                text(":: C stack overflow"),
                text("error in error handling"),
                HostValue::Number(19000.0),
            ]
        );
    }

    #[test]
    fn a_rejected_table_growth_leaves_the_table_unchanged() {
        let limit = 64 << 10;
        let mut vm = Vm::with_limits(budget(limit)).expect("vm");
        let returned = run(
            &mut vm,
            "local t, failures = {}, 0
             for i = 1, 100000 do
                 if not pcall(rawset, t, i, i) then
                     failures += 1
                     if failures == 5 then break end
                 end
             end
             return #t, failures",
        )
        .expect("runs");
        assert_eq!(returned.get(1), Some(&HostValue::Number(5.0)));
        let Some(HostValue::Number(len)) = returned.first() else {
            panic!("{returned:?}");
        };
        assert!(*len < 4096.0, "the table grew to {len} entries");
        assert!(vm.memory_bytes() <= limit, "{} bytes", vm.memory_bytes());
    }

    #[test]
    fn gsub_stops_at_the_string_limit_before_building_its_result() {
        let mut vm = Vm::with_limits(Limits {
            max_string_bytes: 10_000,
            max_memory_bytes: Some(1 << 20),
            ..Limits::default()
        })
        .expect("vm");
        let returned = run(
            &mut vm,
            r#"local s = string.rep("a", 5000)
               return pcall(string.gsub, s, ".", string.rep("b", 5000))"#,
        )
        .expect("runs");
        assert_eq!(
            returned,
            vec![
                HostValue::Boolean(false),
                HostValue::String(b"not enough memory".to_vec())
            ]
        );
        let returned = run(
            &mut vm,
            r#"return pcall(string.gsub, string.rep("ab", 2000), "(.)", "%0%0%0%0%0%0%0%0")"#,
        )
        .expect("runs");
        assert_eq!(returned.first(), Some(&HostValue::Boolean(false)));
    }

    #[test]
    fn a_refused_allocation_is_not_charged() {
        let mut vm = Vm::with_limits(budget(32 << 10)).expect("vm");
        let before = vm.memory_bytes();
        assert!(
            vm.set_global("big", HostValue::String(vec![b'x'; 40 << 10]))
                .is_err()
        );
        // Only the global's name may have been charged.
        assert!(
            vm.memory_bytes() < before + 1024,
            "{} after {before}",
            vm.memory_bytes()
        );
        vm.set_global("small", HostValue::String(b"ab".to_vec()))
            .expect("a small string still fits");
    }

    #[test]
    fn the_stack_limit_holds_below_the_initial_stack() {
        let mut vm = Vm::with_limits(Limits {
            max_stack_slots: 4,
            ..Limits::default()
        })
        .expect("vm");
        let error = run(
            &mut vm,
            "local a, b, c, d, e, f, g, h = 1, 2, 3, 4, 5, 6, 7, 8
             return a + h",
        )
        .expect_err("needs more than 4 slots");
        assert!(
            error.message().contains("stack overflow"),
            "{}",
            error.message()
        );
    }

    #[test]
    fn concat_and_format_stop_at_the_string_limit_before_building() {
        let mut vm = Vm::with_limits(Limits {
            max_string_bytes: 16 << 10,
            max_memory_bytes: Some(1 << 20),
            ..Limits::default()
        })
        .expect("vm");
        let returned = run(
            &mut vm,
            r#"local big, refs = string.rep("x", 8192), {}
               for i = 1, 1000 do refs[i] = big end
               local ok1, e1 = pcall(table.concat, refs)
               local ok2, e2 = pcall(string.format, "%s%s%s", big, big, big)
               local ok3, e3 = pcall(string.format, "%q", string.rep("\0", 5000))
               return ok1, e1, ok2, e2, ok3, e3"#,
        )
        .expect("runs");
        let refused = [
            HostValue::Boolean(false),
            HostValue::String(b"not enough memory".to_vec()),
        ];
        assert_eq!(
            returned,
            [refused.clone(), refused.clone(), refused].concat()
        );
    }

    #[test]
    fn a_small_budget_is_collected_before_it_runs_out() {
        let mut vm = Vm::with_limits(budget(64 << 10)).expect("vm");
        let returned = run(
            &mut vm,
            "for i = 1, 10000 do local t = {i, i + 1} end
             return 'done'",
        )
        .expect("garbage is collected under a 64 KiB budget");
        assert_eq!(returned, vec![HostValue::String(b"done".to_vec())]);
    }

    #[test]
    fn a_vector_key_with_a_nan_component_is_refused_like_the_reference() {
        // Only product-O2 folds `vector.create` into a constant; this VM has
        // no vector library yet (lane 5).
        let source = "local v = vector.create(1, 2, 3)
                      local ok, e = pcall(rawset, {}, v * (0 / 0), 1)
                      return ok, e";
        let bytecode = compile(source, Profile::ProductO2).expect("compiles");
        let mut loaded = Vec::new();
        let reference = super::run_reference(&bytecode, Profile::ProductO2);
        let nuxie = super::run_nuxie(&bytecode, Profile::ProductO2, &mut loaded);
        assert_eq!(nuxie.returns, reference.returns);
        assert_eq!(
            nuxie.returns.first().map(String::as_str),
            Some("boolean false")
        );
    }

    #[test]
    fn concatenation_is_checked_against_the_budget_before_it_builds() {
        let mut vm = Vm::with_limits(budget(64 << 10)).expect("vm");
        let returned = run(
            &mut vm,
            r#"local s = string.rep("x", 20000)
               return pcall(function() return s .. s .. s .. s .. s .. s .. s .. s end)"#,
        )
        .expect("runs");
        assert_eq!(
            returned,
            vec![
                HostValue::Boolean(false),
                HostValue::String(b"not enough memory".to_vec())
            ]
        );
    }

    #[test]
    fn repeated_runs_are_collected_at_the_host_boundary() {
        let mut vm = Vm::with_limits(budget(64 << 10)).expect("vm");
        let bytecode = compile("return 1", Profile::Product).expect("compiles");
        for _ in 0..5000 {
            vm.run(b"=again", &bytecode)
                .expect("each run fits the budget");
        }
    }

    #[test]
    fn quoted_formatting_is_checked_at_its_real_length() {
        let mut vm = Vm::with_limits(Limits {
            max_string_bytes: 100,
            ..Limits::default()
        })
        .expect("vm");
        let returned = run(
            &mut vm,
            r#"return #string.format("%q", string.rep("a", 30))"#,
        )
        .expect("a 32-byte result fits a 100-byte limit");
        assert_eq!(returned, vec![HostValue::Number(32.0)]);
    }

    #[test]
    fn a_handled_tostring_error_leaves_the_vm_usable() {
        let mut vm = Vm::new().expect("vm");
        vm.register("probe", |ctx| {
            let rendered = ctx.tostring_value(1).is_ok();
            ctx.push(HostValue::Boolean(rendered))?;
            Ok(1)
        })
        .expect("register");
        let returned = run(
            &mut vm,
            r##"local bad = setmetatable({}, {__tostring = function() error("boom") end})
               local a = probe(bad)
               return a, select("#", probe(bad)), probe(1)"##,
        )
        .expect("runs");
        assert_eq!(
            returned,
            vec![
                HostValue::Boolean(false),
                HostValue::Number(1.0),
                HostValue::Boolean(true)
            ]
        );
        assert_eq!(
            run(&mut vm, "return 2").expect("the next run starts clean"),
            vec![HostValue::Number(2.0)]
        );
    }

    #[test]
    fn rejected_loads_are_collected_at_the_host_boundary() {
        let mut vm = Vm::with_limits(budget(64 << 10)).expect("vm");
        let bytecode = compile(
            r#"local t = {"a string", "another", 1.5} return t"#,
            Profile::Product,
        )
        .expect("compiles");
        let truncated = bytecode.get(..bytecode.len() - 3).expect("long enough");
        for _ in 0..5000 {
            assert!(vm.run(b"=cut", truncated).is_err());
        }
        assert_eq!(
            run(&mut vm, "return 1").expect("the budget is still free"),
            vec![HostValue::Number(1.0)]
        );
    }

    #[test]
    fn replacing_a_global_from_the_host_is_collected() {
        let mut vm = Vm::with_limits(budget(64 << 10)).expect("vm");
        for i in 0..20_000 {
            vm.set_global(
                "value",
                HostValue::String(format!("value {i}").into_bytes()),
            )
            .expect("the replaced strings are collected");
        }
    }

    #[test]
    fn string_builders_check_the_budget_and_empty_repeats_are_immediate() {
        let mut vm = Vm::with_limits(budget(256 << 10)).expect("vm");
        let returned = run(
            &mut vm,
            r#"local s = string.rep("a", 150000)
               local ok1 = pcall(string.upper, s)
               local ok2 = pcall(string.reverse, s)
               return ok1, ok2, #string.rep("", 2147483647)"#,
        )
        .expect("runs");
        assert_eq!(
            returned,
            vec![
                HostValue::Boolean(false),
                HostValue::Boolean(false),
                HostValue::Number(0.0)
            ]
        );
    }

    #[test]
    fn table_create_and_clone_check_the_budget_before_allocating() {
        let limit = 256 << 10;
        let mut vm = Vm::with_limits(budget(limit)).expect("vm");
        let returned = run(
            &mut vm,
            r#"local ok1, e1 = pcall(table.create, 100000, true)
               local t = table.create(9000, 1)
               local ok2, e2 = pcall(table.clone, t)
               local ok3, e3 = pcall(table.create, 2^26 + 1)
               return ok1, e1, ok2, e2, ok3, e3"#,
        )
        .expect("runs");
        let refused = [
            HostValue::Boolean(false),
            HostValue::String(b"not enough memory".to_vec()),
        ];
        let overflow = [
            HostValue::Boolean(false),
            HostValue::String(b":: table overflow".to_vec()),
        ];
        assert_eq!(returned, [refused.clone(), refused, overflow].concat());
        assert!(vm.memory_bytes() <= limit, "{} bytes", vm.memory_bytes());
    }

    #[test]
    fn split_and_sparse_move_stay_inside_the_budget() {
        let limit = 256 << 10;
        let mut vm = Vm::with_limits(budget(limit)).expect("vm");
        let returned = run(
            &mut vm,
            r#"return pcall(string.split, string.rep("ab,", 30000))"#,
        )
        .expect("runs");
        assert_eq!(
            returned,
            vec![
                HostValue::Boolean(false),
                HostValue::String(b"not enough memory".to_vec()),
            ]
        );
        assert!(vm.memory_bytes() <= limit, "{} bytes", vm.memory_bytes());
        // A fresh VM: the failed split's garbage is not collected until the
        // next safepoint. From 5, so the destination's array part is not
        // resized (from 1 it would be, to 2^30: "table overflow").
        let mut vm = Vm::with_limits(budget(limit)).expect("vm");
        let returned = run(
            &mut vm,
            "local src = {}
             for i = 1, 200 do src[i * 1000] = i end
             local moved = table.move(src, 1, 2^30, 5, {})
             return moved[1004], moved[200004]",
        )
        .expect("runs");
        assert_eq!(returned, vec![HostValue::Number(1.0), HostValue::Number(200.0)]);
        assert!(vm.memory_bytes() <= limit, "{} bytes", vm.memory_bytes());
    }

    #[test]
    fn builtin_upvalues_stay_counted_after_a_collection() {
        // 2000 retained gmatch iterators, three upvalues each (96 KB of
        // upvalue charges). The script leaves almost no garbage, so the
        // collection's live estimate must stay near what was charged; before
        // the fix it dropped every builtin upvalue.
        let mut vm = Vm::new().expect("vm");
        run(
            &mut vm,
            r#"keep = table.create(2000)
               for i = 1, 2000 do keep[i] = string.gmatch("abc", ".") end"#,
        )
        .expect("runs");
        let before = vm.memory_bytes();
        vm.collect();
        let after = vm.memory_bytes();
        assert!(after + 8192 >= before, "{before} bytes before, {after} after");
    }

    #[test]
    fn a_sparse_move_is_charged_only_for_the_entries_it_moves() {
        let mut vm = Vm::with_limits(budget(128 << 10)).expect("vm");
        let returned = run(
            &mut vm,
            "local t = table.create(3000, true)
             return #table.move(t, 1000000, 2000000, -2000000, {})",
        )
        .expect("an empty source range needs no buffer");
        assert_eq!(returned, vec![HostValue::Number(0.0)]);
    }

    #[test]
    fn a_sparse_move_charges_its_temporary_entries() {
        // 8193 entries in range need a temporary copy; under this budget the
        // copy does not fit next to the source, so the move fails cleanly
        // and memory stays inside the limit.
        let limit = 900_000;
        let mut vm = Vm::with_limits(budget(limit)).expect("vm");
        let returned = run(
            &mut vm,
            "local t = {}
             for i = 1, 8193 do t[-i * 100] = i end
             local ok, e = pcall(table.move, t, -1000000, 0, -1000000, t)
             return ok, e",
        )
        .expect("runs");
        assert!(vm.memory_bytes() <= limit, "{} bytes", vm.memory_bytes());
        if returned.first() == Some(&HostValue::Boolean(false)) {
            assert_eq!(
                returned.get(1),
                Some(&HostValue::String(b"not enough memory".to_vec()))
            );
        }
    }
}
