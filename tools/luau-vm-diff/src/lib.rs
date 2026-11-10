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

/// Runs `source` under every profile.
pub fn run_case(source: &str) -> Result<Vec<Run>, String> {
    Profile::ALL
        .iter()
        .map(|&profile| {
            let bytecode =
                compile(source, profile).map_err(|e| format!("{}: {e}", profile.name()))?;
            let reference = run_reference(&bytecode, profile);
            let mut bytecodes = vec![bytecode.clone()];
            let nuxie = run_nuxie(&bytecode, profile, &mut bytecodes);
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
        PRINTED.with(|lines| lines.borrow_mut().push(String::from_utf8_lossy(&line).into_owned()));
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
            move |lua, (source, name): (LuaString, Option<LuaString>)| -> luaur_rt::Result<MultiValue> {
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
                        String::from_utf8_lossy(&raw_tostring(l, index)?).into_owned()
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
        let mut vm = Vm::new()?;
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
                lines.push(String::from_utf8_lossy(&line).into_owned());
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
        vm.sandbox()?;
        vm.run(b"=case", bytecode)
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

fn render_host(value: &nuxie_luau_vm::HostValue) -> String {
    use nuxie_luau_vm::HostValue;
    match value {
        HostValue::Nil => "nil".to_owned(),
        HostValue::Boolean(b) => b.to_string(),
        HostValue::Number(n) => nuxie_luau_vm::format_number(*n),
        HostValue::String(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        other => other.type_name().to_owned(),
    }
}

/// The opcodes of every prototype in `bytecode`, as `nuxie-luau-vm`'s
/// loader decodes them.
pub fn opcodes(bytecode: &[u8]) -> Result<Vec<u8>, String> {
    let mut vm = nuxie_luau_vm::Vm::new().map_err(|e| e.message())?;
    vm.opcodes(bytecode).map_err(|e| e.message())
}

/// Opcode names in `LuauOpcode` order.
pub const OPCODE_NAMES: [&str; 90] = [
    "NOP", "BREAK", "LOADNIL", "LOADB", "LOADN", "LOADK", "MOVE", "GETGLOBAL", "SETGLOBAL",
    "GETUPVAL", "SETUPVAL", "CLOSEUPVALS", "GETIMPORT", "GETTABLE", "SETTABLE", "GETTABLEKS",
    "SETTABLEKS", "GETTABLEN", "SETTABLEN", "NEWCLOSURE", "NAMECALL", "CALL", "RETURN", "JUMP",
    "JUMPBACK", "JUMPIF", "JUMPIFNOT", "JUMPIFEQ", "JUMPIFLE", "JUMPIFLT", "JUMPIFNOTEQ",
    "JUMPIFNOTLE", "JUMPIFNOTLT", "ADD", "SUB", "MUL", "DIV", "MOD", "POW", "ADDK", "SUBK",
    "MULK", "DIVK", "MODK", "POWK", "AND", "OR", "ANDK", "ORK", "CONCAT", "NOT", "MINUS",
    "LENGTH", "NEWTABLE", "DUPTABLE", "SETLIST", "FORNPREP", "FORNLOOP", "FORGLOOP",
    "FORGPREP_INEXT", "FASTCALL3", "FORGPREP_NEXT", "NATIVECALL", "GETVARARGS", "DUPCLOSURE",
    "PREPVARARGS", "LOADKX", "JUMPX", "FASTCALL", "COVERAGE", "CAPTURE", "SUBRK", "DIVRK",
    "FASTCALL1", "FASTCALL2", "FASTCALL2K", "FORGPREP", "JUMPXEQKNIL", "JUMPXEQKB", "JUMPXEQKN",
    "JUMPXEQKS", "IDIV", "IDIVK", "GETUDATAKS", "SETUDATAKS", "NAMECALLUDATA", "NEWCLASSMEMBER",
    "CALLFB", "CMPPROTO", "NEWCLASS",
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
