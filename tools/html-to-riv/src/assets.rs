//! Compiler-owned image admission. Decode portably before embedding exact bytes;
//! no runtime factory, platform decoder, asset loader, or pixel rewriting.
use crate::{
    Diagnostic,
    wire::{Record, Value},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Cursor};
#[path = "jpeg_validation.rs"]
mod jpeg_validation;

const MAX_ASSETS: usize = 256;
const MAX_ENCODED_BYTES: usize = 16 * 1024 * 1024;
const MAX_AXIS: u32 = 8192;
const MAX_RGBA_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Asset {
    Image { bytes: Vec<u8> },
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            kind: String,
            bytes: Vec<u8>,
        }
        let Fields { kind, bytes } = crate::request::object(deserializer, "an image asset object")?;
        if kind != "image" {
            return Err(serde::de::Error::unknown_variant(&kind, &["image"]));
        }
        Ok(Self::Image { bytes })
    }
}

pub type AssetMap = BTreeMap<String, Asset>;

pub(crate) fn deserialize_asset_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<AssetMap, D::Error> {
    struct Assets;
    impl<'de> serde::de::Visitor<'de> for Assets {
        type Value = AssetMap;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an asset map object with unique keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<AssetMap, A::Error> {
            let mut result = AssetMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if result.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate asset key {key:?}"
                    )));
                }
                result.insert(key, map.next_value::<Asset>()?);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Assets)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ImageMetadata {
    /// Zero-based Backboard FileAsset vector index, not an authored asset id.
    pub asset_index: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
struct Image {
    metadata: ImageMetadata,
    bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub(crate) struct AssetTable {
    names: BTreeMap<String, ImageMetadata>,
    images: Vec<Image>,
}

impl AssetTable {
    pub(crate) fn validate(assets: &AssetMap) -> Result<Self, Diagnostic> {
        if assets.len() > MAX_ASSETS {
            return Err(Diagnostic::new(
                "asset-limit",
                "assets",
                "At most 256 supplied assets are admitted",
            ));
        }
        // Count supplied bytes before interning; aliases cannot bypass this cap.
        let mut total = 0usize;
        for (name, Asset::Image { bytes }) in assets {
            let source = asset_source(name);
            if name.is_empty() {
                return Err(Diagnostic::new(
                    "invalid-asset",
                    &source,
                    "Asset keys must be nonempty",
                ));
            }
            total = total
                .checked_add(bytes.len())
                .filter(|n| *n <= MAX_ENCODED_BYTES)
                .ok_or_else(|| {
                    Diagnostic::new(
                        "asset-limit",
                        &source,
                        "Supplied encoded assets exceed 16 MiB in total",
                    )
                })?;
        }
        let mut table = Self::default();
        let mut interned: BTreeMap<&[u8], ImageMetadata> = BTreeMap::new();
        for (name, Asset::Image { bytes }) in assets {
            let metadata = if let Some(metadata) = interned.get(bytes.as_slice()) {
                *metadata
            } else {
                let (width, height) = decode_image(bytes, &asset_source(name))?;
                let metadata = ImageMetadata {
                    asset_index: table.images.len() as u32,
                    width,
                    height,
                };
                interned.insert(bytes, metadata);
                table.images.push(Image {
                    metadata,
                    bytes: bytes.clone(),
                });
                metadata
            };
            table.names.insert(name.clone(), metadata);
        }
        Ok(table)
    }

    pub(crate) fn get(&self, name: &str) -> Result<ImageMetadata, Diagnostic> {
        self.names.get(name).copied().ok_or_else(|| {
            Diagnostic::new(
                "missing-image",
                "src",
                format!("No supplied image asset matches src {name:?}"),
            )
        })
    }

    /// All supplied assets, including unused ones, are validated and emitted.
    /// Stable key order chooses the first index for aliases with identical bytes.
    pub(crate) fn global_records(&self) -> Result<Vec<Record>, Diagnostic> {
        let mut records = Vec::with_capacity(self.images.len() * 2);
        for image in &self.images {
            let metadata = image.metadata;
            let mut asset = Record::new("ImageAsset");
            asset.set(
                "name",
                Value::String(format!("image{}", metadata.asset_index)),
            )?;
            asset.set("assetId", Value::Uint(metadata.asset_index + 1))?;
            asset.set("width", Value::Float(metadata.width as f32))?;
            asset.set("height", Value::Float(metadata.height as f32))?;
            records.push(asset);
            let mut contents = Record::new("FileAssetContents");
            contents.set("bytes", Value::Bytes(image.bytes.clone()))?;
            records.push(contents);
        }
        Ok(records)
    }
}

fn asset_source(name: &str) -> String {
    format!(
        "assets[{}]",
        serde_json::to_string(name).expect("a string is serializable")
    )
}
fn invalid(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("invalid-image", source, message)
}
fn unsupported(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("unsupported-image", source, message)
}
fn dimensions(width: u32, height: u32, source: &str) -> Result<usize, Diagnostic> {
    if width == 0 || height == 0 {
        return Err(invalid(source, "Image dimensions must be positive"));
    }
    if width > MAX_AXIS || height > MAX_AXIS {
        return Err(Diagnostic::new(
            "asset-limit",
            source,
            "Each image axis must be at most 8192 pixels and its decoded RGBA footprint at most 16 MiB",
        ));
    }
    let rgba = u64::from(width) * u64::from(height) * 4;
    if rgba > MAX_RGBA_BYTES as u64 {
        return Err(Diagnostic::new(
            "asset-limit",
            source,
            "Each image axis must be at most 8192 pixels and its decoded RGBA footprint at most 16 MiB",
        ));
    }
    Ok(rgba as usize)
}
fn decode_image(bytes: &[u8], source: &str) -> Result<(u32, u32), Diagnostic> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(bytes, source)
    } else if bytes.starts_with(&[0xff, 0xd8]) {
        decode_jpeg(bytes, source)
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        decode_webp(bytes, source)
    } else {
        Err(unsupported(
            source,
            "Expected a static unprofiled PNG, JPEG, or WebP image",
        ))
    }
}
fn be32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().expect("checked four-byte field"))
}
fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("checked four-byte field"))
}

