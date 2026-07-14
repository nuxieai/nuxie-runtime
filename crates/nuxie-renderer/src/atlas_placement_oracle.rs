//! Canonical interchange format for C++/Rust feather-atlas placement facts.

const MAGIC: [u8; 8] = *b"RIVEATP\0";
const VERSION: u32 = 1;
const FIELD_COUNT: usize = 19;
const BYTE_COUNT: usize = 8 + 4 + FIELD_COUNT * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AtlasPlacement {
    pub(crate) frame_size: [u32; 2],
    pub(crate) bounds: [i32; 4],
    pub(crate) origin: [u32; 2],
    pub(crate) content_size: [u32; 2],
    pub(crate) physical_size: [u32; 2],
    pub(crate) scale_bits: u32,
    pub(crate) translate_bits: [u32; 2],
    pub(crate) scissor: [u32; 4],
}

impl AtlasPlacement {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != BYTE_COUNT {
            return Err(format!(
                "atlas-placement byte length must be {BYTE_COUNT}, got {}",
                bytes.len()
            ));
        }
        if bytes[..8] != MAGIC {
            return Err("invalid atlas-placement magic".into());
        }
        let mut fields = [0u32; FIELD_COUNT + 1];
        for (field, chunk) in fields.iter_mut().zip(bytes[8..].chunks_exact(4)) {
            *field = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        if fields[0] != VERSION {
            return Err(format!("unsupported atlas-placement version {}", fields[0]));
        }
        Ok(Self {
            frame_size: [fields[1], fields[2]],
            bounds: [
                fields[3] as i32,
                fields[4] as i32,
                fields[5] as i32,
                fields[6] as i32,
            ],
            origin: [fields[7], fields[8]],
            content_size: [fields[9], fields[10]],
            physical_size: [fields[11], fields[12]],
            scale_bits: fields[13],
            translate_bits: [fields[14], fields[15]],
            scissor: [fields[16], fields[17], fields[18], fields[19]],
        })
    }

    pub(crate) fn serialize(self) -> Vec<u8> {
        let fields = [
            VERSION,
            self.frame_size[0],
            self.frame_size[1],
            self.bounds[0] as u32,
            self.bounds[1] as u32,
            self.bounds[2] as u32,
            self.bounds[3] as u32,
            self.origin[0],
            self.origin[1],
            self.content_size[0],
            self.content_size[1],
            self.physical_size[0],
            self.physical_size[1],
            self.scale_bits,
            self.translate_bits[0],
            self.translate_bits[1],
            self.scissor[0],
            self.scissor[1],
            self.scissor[2],
            self.scissor[3],
        ];
        let mut bytes = Vec::with_capacity(BYTE_COUNT);
        bytes.extend_from_slice(&MAGIC);
        for field in fields {
            bytes.extend_from_slice(&field.to_le_bytes());
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::AtlasPlacement;

    fn fixture() -> AtlasPlacement {
        AtlasPlacement {
            frame_size: [1756, 2048],
            bounds: [1490, 900, 1756, 1150],
            origin: [0, 0],
            content_size: [101, 95],
            physical_size: [126, 118],
            scale_bits: 0.25f32.to_bits(),
            translate_bits: [(-370.5f32).to_bits(), (-223.0f32).to_bits()],
            scissor: [0, 0, 101, 95],
        }
    }

    #[test]
    fn atlas_placement_round_trips() {
        let fixture = fixture();
        assert_eq!(
            AtlasPlacement::parse(&fixture.serialize()).unwrap(),
            fixture
        );
    }

    #[test]
    fn atlas_placement_rejects_wrong_size_and_magic() {
        let bytes = fixture().serialize();
        assert!(AtlasPlacement::parse(&bytes[..bytes.len() - 1]).is_err());
        let mut wrong_magic = bytes;
        wrong_magic[0] ^= 1;
        assert!(AtlasPlacement::parse(&wrong_magic).is_err());
    }
}
