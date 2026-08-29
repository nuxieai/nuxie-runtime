use std::collections::HashMap;

use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle, generated::assets::shader_asset_base::ShaderAssetBase,
    signed_content_header::SignedContentHeader,
};

#[derive(Clone, Copy)]
pub struct TextureSamplerPair {
    pub tex_group: u8,
    pub tex_binding: u8,
    pub samp_group: u8,
    pub samp_binding: u8,
}

#[derive(Clone, Copy)]
struct ShaderVariant {
    offset: u32,
    size: u32,
}

pub struct ShaderAsset {
    pub base: ShaderAssetBase,
    // Exact imported bytes are retained solely for the host's authenticated
    // shader provenance check; variant ownership remains the fields below.
    encoded_payload: Vec<u8>,
    bytes: Vec<u8>,
    index: HashMap<u8, ShaderVariant>,
    pairs: Vec<TextureSamplerPair>,
}

impl Default for ShaderAsset {
    fn default() -> Self {
        Self {
            base: ShaderAssetBase::default(),
            encoded_payload: Vec::new(),
            bytes: Vec::new(),
            index: HashMap::new(),
            pairs: Vec::new(),
        }
    }
}

impl ShaderAsset {
    pub fn decode_bytes(&mut self, data: &mut Vec<u8>, factory: &RuntimeFactoryHandle) -> bool {
        self.decode(data.as_slice(), factory)
    }

    pub fn decode(&mut self, data: &[u8], _factory: &RuntimeFactoryHandle) -> bool {
        self.encoded_payload = data.to_vec();
        let envelope = SignedContentHeader::new(data);
        if !envelope.is_valid() {
            return false;
        }
        self.bytes = envelope.content().to_vec();
        self.index.clear();
        self.pairs.clear();

        if self.bytes.len() < 8 {
            return false;
        }
        let magic =
            u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]]);
        if magic != 0x5253_5442 {
            return false;
        }
        let version = u16::from_le_bytes([self.bytes[4], self.bytes[5]]);
        if version != 4 {
            return false;
        }
        let variant_count = self.bytes[6];
        let section_count = self.bytes[7];
        let mut cursor = 8usize;

        const VARIANT_DESCRIPTOR_SIZE: usize = 9;
        if cursor + variant_count as usize * VARIANT_DESCRIPTOR_SIZE > self.bytes.len() {
            return false;
        }
        for _ in 0..variant_count {
            let target = self.bytes[cursor];
            let blob_offset = u32::from_le_bytes([
                self.bytes[cursor + 1],
                self.bytes[cursor + 2],
                self.bytes[cursor + 3],
                self.bytes[cursor + 4],
            ]);
            let blob_size = u32::from_le_bytes([
                self.bytes[cursor + 5],
                self.bytes[cursor + 6],
                self.bytes[cursor + 7],
                self.bytes[cursor + 8],
            ]);
            cursor += VARIANT_DESCRIPTOR_SIZE;
            self.index.insert(
                target,
                ShaderVariant {
                    offset: blob_offset,
                    size: blob_size,
                },
            );
        }

        for _ in 0..section_count {
            if cursor + 3 > self.bytes.len() {
                return false;
            }
            let tag = self.bytes[cursor];
            let length =
                u16::from_le_bytes([self.bytes[cursor + 1], self.bytes[cursor + 2]]) as usize;
            cursor += 3;
            if cursor + length > self.bytes.len() {
                return false;
            }
            if tag == 1 && length >= 1 {
                let pair_count = self.bytes[cursor] as usize;
                if length >= 1 + pair_count * 4 {
                    let mut pair_cursor = cursor + 1;
                    for _ in 0..pair_count {
                        self.pairs.push(TextureSamplerPair {
                            tex_group: self.bytes[pair_cursor],
                            tex_binding: self.bytes[pair_cursor + 1],
                            samp_group: self.bytes[pair_cursor + 2],
                            samp_binding: self.bytes[pair_cursor + 3],
                        });
                        pair_cursor += 4;
                    }
                }
            }
            cursor += length;
        }

        let blob_data_start = cursor;
        for variant in self.index.values_mut() {
            let absolute_offset = variant.offset as usize + blob_data_start;
            if absolute_offset > self.bytes.len()
                || variant.size as usize > self.bytes.len() - absolute_offset
            {
                self.index.clear();
                return false;
            }
            variant.offset = absolute_offset as u32;
        }
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "rstb"
    }

    pub fn encoded_payload(&self) -> &[u8] {
        &self.encoded_payload
    }
    pub fn content_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn find_shader(&self, target: u8) -> &[u8] {
        let Some(variant) = self.index.get(&target) else {
            return &[];
        };
        let start = variant.offset as usize;
        &self.bytes[start..start + variant.size as usize]
    }

    pub fn texture_sampler_pairs(&self) -> &[TextureSamplerPair] {
        &self.pairs
    }
}
