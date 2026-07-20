//! Generated skeleton item.
//! Node: `cxx:Enum:Luau.Common:Common/include/Luau/TimeTrace.h:35:event_type`
//! Source: `Common/include/Luau/TimeTrace.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/TimeTrace.h
//! - source_includes:
//!   - includes -> source_file Common/include/Luau/Common.h
//!   - includes -> source_file VM/src/lstring.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/TimeTrace.h

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    Enter = 0,
    Leave = 1,
    ArgName = 2,
    ArgValue = 3,
}

#[allow(non_upper_case_globals)]
impl EventType {
    pub const Enter: Self = Self::Enter;
    pub const Leave: Self = Self::Leave;
    pub const ArgName: Self = Self::ArgName;
    pub const ArgValue: Self = Self::ArgValue;
}
