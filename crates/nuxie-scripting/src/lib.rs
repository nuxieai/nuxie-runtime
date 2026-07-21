//! Scripting spike for the Rive Rust runtime.
//!
//! Proves out `luaur` (pure-Rust Luau) as the scripting VM. Two layers:
//!
//! - [`envelope`]: dependency-free parsing of Rive's signed-content envelope
//!   that wraps every `ScriptAsset` payload in a `.riv` file
//!   (`[flags:1] [signature:64 if signed] [luau_bytecode:N]`, mirroring
//!   C++ `nuxie::SignedContentHeader`).
//! - [`vm`] (feature `luau`, default on): boot a Luau VM, run source, load
//!   the precompiled Luau bytecode that `.riv` files actually carry, and
//!   call functions / read results.

pub mod envelope;

#[cfg(feature = "luau")]
pub mod gpu_canvas;

#[cfg(feature = "luau")]
mod shader_asset;

#[cfg(feature = "luau")]
pub mod vm;