fn decode_png(bytes: &[u8], source: &str) -> Result<(u32, u32), Diagnostic> {
    let mut pos = 8usize;
    let mut header = None;
    let mut idat = Vec::new();
    let mut ended = false;
    while pos < bytes.len() {
        let prefix = bytes
            .get(pos..pos.saturating_add(8))
            .ok_or_else(|| invalid(source, "Truncated PNG chunk header"))?;
        let len = be32(&prefix[..4]) as usize;
        let end = pos
            .checked_add(12)
            .and_then(|n| n.checked_add(len))
            .filter(|n| *n <= bytes.len())
            .ok_or_else(|| invalid(source, "Truncated PNG chunk payload or CRC"))?;
        let data = &bytes[pos + 8..end - 4];
        match &prefix[4..] {
            b"IHDR" if pos == 8 && len == 13 => {
                let (width, height) = (be32(&data[..4]), be32(&data[4..8]));
                dimensions(width, height, source)?;
                if data[8] != 8 || ![2, 6].contains(&data[9]) || data[10..] != [0, 0, 0] {
                    return Err(unsupported(
                        source,
                        "Use a noninterlaced 8-bit RGB/RGBA PNG with standard compression and filters",
                    ));
                }
                header = Some((width, height, if data[9] == 2 { 3usize } else { 4 }));
            }
            b"IDAT" if header.is_some() => idat.extend_from_slice(data),
            b"IEND" if len == 0 && header.is_some() && !idat.is_empty() && end == bytes.len() => {
                ended = true;
            }
            b"IHDR" | b"IDAT" | b"IEND" => {
                return Err(invalid(
                    source,
                    "Invalid PNG chunk order, header, or trailing bytes",
                ));
            }
            name => {
                return Err(unsupported(
                    source,
                    format!(
                        "PNG chunk {:?} is outside the initial unprofiled static image profile",
                        String::from_utf8_lossy(name)
                    ),
                ));
            }
        }
        pos = end;
    }
    let (width, height, channels) = header
        .filter(|_| ended)
        .ok_or_else(|| invalid(source, "PNG requires complete IHDR, IDAT and IEND chunks"))?;
    // png deliberately tolerates missing Adler32 and trailing compressed bytes.
    // Require exactly one complete zlib stream with exactly the filtered rows.
    let expected = (width as usize * channels + 1) * height as usize;
    let mut filtered = vec![0; expected + 1];
    let mut inflater = flate2::Decompress::new(true);
    let status = inflater
        .decompress(&idat, &mut filtered, flate2::FlushDecompress::Finish)
        .map_err(|_| invalid(source, "Invalid PNG zlib data or Adler32 checksum"))?;
    if status != flate2::Status::StreamEnd
        || inflater.total_in() != idat.len() as u64
        || inflater.total_out() != expected as u64
    {
        return Err(invalid(
            source,
            "PNG requires one complete zlib stream and exactly its declared pixel rows",
        ));
    }
    drop(filtered);
    drop(idat);
    let mut options = png::DecodeOptions::default();
    options.set_ignore_checksums(false);
    options.set_skip_ancillary_crc_failures(false);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(bytes), options);
    decoder.set_limits(png::Limits {
        bytes: MAX_RGBA_BYTES + MAX_AXIS as usize,
    });
    let mut reader = decoder
        .read_info()
        .map_err(|_| invalid(source, "PNG header or CRC validation failed"))?;
    let size = reader
        .output_buffer_size()
        .filter(|n| *n == width as usize * height as usize * channels)
        .ok_or_else(|| invalid(source, "PNG decoded dimensions disagree with its container"))?;
    let info = reader
        .next_frame(&mut vec![0; size])
        .map_err(|_| invalid(source, "PNG full pixel decode or CRC validation failed"))?;
    if info.width != width || info.height != height || info.buffer_size() != size {
        return Err(invalid(
            source,
            "PNG decoded dimensions disagree with its container",
        ));
    }
    reader
        .finish()
        .map_err(|_| invalid(source, "PNG final chunk or CRC validation failed"))?;
    Ok((width, height))
}

