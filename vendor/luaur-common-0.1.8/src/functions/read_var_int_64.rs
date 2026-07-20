//! Generated skeleton item.
//! Node: `cxx:Function:Luau.Common:Common/src/BytecodeWire.cpp:9:read_var_int_64`
//! Source: `Common/src/BytecodeWire.cpp`
//! Graph edges:
//! - declared_by: source_file Common/src/BytecodeWire.cpp
//! - source_includes:
//!   - includes -> source_file Common/include/Luau/BytecodeWire.h
//!   - includes -> source_file Common/include/Luau/Common.h
//! - incoming:
//!   - declares <- source_file Common/src/BytecodeWire.cpp

use crate::functions::read::read;

pub fn read_var_int_64(data: &[u8], offset: &mut usize) -> u64 {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    let mut byte: u8;

    loop {
        byte = read::<u8>(data, offset);
        result |= ((byte & 127) as u64) << shift;
        shift += 7;

        if (byte & 128) == 0 {
            break;
        }
    }

    result
}
