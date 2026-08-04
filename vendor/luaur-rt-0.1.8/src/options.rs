//! [`StdLib`] and [`LuaOptions`] — the standard-library selection and the
//! per-VM behavioral options accepted by [`Lua::new_with`](crate::Lua::new_with).
//!
//! Mirrors `mlua::StdLib` / `mlua::LuaOptions`. luaur opens the Luau standard
//! library wholesale (`luaL_openlibs`), so the [`StdLib`] bit-set is provided
//! for signature parity with mlua: the practically meaningful distinction is
//! [`StdLib::NONE`] (open nothing) vs. anything else (open the full Luau base
//! libraries). The Lua-5.x-specific libraries that mlua's flags name (`debug`,
//! `package`, `ffi`, ...) are not separable in Luau and are documented as such.

use std::ops::{BitOr, BitOrAssign};

/// A bit-set selecting which standard libraries to open. Mirrors `mlua::StdLib`.
///
/// **DEVIATION:** luaur opens the Luau base libraries as a single unit
/// (`luaL_openlibs`); the individual Lua-5.x library bits cannot be toggled
/// independently. [`StdLib::NONE`] opens nothing; any non-empty selection opens
/// the full Luau standard library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdLib(u32);

impl StdLib {
    /// Open no standard libraries.
    pub const NONE: StdLib = StdLib(0);
    /// The Luau `coroutine` library (part of the base set).
    pub const COROUTINE: StdLib = StdLib(1 << 0);
    /// The `table` library.
    pub const TABLE: StdLib = StdLib(1 << 1);
    /// The `os` library.
    pub const OS: StdLib = StdLib(1 << 2);
    /// The `string` library.
    pub const STRING: StdLib = StdLib(1 << 3);
    /// The `math` library.
    pub const MATH: StdLib = StdLib(1 << 4);
    /// The Luau-specific `bit32` / `buffer` / `vector` libraries.
    pub const BIT32: StdLib = StdLib(1 << 5);
    /// The base library (`print`, `assert`, `pcall`, ...).
    pub const BASE: StdLib = StdLib(1 << 6);
    /// Every safe library (the default Luau set opened by `luaL_openlibs`).
    pub const ALL_SAFE: StdLib = StdLib(u32::MAX & !(1 << 31));
    /// Every library, including unsafe ones (same as [`StdLib::ALL_SAFE`] in
    /// Luau, which has no unsafe `debug`/`ffi` libraries in its base set).
    pub const ALL: StdLib = StdLib(u32::MAX);

    /// Whether this selection is empty (opens nothing).
    pub(crate) fn is_none(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for StdLib {
    type Output = StdLib;
    fn bitor(self, rhs: StdLib) -> StdLib {
        StdLib(self.0 | rhs.0)
    }
}

impl BitOrAssign for StdLib {
    fn bitor_assign(&mut self, rhs: StdLib) {
        self.0 |= rhs.0;
    }
}

/// Per-VM behavioral options. Mirrors `mlua::LuaOptions`.
#[derive(Debug, Clone, Copy)]
pub struct LuaOptions {
    /// Whether a Rust panic raised inside a callback should be **caught** and
    /// converted into a catchable Lua error (the default, `true`), or allowed to
    /// propagate across the VM boundary as a Rust unwind (`false`). Mirrors
    /// `mlua::LuaOptions::catch_rust_panics`.
    pub(crate) catch_rust_panics: bool,
}

impl LuaOptions {
    /// The default options (`catch_rust_panics = true`). Mirrors
    /// `mlua::LuaOptions::new`.
    pub const fn new() -> LuaOptions {
        LuaOptions {
            catch_rust_panics: true,
        }
    }

    /// Set whether Rust panics in callbacks are caught and converted to Lua
    /// errors. Mirrors `mlua::LuaOptions::catch_rust_panics`.
    pub const fn catch_rust_panics(mut self, enabled: bool) -> LuaOptions {
        self.catch_rust_panics = enabled;
        self
    }
}

impl Default for LuaOptions {
    fn default() -> Self {
        LuaOptions::new()
    }
}
