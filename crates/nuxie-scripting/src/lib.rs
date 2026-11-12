//! Scripting spike for the Rive Rust runtime.
//!
//! The Luau scripting host. Two layers:
//!
//! - [`envelope`]: dependency-free parsing of Rive's signed-content envelope
//!   that wraps every `ScriptAsset` payload in a `.riv` file
//!   (`[flags:1] [signature:64 if signed] [luau_bytecode:N]`, mirroring
//!   C++ `nuxie::SignedContentHeader`).
//! - [`vm`] (feature `luau`, default on; the safe VM in `nuxie-luau-vm`, or
//!   the vendored `luaur-rt` under `vendored-vm`): boot a Luau VM, load the precompiled
//!   Luau bytecode that `.riv` files actually carry, and call functions / read
//!   results. Source compilation belongs to editor tooling.

pub mod envelope;
pub mod logging_scripting_context;
pub use logging_scripting_context::make_logging_scripting_context_factory;

#[cfg(feature = "luau-host")]
pub mod gpu_canvas;

#[cfg(feature = "luau-host")]
pub mod host_commands;

#[cfg(feature = "luau-host")]
mod shader_asset;

#[cfg(feature = "luau-host")]
pub use shader_asset::{decode_browser_shader_asset, decode_trusted_shader_asset};

#[cfg(feature = "luau-host")]
pub mod luau;

#[cfg(feature = "luau-host")]
pub mod vm;

#[cfg(feature = "luau-host")]
pub use vm::{BytecodeValidationError, validate_executable_luau_bytecode};

/// The Luau handle types a host extension names when it reads script
/// arguments or builds return values (`vm::ScriptVm::lua` returns this `Lua`).
#[cfg(feature = "luau-host")]
pub use crate::luau::{Lua, Value};
