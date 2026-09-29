//! Direct translation of rive/manifest_sections.hpp.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestSections {
    Names = 0,
    Paths = 1,
    Watermark = 2,
}

pub const WATERMARK_SECTION_VERSION: u64 = 1;
pub const WATERMARK_FLAG_ENABLED: u64 = 0x1;

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

/// Appends section ID, payload size, then version, flags and artboard index.
pub fn write_watermark_manifest_section(artboard_index: u32, out: &mut Vec<u8>) {
    use manifest_detail::append_var_uint;
    let mut section = Vec::new();
    append_var_uint(&mut section, WATERMARK_SECTION_VERSION);
    append_var_uint(&mut section, WATERMARK_FLAG_ENABLED);
    append_var_uint(&mut section, u64::from(artboard_index));
    append_var_uint(out, ManifestSections::Watermark as u64);
    append_var_uint(out, section.len() as u64);
    out.extend_from_slice(&section);
}