/// Inspect every marker, including metadata between progressive scans. Decoder
/// metadata accessors alone can hide broken ICC sequences or late EXIF records.
fn jpeg_container(bytes: &[u8], source: &str) -> Result<(u32, u32), Diagnostic> {
    let mut pos = 2usize;
    let mut frame = None;
    let mut scans = 0usize;
    let mut jfif = false;
    while pos < bytes.len() {
        if bytes[pos] != 0xff {
            return Err(invalid(source, "Expected a JPEG marker"));
        }
        while bytes.get(pos) == Some(&0xff) {
            pos += 1;
        }
        let marker = *bytes
            .get(pos)
            .ok_or_else(|| invalid(source, "Truncated JPEG marker"))?;
        pos += 1;
        if marker == 0xd9 {
            if pos == bytes.len() && scans > 0 {
                return frame.ok_or_else(|| invalid(source, "JPEG has no supported frame"));
            }
            return Err(invalid(
                source,
                "JPEG requires a complete scan and exact end marker",
            ));
        }
        if matches!(marker, 0x00 | 0x01 | 0xd0..=0xd8) {
            return Err(invalid(source, "Unexpected standalone JPEG marker"));
        }
        let size = bytes
            .get(pos..pos.saturating_add(2))
            .ok_or_else(|| invalid(source, "Truncated JPEG marker length"))?;
        let len = u16::from_be_bytes([size[0], size[1]]) as usize;
        let end = pos
            .checked_add(len)
            .filter(|end| len >= 2 && *end <= bytes.len())
            .ok_or_else(|| invalid(source, "Invalid JPEG segment length"))?;
        let data = &bytes[pos + 2..end];
        match marker {
            0xe0 => {
                if jfif
                    || frame.is_some()
                    || data.len() != 14
                    || &data[..5] != b"JFIF\0"
                    || !matches!(&data[5..7], [1, 1] | [1, 2])
                    || data[7] != 0
                    || data[8..10] != data[10..12]
                    || data[8..10] == [0, 0]
                    || data[12..] != [0, 0]
                {
                    return Err(unsupported(
                        source,
                        "Use a neutral JFIF header without density units or thumbnails",
                    ));
                }
                jfif = true;
            }
            0xc0 | 0xc2 => {
                if frame.is_some() || data.len() != 15 || data[0] != 8 || data[5] != 3 {
                    return Err(unsupported(
                        source,
                        "Use an 8-bit three-component baseline or progressive JPEG",
                    ));
                }
                let (width, height) = (
                    u16::from_be_bytes([data[3], data[4]]) as u32,
                    u16::from_be_bytes([data[1], data[2]]) as u32,
                );
                dimensions(width, height, source)?;
                frame = Some((width, height));
            }
            0xc4 | 0xdb | 0xdd => {} // Huffman/quantization/restart tables: full decoder validates payloads.
            0xda if frame.is_some() => scans += 1,
            0xda => return Err(invalid(source, "JPEG scan precedes its frame")),
            _ => {
                return Err(unsupported(
                    source,
                    format!(
                        "JPEG marker FF{marker:02X} is outside the initial unprofiled static image profile"
                    ),
                ));
            }
        }
        pos = end;
        if marker == 0xda {
            loop {
                let next = bytes
                    .get(pos)
                    .ok_or_else(|| invalid(source, "JPEG scan has no end marker"))?;
                if *next != 0xff {
                    pos += 1;
                    continue;
                }
                let start = pos;
                while bytes.get(pos) == Some(&0xff) {
                    pos += 1;
                }
                let next = *bytes
                    .get(pos)
                    .ok_or_else(|| invalid(source, "Truncated JPEG scan marker"))?;
                if next == 0 || (0xd0..=0xd7).contains(&next) {
                    pos += 1;
                } else {
                    pos = start;
                    break;
                }
            }
        }
    }
    Err(invalid(source, "JPEG is missing its end marker"))
}

