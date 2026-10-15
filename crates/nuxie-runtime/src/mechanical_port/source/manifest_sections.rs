//! Direct translation of rive/manifest_sections.hpp.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestSections {
    Names = 0,
    Paths = 1,
}

pub mod manifest_detail {
    /// Mirrors BinaryWriter::writeVarUint without requiring a runtime writer.
    pub fn append_var_uint(out: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }
}
