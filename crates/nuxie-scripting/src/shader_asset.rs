//! Bounded decoder for the canonical RSTB v4 GLSL payload in `ShaderAsset`.

use nuxie_render_api::{GpuCanvasShader, GpuCanvasShaderStage};

use crate::envelope::SignedContent;
use crate::vm::{Error, Result};

const RSTB_MAGIC: u32 = 0x5253_5442;
const RSTB_VERSION: u16 = 4;
const GLSL_SOURCE_TARGET: u8 = 1;
const MAX_RSTB_BYTES: usize = 4 * 1024 * 1024;
const MAX_SHADER_STAGE_BYTES: usize = 1024 * 1024;
const MAX_VARIANTS: usize = 32;
const MAX_SECTIONS: usize = 32;
const MAX_ENTRIES: usize = 8;

#[derive(Debug, Clone, Copy)]
struct VariantDescriptor {
    target: u8,
    offset: usize,
    size: usize,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn read_bytes(&mut self, length: usize, label: &str) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| Error::runtime(format!("RSTB is truncated in {label}")))?;
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn read_u8(&mut self, label: &str) -> Result<u8> {
        Ok(self.read_bytes(1, label)?[0])
    }

    fn read_u16(&mut self, label: &str) -> Result<u16> {
        let bytes: [u8; 2] = self
            .read_bytes(2, label)?
            .try_into()
            .map_err(|_| Error::runtime(format!("RSTB is truncated in {label}")))?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, label: &str) -> Result<u32> {
        let bytes: [u8; 4] = self
            .read_bytes(4, label)?
            .try_into()
            .map_err(|_| Error::runtime(format!("RSTB is truncated in {label}")))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_string(&mut self, label: &str) -> Result<String> {
        let length = usize::from(self.read_u16(label)?);
        let bytes = self.read_bytes(length, label)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::runtime(format!("RSTB {label} is not UTF-8")))
    }
}

pub(crate) fn decode_shader_asset(name: &str, payload: &[u8]) -> Result<GpuCanvasShader> {
    let envelope = SignedContent::parse(payload)
        .map_err(|error| Error::runtime(format!("ShaderAsset '{name}': {error}")))?;
    let rstb = envelope.content;
    if rstb.len() > MAX_RSTB_BYTES {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' RSTB exceeds {MAX_RSTB_BYTES} bytes"
        )));
    }

    let mut cursor = Cursor::new(rstb);
    if cursor.read_u32("magic")? != RSTB_MAGIC {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' has invalid RSTB magic"
        )));
    }
    if cursor.read_u16("version")? != RSTB_VERSION {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' must use RSTB version {RSTB_VERSION}"
        )));
    }
    let variant_count = usize::from(cursor.read_u8("variant count")?);
    let section_count = usize::from(cursor.read_u8("section count")?);
    if variant_count == 0 || variant_count > MAX_VARIANTS {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' RSTB variant count must be between 1 and {MAX_VARIANTS}"
        )));
    }
    if section_count > MAX_SECTIONS {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' RSTB section count exceeds {MAX_SECTIONS}"
        )));
    }

    let mut descriptors = Vec::with_capacity(variant_count);
    for _ in 0..variant_count {
        descriptors.push(VariantDescriptor {
            target: cursor.read_u8("variant target")?,
            offset: usize::try_from(cursor.read_u32("variant offset")?)
                .map_err(|_| Error::runtime("RSTB variant offset is not addressable"))?,
            size: usize::try_from(cursor.read_u32("variant size")?)
                .map_err(|_| Error::runtime("RSTB variant size is not addressable"))?,
        });
    }
    for _ in 0..section_count {
        let _tag = cursor.read_u8("section tag")?;
        let length = usize::from(cursor.read_u16("section length")?);
        cursor.read_bytes(length, "section payload")?;
    }

    let blob_data = cursor.read_bytes(cursor.remaining(), "blob data")?;
    let mut expected_offset = 0usize;
    let mut glsl = None;
    for descriptor in descriptors {
        if descriptor.offset != expected_offset {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' RSTB variants are not canonically packed"
            )));
        }
        let end = descriptor
            .offset
            .checked_add(descriptor.size)
            .filter(|end| *end <= blob_data.len())
            .ok_or_else(|| {
                Error::runtime(format!("ShaderAsset '{name}' RSTB variant is truncated"))
            })?;
        let bytes = &blob_data[descriptor.offset..end];
        if descriptor.target == GLSL_SOURCE_TARGET {
            if glsl.replace(bytes).is_some() {
                return Err(Error::runtime(format!(
                    "ShaderAsset '{name}' has duplicate RSTB GLSL source variants"
                )));
            }
        }
        expected_offset = end;
    }
    if expected_offset != blob_data.len() {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' RSTB has trailing blob bytes"
        )));
    }
    let glsl = glsl.ok_or_else(|| {
        Error::runtime(format!(
            "ShaderAsset '{name}' has no canonical RSTB target-1 GLSL source"
        ))
    })?;
    decode_per_entry_glsl(name, glsl)
}