fn decode_jpeg(bytes: &[u8], source: &str) -> Result<(u32, u32), Diagnostic> {
    let expected = jpeg_container(bytes, source)?;
    jpeg_validation::validate(bytes, source)?;
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder.set_max_decoding_buffer_size(MAX_RGBA_BYTES);
    decoder
        .read_info()
        .map_err(|_| invalid(source, "JPEG header decode failed"))?;
    let check = |info: Option<jpeg_decoder::ImageInfo>| -> Result<(), Diagnostic> {
        let info = info.ok_or_else(|| invalid(source, "JPEG decoder returned no dimensions"))?;
        if (u32::from(info.width), u32::from(info.height)) != expected
            || info.pixel_format != jpeg_decoder::PixelFormat::RGB24
        {
            return Err(invalid(
                source,
                "JPEG decoded format disagrees with its container",
            ));
        }
        Ok(())
    };
    check(decoder.info())?;
    let pixels = decoder
        .decode()
        .map_err(|_| invalid(source, "JPEG full pixel decode failed"))?;
    check(decoder.info())?;
    if pixels.len() != expected.0 as usize * expected.1 as usize * 3
        || decoder.exif_data().is_some()
        || decoder.xmp_data().is_some()
        || decoder.icc_profile().is_some()
    {
        return Err(invalid(
            source,
            "JPEG decoded data disagrees with its admitted container",
        ));
    }
    Ok(expected)
}

