//! Luau helpers compiled into this crate at build time (see `build.rs`).
//!
//! Loading bytecode is `unsafe` on the vendored VM because its loader trusts
//! its input; the safe VM validates it. Only the variants below can be loaded
//! through here, and each one is the output of the pinned build-time compiler
//! for Luau source embedded in this crate.

use crate::luau::{Function, Lua, Result};

#[derive(Clone, Copy, Debug)]
pub(super) enum EmbeddedChunk {
    PromiseLibrary,
    DataValueMetatable,
    PropertyMetatable,
    HostDispatch,
}

impl EmbeddedChunk {
    fn name(self) -> &'static str {
        match self {
            Self::PromiseLibrary => "rive_promise",
            Self::DataValueMetatable => "rive_data_value_metatable",
            Self::PropertyMetatable => "rive_property_metatable",
            Self::HostDispatch => "nuxie_host_dispatch",
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
        }
    }

    /// Load the chunk into an unexecuted function.
    // On the safe VM `load_bytecode` is a safe function (it validates).
    #[cfg_attr(feature = "nuxie-vm", allow(unused_unsafe))]
    pub(super) fn load(self, lua: &Lua) -> Result<Function> {
        // SAFETY: every variant's bytecode is produced by the pinned build-time
        // compiler from source embedded in this crate.
        unsafe { lua.load_bytecode(self.name(), self.bytecode()) }
    }
}
