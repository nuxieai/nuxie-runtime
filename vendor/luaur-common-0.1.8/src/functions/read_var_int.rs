//! Generated skeleton item.
//! Node: `cxx:Function:Luau.Common:Common/src/BytecodeWire.cpp:26:read_var_int`
//! Source: `Common/src/BytecodeWire.cpp`
//! Graph edges:
//! - declared_by: source_file Common/src/BytecodeWire.cpp
//! - source_includes:
//!   - includes -> source_file Common/include/Luau/BytecodeWire.h
//!   - includes -> source_file Common/include/Luau/Common.h
//! - incoming:
//!   - declares <- source_file Common/src/BytecodeWire.cpp

use crate::functions::read_var_int_64::read_var_int_64;

pub fn read_var_int(data: &[u8], offset: &mut usize) -> u32 {
    read_var_int_64(data, offset) as u32
}
