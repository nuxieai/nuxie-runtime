use std::env;
use std::fs;
use std::path::PathBuf;

use luaur_compiler::functions::luau_compile::luau_compile;

const BIND: &str = r#"
local func, prepend = ...
return function(...)
    return func(prepend(...))
end
"#;

const TABLE_LEN: &str = "local t = ...; return #t";

const USERDATA_DISPATCH: &str = r#"
local getters, setters, methods = ...
local function index(ud, key)
    local getter = getters[key]
    if getter ~= nil then
        return getter(ud)
    end
    return methods[key]
end
local function newindex(ud, key, value)
    local setter = setters[key]
    if setter ~= nil then
        return setter(ud, value)
    end
    error(string.format("attempt to set unknown field '%s' on userdata", tostring(key)), 0)
end
return index, newindex
"#;

const ASYNC_POLLER: &str = r#"
local poll, yield = poll, yield
local future = get_future(...)
local nres, res, res2 = poll(future)
while true do
    if nres ~= nil then
        if nres == 0 then
            return
        elseif nres == 1 then
            return res
        elseif nres == 2 then
            return res, res2
        elseif nres < 0 then
            yield()
        else
            return unpack(res, nres)
        end
    end

    if res2 == nil then
        nres, res, res2 = poll(future, yield(res))
    elseif res2 == 0 then
        nres, res, res2 = poll(future, yield())
    elseif res2 == 1 then
        nres, res, res2 = poll(future, yield(res))
    else
        nres, res, res2 = poll(future, yield(unpack(res, res2)))
    end
end
"#;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    for (name, source) in [
        ("bind.luau-bytecode", BIND),
        ("table-len.luau-bytecode", TABLE_LEN),
        ("userdata-dispatch.luau-bytecode", USERDATA_DISPATCH),
        ("async-poller.luau-bytecode", ASYNC_POLLER),
    ] {
        fs::write(output.join(name), compile(source)).expect("write luaur-rt builtin bytecode");
    }
}

fn compile(source: &str) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null() && output_size > 0, "Luau compile failed");
    // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
    let bytecode = unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut core::ffi::c_void);
    }
    // SAFETY: output is the allocation returned by luau_compile above.
    unsafe { free(output.cast()) };
    assert_ne!(bytecode.first(), Some(&0), "Luau builtin source is invalid");
    bytecode
}
