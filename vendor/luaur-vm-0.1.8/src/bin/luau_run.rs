//! Differential-oracle driver: load precompiled Luau bytecode (produced by
//! the C++ `luau-compile --binary`) and run it on the Rust VM. Errors —
//! including `todo!()` panics from not-yet-ported functions — surface as Lua
//! errors via the catch_unwind in luaD_rawrunprotected, so every run either
//! prints results or names the next function to port.
//!
//! Mirrors the real `luau` CLI: the chunk is loaded into a fresh thread and
//! run with `lua_resume` (not `lua_pcall` on the main thread), so top-level
//! `coroutine.running()`/`isyieldable()` behave as in the reference CLI.

use std::io::Read;

use luaur_vm::functions::lua_gettop::lua_gettop;
use luaur_vm::functions::lua_l_newstate::lua_l_newstate;
use luaur_vm::functions::lua_l_openlibs::lua_l_openlibs;
use luaur_vm::functions::lua_newthread::lua_newthread;
use luaur_vm::functions::lua_resume::lua_resume;
use luaur_vm::functions::lua_tolstring::lua_tolstring;
use luaur_vm::functions::lua_tonumberx::lua_tonumberx;
use luaur_vm::functions::lua_type::lua_type;
use luaur_vm::functions::luau_load::luau_load;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));

    let path = std::env::args()
        .nth(1)
        .expect("usage: luau_run <bytecode-file>");
    let mut bc = Vec::new();
    std::fs::File::open(&path)
        .expect("cannot open bytecode file")
        .read_to_end(&mut bc)
        .expect("cannot read bytecode file");

    // mirror the C++ CLI: setLuauFlagsDefault(true) — v11+ bytecode needs it
    luaur_common::set_all_flags(true);

    unsafe {
        let l = lua_l_newstate();
        assert!(!l.is_null(), "lua_l_newstate returned null");
        lua_l_openlibs(l);

        // Run the chunk on a fresh thread, like CLI/src/Repl.cpp's runCode: the
        // thread T is rooted on L's stack, and we load the function directly into
        // T (the global string table / GC is shared), then resume it.
        let t = lua_newthread(l);
        assert!(!t.is_null(), "lua_newthread returned null");

        let rc = luau_load(
            t,
            c"=script".as_ptr(),
            bc.as_ptr() as *const core::ffi::c_char,
            bc.len(),
            0,
        );
        if rc != 0 {
            eprintln!("luau_load failed: rc={rc}");
            std::process::exit(2);
        }

        let status = lua_resume(t, core::ptr::null_mut(), 0);
        if status != 0 {
            // The error object is on top of T's stack — surface its text so the
            // differential oracle reports WHY a run failed, not just the status.
            let mut len = 0usize;
            let s = lua_tolstring(t, -1, &mut len);
            let msg = if s.is_null() {
                "<non-string error>".to_string()
            } else {
                let bytes = std::slice::from_raw_parts(s as *const u8, len);
                String::from_utf8_lossy(bytes).into_owned()
            };
            eprintln!("pcall status={status}: {msg}");
            std::process::exit(3);
        }

        let n = lua_gettop(t);
        println!("results: {n}");
        for i in 1..=n {
            if lua_type(t, i) == luaur_vm::enums::lua_type::lua_Type::LUA_TSTRING as i32 {
                let mut len = 0usize;
                let s = lua_tolstring(t, i, &mut len);
                if !s.is_null() {
                    let bytes = std::slice::from_raw_parts(s as *const u8, len);
                    let text = String::from_utf8_lossy(bytes);
                    println!("  [{i}] = {:?}", text.as_ref());
                } else {
                    println!("  [{i}] = <non-number>");
                }
            } else {
                let mut isnum: core::ffi::c_int = 0;
                let v = lua_tonumberx(t, i, &mut isnum);
                if isnum != 0 {
                    println!("  [{i}] = {v}");
                } else {
                    let mut len = 0usize;
                    let s = lua_tolstring(t, i, &mut len);
                    if !s.is_null() {
                        let bytes = std::slice::from_raw_parts(s as *const u8, len);
                        let text = String::from_utf8_lossy(bytes);
                        println!("  [{i}] = {:?}", text.as_ref());
                    } else {
                        println!("  [{i}] = <non-number>");
                    }
                }
            }
        }
    }
}
