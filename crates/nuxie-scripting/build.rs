use std::env;
use std::fs;
use std::path::PathBuf;

use luaur_compiler::functions::luau_compile::luau_compile;

fn main() {
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    for (source_path, constant, delimiter, output_name) in [
        (
            "src/vm/lua_promise.rs",
            "PROMISE_LIBRARY",
            "##",
            "promise-library.luau-bytecode",
        ),
        (
            "src/vm/lua_data_value.rs",
            "DATA_VALUE_METATABLE_PATCHER_SOURCE",
            "#",
            "data-value-metatable.luau-bytecode",
        ),
        (
            "src/vm/view_model.rs",
            "PROPERTY_METATABLE_PATCHER_SOURCE",
            "#",
            "property-metatable.luau-bytecode",
        ),
    ] {
        println!("cargo:rerun-if-changed={source_path}");
        let rust_source = fs::read_to_string(source_path).expect("read embedded Luau source");
        let source = extract_raw_string_constant(&rust_source, constant, delimiter);
        fs::write(output.join(output_name), compile(source)).expect("write embedded Luau bytecode");
    }
}

fn extract_raw_string_constant<'a>(source: &'a str, constant: &str, delimiter: &str) -> &'a str {
    let start_marker = format!("const {constant}: &str = r{delimiter}\"");
    let end_marker = format!("\n\"{delimiter};");
    let start = source
        .find(&start_marker)
        .map(|offset| offset + start_marker.len())
        .unwrap_or_else(|| panic!("{constant} start marker"));
    let rest = &source[start..];
    let end = rest
        .find(&end_marker)
        .unwrap_or_else(|| panic!("{constant} end marker"));
    &rest[..end]
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
    assert_ne!(
        bytecode.first(),
        Some(&0),
        "embedded Luau source is invalid"
    );
    bytecode
}
