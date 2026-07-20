//! Generated skeleton item.
//! Node: `cxx:Enum:Luau.Common:Common/include/Luau/Bytecode.h:747:luau_proto_flag`
//! Source: `Common/include/Luau/Bytecode.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/Bytecode.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/Bytecode.h

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LuauProtoFlag {
    /// used to tag main proto for modules with --!native
    LPF_NATIVE_MODULE = 1 << 0,
    /// used to tag individual protos as not profitable to compile natively
    LPF_NATIVE_COLD = 1 << 1,
    /// used to tag main proto for modules that have at least one function with native attribute
    LPF_NATIVE_FUNCTION = 1 << 2,
    /// function can be inlined
    LPF_INLINABLE = 1 << 3,
}

impl LuauProtoFlag {
    pub const LPF_NATIVE_MODULE: Self = Self::LPF_NATIVE_MODULE;
    pub const LPF_NATIVE_COLD: Self = Self::LPF_NATIVE_COLD;
    pub const LPF_NATIVE_FUNCTION: Self = Self::LPF_NATIVE_FUNCTION;
    pub const LPF_INLINABLE: Self = Self::LPF_INLINABLE;
}
