//! Differential-oracle compiler driver: read a `.lua` source file, compile it
//! to Luau bytecode with the *Rust* compiler (luau_compile, null options →
//! optimizationLevel=1/debugLevel=1, same as the C++ `luau-compile --binary`),
//! and write the raw bytecode bytes to stdout. Lets us run the FULL Rust stack
//! (Rust compile → Rust VM) and diff against the C++ compile→run oracle, which
//! exercises the parser/compiler — including Luau type-annotation syntax.

use std::io::{Read, Write};

use luaur_compiler::functions::luau_compile::luau_compile;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: luau_compile <lua-file>");
    let mut src = Vec::new();
    std::fs::File::open(&path)
        .expect("cannot open source file")
        .read_to_end(&mut src)
        .expect("cannot read source file");

    // mirror the CLI's flag state so compilation decisions match the oracle
    luaur_common::set_all_flags(true);

    unsafe {
        let mut outsize: usize = 0;
        let bc = luau_compile(
            src.as_ptr() as *const core::ffi::c_char,
            src.len(),
            core::ptr::null_mut(),
            &mut outsize,
        );
        assert!(!bc.is_null(), "luau_compile returned null");
        let bytes = std::slice::from_raw_parts(bc as *const u8, outsize);
        std::io::stdout()
            .write_all(bytes)
            .expect("cannot write bytecode");
    }
}
