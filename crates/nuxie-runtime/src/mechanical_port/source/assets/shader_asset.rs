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
    entry_bindings_offset: usize,
    entry_bindings_size: usize,
    indexed: bool,
    imported: bool,
    require_signed_scripts: bool,
    import_finished: bool,
    refusal_logged: bool,
}

impl Default for ShaderAsset {
    fn default() -> Self {
        Self {
            base: ShaderAssetBase::default(),
            encoded_payload: Vec::new(),
            bytes: Vec::new(),
            index: HashMap::new(),
            pairs: Vec::new(),
            entry_bindings_offset: 0,
            entry_bindings_size: 0,
            indexed: false,
            imported: false,
            require_signed_scripts: false,
            import_finished: false,
            refusal_logged: false,
        }
    }
}

impl ShaderAsset {
    pub fn decode_bytes(&mut self, data: &mut Vec<u8>, factory: &RuntimeFactoryHandle) -> bool {
        self.decode(data.as_slice(), factory)
    }

    pub fn decode(&mut self, data: &[u8], _factory: &RuntimeFactoryHandle) -> bool {
        self.decode_envelope(data, false)
    }

    /// Decode in-band shader bytes the host authorized with the file that
    /// carries them, exactly as `ScriptAsset::decode_with_host_authorization`
    /// does for that file's scripts. Upstream verifies a file's in-band Luau
    /// bytecode and RSTB blobs as one group, so the authority that admits a
    /// file's scripts admits its shaders too.
    ///
    /// # Safety
    /// The caller must have authorized execution of these exact original
    /// in-band bytes. A capability for another artifact, or bytes substituted
    /// by an external asset loader, does not satisfy this requirement.
    pub unsafe fn decode_with_host_authorization(
        &mut self,
        data: &[u8],
        _factory: &RuntimeFactoryHandle,
    ) -> bool {
        self.decode_envelope(data, true)
    }

    fn decode_envelope(&mut self, data: &[u8], host_authorized: bool) -> bool {
        let envelope = SignedContentHeader::new(data);
        if !envelope.is_valid() {
            return false;
        }
        self.encoded_payload = data.to_vec();
        self.base.set_verified(
            host_authorized
                || (envelope.is_signed()
                    && crate::source::importers::text_asset_importer::verifies_content_signature(
                        envelope.signature(),
                        envelope.content(),
                    )),
        );
        self.refusal_logged = false;
        self.bytes = envelope.content().to_vec();
        self.indexed = false;
        let indexed = self.admit();
        if self.import_finished {
            self.log_refusal();
        }
        indexed
    }

    pub fn imported_with(&mut self, require_signed_scripts: bool) {
        self.imported = true;
        self.require_signed_scripts = require_signed_scripts;
    }

    fn accepted(&self) -> bool {
        !self.imported
            || crate::source::file::File::accepts_script(
                self.require_signed_scripts,
                self.base.verified(),
            )
    }

    pub fn admit(&mut self) -> bool {
        if self.indexed || !self.accepted() {
            return true;
        }
        self.indexed = self.index();
        self.indexed
    }

    /// Whether this file refused the shader's content because its signature
    /// did not verify, so `rstb` is empty although content was decoded.
    pub fn refused(&self) -> bool {
        !self.indexed && !self.bytes.is_empty() && !self.accepted()
    }

    pub fn finish_import(&mut self) {
        self.import_finished = true;
        self.log_refusal();
    }

    fn log_refusal(&mut self) {
        if self.indexed || self.refusal_logged || self.bytes.is_empty() || self.accepted() {
            return;
        }
        self.refusal_logged = true;
        eprintln!(
            "Shader '{}' is unavailable: its signature did not verify.",
            self.base.name()
        );
    }

    fn index(&mut self) -> bool {
        self.index.clear();
        self.pairs.clear();
        self.entry_bindings_offset = 0;
        self.entry_bindings_size = 0;

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
            if tag == 2 && length >= 1 {
                self.entry_bindings_offset = cursor;
                self.entry_bindings_size = length;
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

    pub fn entry_bindings(&self) -> &[u8] {
        if !self.indexed || self.entry_bindings_size == 0 {
            return &[];
        }
        &self.bytes
            [self.entry_bindings_offset..self.entry_bindings_offset + self.entry_bindings_size]
    }

    pub fn encoded_payload(&self) -> &[u8] {
        &self.encoded_payload
    }
    /// Decoded RSTB container for forwarding across process or module boundaries.
    pub fn rstb(&self) -> &[u8] {
        if !self.indexed {
            return &[];
        }
        &self.bytes
    }

    pub fn find_shader(&self, target: u8) -> &[u8] {
        if !self.indexed {
            return &[];
        }
        let Some(variant) = self.index.get(&target) else {
            return &[];
        };
        let start = variant.offset as usize;
        &self.bytes[start..start + variant.size as usize]
    }

    pub fn texture_sampler_pairs(&self) -> &[TextureSamplerPair] {
        if !self.indexed {
            return &[];
        }
        &self.pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    /// An unsigned envelope around an RSTB table with no variants.
    const UNSIGNED_RSTB: [u8; 9] = [0, 0x42, 0x54, 0x53, 0x52, 4, 0, 0, 0];

    #[test]
    fn host_authorized_in_band_bytes_are_admitted_like_the_files_scripts() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        // A file that requires signed scripts refuses unsigned shader bytes in
        // every build, so it shows what host authorization changes.
        let mut plain = ShaderAsset::default();
        plain.imported_with(true);
        assert!(plain.decode(&UNSIGNED_RSTB, &factory));
        assert!(!plain.base.verified());
        assert!(plain.rstb().is_empty());
        assert!(plain.refused());

        let mut authorized = ShaderAsset::default();
        authorized.imported_with(true);
        // SAFETY: the test owns these exact in-band bytes.
        assert!(unsafe { authorized.decode_with_host_authorization(&UNSIGNED_RSTB, &factory) });
        assert!(authorized.base.verified());
        assert_eq!(authorized.rstb(), &UNSIGNED_RSTB[1..]);
        assert!(!authorized.refused());
    }
}
