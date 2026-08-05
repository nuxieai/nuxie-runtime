#![allow(dead_code)]

use luaur_rt::{Error, FromLuaMulti, Function, Result, Value};
use nuxie_scripting::vm::ScriptVm;

fn compile_source(source: &str) -> Result<Vec<u8>> {
    use luaur_compiler::functions::luau_compile::luau_compile;

    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    if output.is_null() || output_size == 0 {
        return Err(Error::runtime("pinned Luau compiler returned no bytecode"));
    }
    // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
    let bytecode = unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    // SAFETY: output is the allocation returned by luau_compile above.
    unsafe { free(output.cast()) };
    if bytecode.first() == Some(&0) {
        return Err(Error::runtime(
            String::from_utf8_lossy(&bytecode[1..]).into_owned(),
        ));
    }
    Ok(bytecode)
}

/// Source helpers used only by baseline conformance tests.
///
/// Production callers intentionally cannot import this module. Editor source
/// compilation and execution live in nuxie-dev; these helpers keep existing
/// binding tests concise without restoring source APIs to `ScriptVm`.
pub trait ScriptVmSourceTestExt {
    fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R>;
    fn load(&self, name: &str, source: &str) -> Result<Function>;
    fn run_source_bytecode<R: FromLuaMulti>(&self, name: &str, source: &str) -> Result<R>;
    fn register_source_module(&self, name: &str, source: &str) -> Result<Value>;
}

impl ScriptVmSourceTestExt for ScriptVm {
    fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R> {
        self.lua().load(source).eval()
    }

    fn load(&self, name: &str, source: &str) -> Result<Function> {
        self.lua().load(source).set_name(name).into_function()
    }

    fn run_source_bytecode<R: FromLuaMulti>(&self, name: &str, source: &str) -> Result<R> {
        self.run_bytecode(name, &compile_source(source)?)
    }

    fn register_source_module(&self, name: &str, source: &str) -> Result<Value> {
        let bytecode = compile_source(source)?;
        let mut payload = Vec::with_capacity(bytecode.len() + 1);
        payload.push(0);
        payload.extend_from_slice(&bytecode);
        self.register_module(name, &payload)
    }
}
