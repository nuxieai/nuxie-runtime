//! Luau helpers compiled into this crate at build time (see `build.rs`).
//!
//! Loading bytecode is `unsafe` because Luau's loader trusts its input. Only
//! the variants below can be loaded through here, and each one is the output
//! of the pinned build-time compiler for Luau source embedded in this crate.

use luaur_rt::{Function, Lua, Result};

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
    pub(super) fn load(self, lua: &Lua) -> Result<Function> {
        // SAFETY: every variant's bytecode is produced by the pinned build-time
        // compiler from source embedded in this crate.
        unsafe { lua.load_bytecode(self.name(), self.bytecode()) }
    }
}