fn decode_per_entry_glsl(name: &str, bytes: &[u8]) -> Result<GpuCanvasShader> {
    let mut cursor = Cursor::new(bytes);
    let entry_count = usize::from(cursor.read_u8("GLSL entry count")?);
    if entry_count == 0 || entry_count > MAX_ENTRIES {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' GLSL entry count must be between 1 and {MAX_ENTRIES}"
        )));
    }
    let mut vertex = None;
    let mut fragment = None;
    for _ in 0..entry_count {
        let stage = cursor.read_u8("GLSL stage")?;
        let logical_entry_point = cursor.read_string("GLSL logical entry point")?;
        let physical_entry_point = cursor.read_string("GLSL physical entry point")?;
        let source_length = usize::try_from(cursor.read_u32("GLSL source length")?)
            .map_err(|_| Error::runtime("GLSL source length is not addressable"))?;
        if source_length == 0 || source_length > MAX_SHADER_STAGE_BYTES {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' GLSL stage size must be between 1 and {MAX_SHADER_STAGE_BYTES} bytes"
            )));
        }
        let source = std::str::from_utf8(cursor.read_bytes(source_length, "GLSL source")?)
            .map_err(|_| Error::runtime(format!("ShaderAsset '{name}' GLSL source is not UTF-8")))?
            .to_owned();
        let decoded = GpuCanvasShaderStage {
            source,
            logical_entry_point,
            physical_entry_point,
        };
        let destination = match stage {
            0 => &mut vertex,
            1 => &mut fragment,
            other => {
                return Err(Error::runtime(format!(
                    "ShaderAsset '{name}' GLSL stage {other} is unsupported"
                )));
            }
        };
        if destination.replace(decoded).is_some() {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' has duplicate GLSL stages"
            )));
        }
    }
    if cursor.remaining() != 0 {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' GLSL source container has trailing bytes"
        )));
    }
    let vertex = vertex
        .ok_or_else(|| Error::runtime(format!("ShaderAsset '{name}' has no GLSL vertex stage")))?;
    let fragment = fragment.ok_or_else(|| {
        Error::runtime(format!("ShaderAsset '{name}' has no GLSL fragment stage"))
    })?;
    if vertex.logical_entry_point != "vs_main" || fragment.logical_entry_point != "fs_main" {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' must expose logical GLSL entries vs_main and fs_main"
        )));
    }
    Ok(GpuCanvasShader { vertex, fragment })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        put_u16(bytes, value.len() as u16);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn canonical_payload() -> Vec<u8> {
        let mut entries = vec![2];
        for (stage, logical, source) in [
            (0, "vs_main", "#version 300 es\nvoid main() {}"),
            (1, "fs_main", "#version 300 es\nvoid main() {}"),
        ] {
            entries.push(stage);
            put_string(&mut entries, logical);
            put_string(&mut entries, "main");
            put_u32(&mut entries, source.len() as u32);
            entries.extend_from_slice(source.as_bytes());
        }
        let mut payload = vec![0];
        put_u32(&mut payload, RSTB_MAGIC);
        put_u16(&mut payload, RSTB_VERSION);
        payload.extend_from_slice(&[1, 0, GLSL_SOURCE_TARGET]);
        put_u32(&mut payload, 0);
        put_u32(&mut payload, entries.len() as u32);
        payload.extend(entries);
        payload
    }

    #[test]
    fn decodes_canonical_rstb_v4_target_one_glsl() {
        let shader = decode_shader_asset("scene", &canonical_payload()).unwrap();
        assert_eq!(shader.vertex.logical_entry_point, "vs_main");
        assert_eq!(shader.vertex.physical_entry_point, "main");
        assert_eq!(shader.fragment.logical_entry_point, "fs_main");
        assert!(shader.fragment.source.starts_with("#version 300 es"));
    }

    #[test]
    fn rejects_noncanonical_variant_offsets_and_missing_stages() {
        let mut payload = canonical_payload();
        payload[10..14].copy_from_slice(&1u32.to_le_bytes());
        assert!(decode_shader_asset("scene", &payload).is_err());

        let mut payload = canonical_payload();
        let descriptor_size = 9usize;
        let blob_start = 1 + 8 + descriptor_size;
        payload[blob_start] = 1;
        assert!(decode_shader_asset("scene", &payload).is_err());
    }
}