fn decode_webp(bytes: &[u8], source: &str) -> Result<(u32, u32), Diagnostic> {
    if bytes.len() < 20 || le32(&bytes[4..8]) as u64 + 8 != bytes.len() as u64 {
        return Err(invalid(
            source,
            "WebP RIFF length must match the complete supplied file",
        ));
    }
    let mut pos = 12usize;
    let mut payload = None;
    while pos < bytes.len() {
        let header = bytes
            .get(pos..pos.saturating_add(8))
            .ok_or_else(|| invalid(source, "Truncated WebP chunk header"))?;
        let len = le32(&header[4..]) as usize;
        let end = pos
            .checked_add(8)
            .and_then(|n| n.checked_add(len))
            .filter(|n| *n <= bytes.len())
            .ok_or_else(|| invalid(source, "Truncated WebP chunk payload"))?;
        let padded = end
            .checked_add(len & 1)
            .filter(|n| *n <= bytes.len())
            .ok_or_else(|| invalid(source, "Missing WebP chunk padding"))?;
        if len & 1 != 0 && bytes[end] != 0 {
            return Err(invalid(source, "WebP chunk padding must be zero"));
        }
        match &header[..4] {
            b"VP8 " | b"VP8L" if payload.is_none() => {
                payload = Some((&header[..4], &bytes[pos + 8..end]))
            }
            b"VP8 " | b"VP8L" => {
                return Err(invalid(
                    source,
                    "WebP requires exactly one still-image payload",
                ));
            }
            name => {
                return Err(unsupported(
                    source,
                    format!(
                        "WebP chunk {:?} is outside the initial unprofiled static image profile",
                        String::from_utf8_lossy(name)
                    ),
                ));
            }
        }
        pos = padded;
    }
    let (kind, data) = payload.ok_or_else(|| invalid(source, "WebP has no image payload"))?;
    let expected = if kind == b"VP8L" {
        if data.len() < 5 || data[0] != 0x2f || data[4] >> 5 != 0 {
            return Err(invalid(source, "Invalid lossless WebP header"));
        }
        let bits = le32(&data[1..5]);
        (1 + (bits & 0x3fff), 1 + ((bits >> 14) & 0x3fff))
    } else {
        if data.len() < 10
            || data[0] & 1 != 0
            || data[0] & 0x10 == 0
            || &data[3..6] != b"\x9d\x01\x2a"
        {
            return Err(invalid(source, "Invalid static lossy WebP frame header"));
        }
        let (width, height) = (
            u16::from_le_bytes([data[6], data[7]]),
            u16::from_le_bytes([data[8], data[9]]),
        );
        if width & 0xc000 != 0 || height & 0xc000 != 0 {
            return Err(unsupported(
                source,
                "Scaled VP8 pixel dimensions are not admitted",
            ));
        }
        (u32::from(width), u32::from(height))
    };
    dimensions(expected.0, expected.1, source)?;
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(bytes))
        .map_err(|_| invalid(source, "WebP header decode failed"))?;
    decoder.set_memory_limit(MAX_RGBA_BYTES);
    if decoder.dimensions() != expected || decoder.is_animated() {
        return Err(invalid(
            source,
            "WebP decoder disagrees with the admitted still-image container",
        ));
    }
    let size = decoder
        .output_buffer_size()
        .filter(|n| *n <= MAX_RGBA_BYTES)
        .ok_or_else(|| invalid(source, "Invalid decoded WebP size"))?;
    decoder
        .read_image(&mut vec![0; size])
        .map_err(|_| invalid(source, "WebP full pixel decode failed"))?;
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const OPAQUE: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/opaque.png");
    const ALPHA: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/alpha.png");
    const BASELINE: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/baseline.jpg");
    const PROGRESSIVE: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/progressive.jpg");
    const LOSSY: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/lossy.webp");
    const LOSSLESS: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/lossless.webp");
    const WEBP_ALPHA: &[u8] = include_bytes!("../fixtures/images/ordinary-r1/alpha.webp");
    fn map(bytes: &[u8]) -> AssetMap {
        [(
            "image".into(),
            Asset::Image {
                bytes: bytes.to_vec(),
            },
        )]
        .into()
    }
    fn error(bytes: &[u8]) -> Diagnostic {
        AssetTable::validate(&map(bytes)).unwrap_err()
    }

    #[test]
    fn seven_static_unprofiled_fixtures_decode_and_keep_exact_bytes() {
        for bytes in [
            OPAQUE,
            ALPHA,
            BASELINE,
            PROGRESSIVE,
            LOSSY,
            LOSSLESS,
            WEBP_ALPHA,
        ] {
            let table = AssetTable::validate(&map(bytes)).unwrap();
            assert_eq!(
                table.get("image").unwrap(),
                ImageMetadata {
                    asset_index: 0,
                    width: 96,
                    height: 64
                }
            );
            let records = table.global_records().unwrap();
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].kind, "ImageAsset");
            assert_eq!(records[1].kind, "FileAssetContents");
            assert!(matches!(records[0].get("width"), Some(Value::Float(96.))));
            assert!(matches!(records[0].get("height"), Some(Value::Float(64.))));
            assert!(
                matches!(records[1].get("bytes"), Some(Value::Bytes(actual)) if actual == bytes)
            );
        }
    }

    #[test]
    fn exact_bytes_intern_in_stable_name_order_with_unused_assets_validated() {
        let assets: AssetMap = [
            (
                "z".into(),
                Asset::Image {
                    bytes: OPAQUE.to_vec(),
                },
            ),
            (
                "a".into(),
                Asset::Image {
                    bytes: ALPHA.to_vec(),
                },
            ),
            (
                "b".into(),
                Asset::Image {
                    bytes: OPAQUE.to_vec(),
                },
            ),
        ]
        .into();
        let table = AssetTable::validate(&assets).unwrap();
        assert_eq!(table.get("a").unwrap().asset_index, 0);
        assert_eq!(table.get("b").unwrap().asset_index, 1);
        assert_eq!(table.get("b").unwrap(), table.get("z").unwrap());
        let records = table.global_records().unwrap();
        assert_eq!(records.len(), 4);
        assert!(matches!(records[2].get("assetId"), Some(Value::Uint(2))));
        let reversed = assets.into_iter().rev().collect();
        assert_eq!(
            crate::wire::encode(&records).unwrap(),
            crate::wire::encode(
                &AssetTable::validate(&reversed)
                    .unwrap()
                    .global_records()
                    .unwrap()
            )
            .unwrap()
        );
        let bad: AssetMap = [(
            "unused\"key".into(),
            Asset::Image {
                bytes: vec![1, 2, 3],
            },
        )]
        .into();
        assert_eq!(
            AssetTable::validate(&bad).unwrap_err().source,
            "assets[\"unused\\\"key\"]"
        );
        assert_eq!(table.get("missing").unwrap_err().code, "missing-image");
        assert_eq!(table.get("missing").unwrap_err().source, "src");
    }

    #[test]
    fn asset_values_are_strict_named_typed_objects() {
        let value = Asset::Image {
            bytes: vec![0, 128, 255],
        };
        let encoded = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<Asset>(&encoded).unwrap(), value);
        for bad in [
            r#"["image",[1,2]]"#,
            r#"null"#,
            r#"{"kind":"image","bytes":[1.0]}"#,
            r#"{"kind":"image","bytes":[256]}"#,
            r#"{"kind":"image","bytes":[-1]}"#,
            r#"{"kind":"image","bytes":"data"}"#,
            r#"{"kind":"font","bytes":[]}"#,
            r#"{"kind":"image","bytes":[],"extra":0}"#,
            r#"{"kind":"image","kind":"image","bytes":[]}"#,
            r#"{"kind":"image","bytes":[],"bytes":[]}"#,
            r#"{"kind":"image"}"#,
        ] {
            assert!(serde_json::from_str::<Asset>(bad).is_err(), "{bad}");
        }
        #[derive(Deserialize)]
        struct Container {
            #[serde(deserialize_with = "deserialize_asset_map")]
            assets: AssetMap,
        }
        let good: Container =
            serde_json::from_str(r#"{"assets":{"__proto__":{"kind":"image","bytes":[]}}}"#)
                .unwrap();
        assert!(good.assets.contains_key("__proto__"));
        for bad in [
            r#"{"assets":null}"#,
            r#"{"assets":[]}"#,
            r#"{"assets":{"a":{"kind":"image","bytes":[]},"a":{"kind":"image","bytes":[]}}}"#,
        ] {
            assert!(serde_json::from_str::<Container>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn supplied_limits_count_aliases_and_report_precise_keys_before_decode() {
        let aliases: AssetMap = (0..MAX_ASSETS)
            .map(|i| {
                (
                    format!("{i:03}"),
                    Asset::Image {
                        bytes: OPAQUE.to_vec(),
                    },
                )
            })
            .collect();
        assert_eq!(
            AssetTable::validate(&aliases)
                .unwrap()
                .global_records()
                .unwrap()
                .len(),
            2
        );
        let mut too_many = aliases;
        too_many.insert(
            "last".into(),
            Asset::Image {
                bytes: OPAQUE.to_vec(),
            },
        );
        assert_eq!(
            AssetTable::validate(&too_many).unwrap_err().code,
            "asset-limit"
        );
        let empty: AssetMap = [(
            "".into(),
            Asset::Image {
                bytes: OPAQUE.to_vec(),
            },
        )]
        .into();
        assert_eq!(
            AssetTable::validate(&empty).unwrap_err().code,
            "invalid-asset"
        );
        let oversized: AssetMap = [
            (
                "a".into(),
                Asset::Image {
                    bytes: vec![0; MAX_ENCODED_BYTES / 2],
                },
            ),
            (
                "b".into(),
                Asset::Image {
                    bytes: vec![0; MAX_ENCODED_BYTES / 2 + 1],
                },
            ),
        ]
        .into();
        let error = AssetTable::validate(&oversized).unwrap_err();
        assert_eq!(error.code, "asset-limit");
        assert_eq!(error.source, "assets[\"b\"]");
        assert!(
            AssetTable::validate(&AssetMap::new())
                .unwrap()
                .global_records()
                .unwrap()
                .is_empty()
        );
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }
    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = (data.len() as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&crc32(&bytes[4..]).to_be_bytes());
        bytes
    }
    fn png(idat: &[u8], extra: Option<(&[u8; 4], &[u8])>) -> Vec<u8> {
        let mut bytes = OPAQUE[..33].to_vec();
        bytes.extend(chunk(b"IDAT", idat));
        if let Some((kind, data)) = extra {
            bytes.extend(chunk(kind, data));
        }
        bytes.extend(chunk(b"IEND", &[]));
        bytes
    }
    fn idat() -> &'static [u8] {
        &OPAQUE[41..OPAQUE.len() - 16]
    }
    fn compress(bytes: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn png_rejects_all_chunk_crc_failures_and_incomplete_containers() {
        for pos in [29, OPAQUE.len() - 16, OPAQUE.len() - 4] {
            let mut bad = OPAQUE.to_vec();
            bad[pos] ^= 1;
            assert_eq!(error(&bad).code, "invalid-image", "CRC offset {pos}");
        }
        let mut trailing = OPAQUE.to_vec();
        trailing.push(0);
        assert_eq!(error(&trailing).code, "invalid-image");
        assert_eq!(error(&OPAQUE[..OPAQUE.len() - 12]).code, "invalid-image");
        let mut long_chunk = OPAQUE.to_vec();
        long_chunk[33..37].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(error(&long_chunk).code, "invalid-image");
    }

    #[test]
    fn png_rejects_late_metadata_even_with_valid_crcs() {
        for kind in [
            b"eXIf", b"iCCP", b"acTL", b"gAMA", b"pHYs", b"sRGB", b"tEXt", b"tRNS",
        ] {
            assert_eq!(
                error(&png(idat(), Some((kind, &[0; 8])))).code,
                "unsupported-image"
            );
        }
    }

    #[test]
    fn png_requires_exact_complete_zlib_and_pixel_rows() {
        for cut in 1..=4 {
            assert_eq!(
                error(&png(&idat()[..idat().len() - cut], None)).code,
                "invalid-image"
            );
        }
        let mut wrong_adler = idat().to_vec();
        *wrong_adler.last_mut().unwrap() ^= 1;
        assert_eq!(error(&png(&wrong_adler, None)).code, "invalid-image");
        for tail in [vec![0], compress(&[])] {
            let mut extra = idat().to_vec();
            extra.extend(tail);
            assert_eq!(error(&png(&extra, None)).code, "invalid-image");
        }
        let row_bytes = (96 * 3 + 1) * 64;
        for count in [row_bytes - 1, row_bytes + 1] {
            assert_eq!(
                error(&png(&compress(&vec![0; count]), None)).code,
                "invalid-image"
            );
        }
        let mut rows = vec![0; row_bytes];
        rows[0] = 5;
        assert_eq!(error(&png(&compress(&rows), None)).code, "invalid-image");
        assert!(AssetTable::validate(&map(&png(&compress(&vec![0; row_bytes]), None))).is_ok());
    }

    #[test]
    fn png_dimension_guards_precede_pixel_allocation() {
        for (width, height) in [
            (8193u32, 1u32),
            (1, 8193),
            (2048, 2049),
            (u32::MAX, u32::MAX),
        ] {
            let mut bytes = OPAQUE.to_vec();
            bytes[16..20].copy_from_slice(&width.to_be_bytes());
            bytes[20..24].copy_from_slice(&height.to_be_bytes());
            assert_eq!(error(&bytes).code, "asset-limit");
        }
        let mut zero = OPAQUE.to_vec();
        zero[16..20].fill(0);
        assert_eq!(error(&zero).code, "invalid-image");
    }

    #[test]
    fn valid_pngs_reach_axis_and_rgba_limits() {
        for (width, height) in [(8192, 1), (1, 8192), (2048, 2048)] {
            let mut bytes = Vec::new();
            {
                let mut encoder = png::Encoder::new(&mut bytes, width, height);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().unwrap();
                writer
                    .write_image_data(&vec![0; width as usize * height as usize * 4])
                    .unwrap();
                writer.finish().unwrap();
            }
            let image = AssetTable::validate(&map(&bytes))
                .unwrap()
                .get("image")
                .unwrap();
            assert_eq!((image.width, image.height), (width, height));
        }
    }

    fn jpeg_marker(marker: u8, data: &[u8]) -> Vec<u8> {
        let mut result = vec![0xff, marker];
        result.extend_from_slice(&((data.len() + 2) as u16).to_be_bytes());
        result.extend_from_slice(data);
        result
    }
    #[test]
    fn jpeg_inspects_late_metadata_and_requires_complete_frame_decode() {
        for fixture in [BASELINE, PROGRESSIVE] {
            for marker in [0xe1, 0xe2, 0xee, 0xfe] {
                let mut bad = fixture[..fixture.len() - 2].to_vec();
                bad.extend(jpeg_marker(marker, b"incomplete metadata"));
                bad.extend_from_slice(&[0xff, 0xd9]);
                assert_eq!(error(&bad).code, "unsupported-image");
            }
            assert_eq!(error(&fixture[..fixture.len() - 1]).code, "invalid-image");
            let mut trailing = fixture.to_vec();
            trailing.push(0);
            assert_eq!(error(&trailing).code, "invalid-image");
            let sos = fixture
                .windows(2)
                .position(|bytes| bytes == [0xff, 0xda])
                .unwrap();
            let header_end =
                sos + 2 + u16::from_be_bytes([fixture[sos + 2], fixture[sos + 3]]) as usize;
            let mut missing_pixels = fixture[..header_end].to_vec();
            missing_pixels.extend_from_slice(&[0xff, 0xd9]);
            assert_eq!(error(&missing_pixels).code, "invalid-image");
        }
    }

    fn riff(chunks: &[u8]) -> Vec<u8> {
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&((chunks.len() + 4) as u32).to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend_from_slice(chunks);
        out
    }
    fn webp_chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = kind.to_vec();
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(payload);
        if payload.len() & 1 != 0 {
            out.push(0);
        }
        out
    }
    #[test]
    fn webp_rejects_hidden_metadata_duplicates_padding_and_truncation() {
        for kind in [
            b"ICCP", b"EXIF", b"XMP ", b"ANIM", b"ANMF", b"VP8X", b"ALPH",
        ] {
            let mut chunks = LOSSLESS[12..].to_vec();
            chunks.extend(webp_chunk(kind, &[0; 10]));
            assert_eq!(error(&riff(&chunks)).code, "unsupported-image");
        }
        let mut doubled = LOSSLESS[12..].to_vec();
        doubled.extend_from_slice(&LOSSLESS[12..]);
        assert_eq!(error(&riff(&doubled)).code, "invalid-image");
        let mut wrong_pad = LOSSLESS.to_vec();
        *wrong_pad.last_mut().unwrap() = 1;
        assert_eq!(error(&wrong_pad).code, "invalid-image");
        let mut trailing = LOSSLESS.to_vec();
        trailing.push(0);
        assert_eq!(error(&trailing).code, "invalid-image");
        assert_eq!(error(&LOSSLESS[..LOSSLESS.len() - 1]).code, "invalid-image");
        let header_only = webp_chunk(b"VP8L", &LOSSLESS[20..25]);
        assert_eq!(error(&riff(&header_only)).code, "invalid-image");
    }
}
