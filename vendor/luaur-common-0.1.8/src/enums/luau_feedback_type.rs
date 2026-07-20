//! Generated skeleton item.
//! Node: `cxx:Enum:Luau.Common:Common/include/Luau/Bytecode.h:759:luau_feedback_type`
//! Source: `Common/include/Luau/Bytecode.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/Bytecode.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/Bytecode.h

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LuauFeedbackType {
    LFT_CALLTARGET = 0,
}

impl LuauFeedbackType {
    pub const LFT_CALLTARGET: Self = Self::LFT_CALLTARGET;
}
