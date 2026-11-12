//! Luau helpers compiled into this crate at build time (see `build.rs`).
//!
//! Loading bytecode is `unsafe` on the vendored VM because its loader trusts
//! its input; the safe VM validates it. Only the variants below can be loaded
//! through here, and each one is the output of the pinned build-time compiler
//! for Luau source embedded in this crate.

use crate::luau::{FromLuaMulti, Function, IntoLuaMulti, Lua, Result, VmState};

#[derive(Clone, Copy, Debug)]
pub(super) enum EmbeddedChunk {
    PromiseLibrary,
    DataValueMetatable,
    PropertyMetatable,
    HostDispatch,
    VectorConstructors,
}

impl EmbeddedChunk {
    fn name(self) -> &'static str {
        match self {
            Self::PromiseLibrary => "rive_promise",
            Self::DataValueMetatable => "rive_data_value_metatable",
            Self::PropertyMetatable => "rive_property_metatable",
            Self::HostDispatch => "nuxie_host_dispatch",
            Self::VectorConstructors => "nuxie_vector_constructors",
        }
    }

    fn bytecode(self) -> &'static [u8] {
        match self {
            Self::PromiseLibrary => {
                include_bytes!(concat!(env!("OUT_DIR"), "/promise-library.luau-bytecode"))
            }
            Self::DataValueMetatable => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/data-value-metatable.luau-bytecode"
            )),
            Self::PropertyMetatable => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/property-metatable.luau-bytecode"
            )),
            Self::HostDispatch => {
                include_bytes!(concat!(env!("OUT_DIR"), "/host-dispatch.luau-bytecode"))
            }
            Self::VectorConstructors => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/vector-constructors.luau-bytecode"
            )),
        }
    }

    /// Load the chunk into an unexecuted function.
    // On the safe VM `load_bytecode` is a safe function (it validates).
    #[cfg_attr(not(feature = "vendored-vm"), allow(unused_unsafe))]
    pub(super) fn load(self, lua: &Lua) -> Result<Function> {
        // SAFETY: every variant's bytecode is produced by the pinned build-time
        // compiler from source embedded in this crate.
        unsafe { lua.load_bytecode(self.name(), self.bytecode()) }
    }

    /// Load and run the chunk as the runtime's own Luau (see [`call`]).
    pub(super) fn run<R: FromLuaMulti>(self, lua: &Lua) -> Result<R> {
        call(lua, &self.load(lua)?, ())
    }
}

/// Call `function`, an embedded chunk or a function one returned, from the
/// host to set up globals or metatables. Upstream does this setup in C++,
/// where it runs no Luau and reaches no interrupt safepoint; here it is
/// Luau, so the script interrupt and the budget it counts against are set
/// aside while it runs. A file's scripts never run inside this call: what it
/// installs is counted as usual when a script calls it later.
pub(super) fn call<R: FromLuaMulti>(
    lua: &Lua,
    function: &Function,
    args: impl IntoLuaMulti,
) -> Result<R> {
    let _set_aside = lua.scoped_interrupt(|_| Ok(VmState::Continue));
    function.call(args)
}
