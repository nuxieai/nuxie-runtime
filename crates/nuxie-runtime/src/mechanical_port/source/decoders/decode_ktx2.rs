use super::{
    astc_footprints::ASTC_FOOTPRINTS, bitmap_decoder::PixelFormat, texture_decoder::decode_texture,
};
use crate::mechanical_port::source::gpu_texture_format::GpuTextureFormat;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ktx2HwSupport {
    pub bc: bool,
    pub astc: bool,
    pub etc2: bool,
}
impl Default for Ktx2HwSupport {
    fn default() -> Self {
        Self {
            bc: true,
            astc: true,
            etc2: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ktx2DecodeResult {
    pub format: GpuTextureFormat,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub level_count: u32,
    pub block_width: u8,
    pub block_height: u8,
    pub srgb: bool,
    pub blocks: Vec<u8>,
    pub software_decoded: bool,
}

impl Default for Ktx2DecodeResult {
    fn default() -> Self {
        Self {
            format: GpuTextureFormat::Rgba32,
            pixel_width: 0,
            pixel_height: 0,
            level_count: 0,
            block_width: 4,
            block_height: 4,
            srgb: false,
            blocks: Vec::new(),
            software_decoded: false,
        }
    }
}

const KTX2_IDENTIFIER: [u8; 12] = [
    0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a,
];
const HEADER_SIZE: usize = 68;
const LEVEL_INDEX_SIZE: usize = 24;
const MAX_DIMENSION: u32 = 16384;
const MAX_LEVELS: u32 = 16;

struct Ktx2Header {
    vk_format: u32,
    pixel_width: u32,
    pixel_height: u32,
    layer_count: u32,
    face_count: u32,
    level_count: u32,
    supercompression_scheme: u32,
}

struct Ktx2LevelIndex {
    byte_offset: u64,
    byte_length: u64,
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn expected_block_bytes(
    width: u32,
    height: u32,
    block_width: u32,
    block_height: u32,
    bytes_per_block: u32,
) -> u64 {
    let blocks_x = (width + block_width - 1) / block_width;
    let blocks_y = (height + block_height - 1) / block_height;
    u64::from(blocks_x) * u64::from(blocks_y) * u64::from(bytes_per_block)
}

pub fn decode_ktx2(bytes: &[u8], out: &mut Ktx2DecodeResult, hw_support: Ktx2HwSupport) -> bool {
    if bytes.len() < KTX2_IDENTIFIER.len() + HEADER_SIZE {
        eprintln!("DecodeKtx2: file too small");
        return false;
    }
    if bytes[..KTX2_IDENTIFIER.len()] != KTX2_IDENTIFIER {
        eprintln!("DecodeKtx2: bad magic");
        return false;
    }
    // The packed header is little-endian. Fields not inspected by the pinned
    // reader (typeSize, depth, DFD/KVD/SGD offsets) remain deliberately unused.
    let header_bytes = &bytes[KTX2_IDENTIFIER.len()..KTX2_IDENTIFIER.len() + HEADER_SIZE];
    let header = Ktx2Header {
        vk_format: read_u32(header_bytes, 0),
        pixel_width: read_u32(header_bytes, 8),
        pixel_height: read_u32(header_bytes, 12),
        layer_count: read_u32(header_bytes, 20),
        face_count: read_u32(header_bytes, 24),
        level_count: read_u32(header_bytes, 28),
        supercompression_scheme: read_u32(header_bytes, 32),
    };
    let (out_format, block_width, block_height, bytes_per_block, srgb) = match header.vk_format {
        145 | 146 => (
            GpuTextureFormat::Bc7,
            4_u8,
            4_u8,
            16,
            header.vk_format == 146,
        ),
        151 | 152 => (GpuTextureFormat::Etc2, 4, 4, 16, header.vk_format == 152),
        157..=184 => {
            let footprint = ASTC_FOOTPRINTS[((header.vk_format - 157) / 2) as usize];
            (
                GpuTextureFormat::Astc,
                footprint.width,
                footprint.height,
                16,
                header.vk_format % 2 == 0,
            )
        }
        _ => {
            eprintln!("DecodeKtx2: unsupported vkFormat {}", header.vk_format);
            return false;
        }
    };
    if header.supercompression_scheme != 0 {
        eprintln!(
            "DecodeKtx2: supercompressionScheme {} not supported",
            header.supercompression_scheme
        );
        return false;
    }
    if header.face_count != 1 || header.layer_count != 0 {
        eprintln!(
            "DecodeKtx2: cubemaps/arrays not supported (faces={} layers={})",
            header.face_count, header.layer_count
        );
        return false;
    }
    if header.pixel_width == 0
        || header.pixel_width > MAX_DIMENSION
        || header.pixel_height == 0
        || header.pixel_height > MAX_DIMENSION
    {
        eprintln!(
            "DecodeKtx2: dimensions out of range ({}x{})",
            header.pixel_width, header.pixel_height
        );
        return false;
    }
    let level_count = if header.level_count == 0 {
        1
    } else {
        header.level_count
    };
    if level_count > MAX_LEVELS {
        eprintln!("DecodeKtx2: levelCount {level_count} exceeds cap {MAX_LEVELS}");
        return false;
    }
    let level_index_offset = KTX2_IDENTIFIER.len() + HEADER_SIZE;
    let level_index_bytes = level_count as usize * LEVEL_INDEX_SIZE;
    if bytes.len() < level_index_offset + level_index_bytes {
        eprintln!("DecodeKtx2: truncated level index");
        return false;
    }
    let entries: Vec<_> = (0..level_count as usize)
        .map(|index| {
            let offset = level_index_offset + index * LEVEL_INDEX_SIZE;
            Ktx2LevelIndex {
                byte_offset: read_u64(bytes, offset),
                byte_length: read_u64(bytes, offset + 8),
                // The pinned reader does not validate uncompressedByteLength.
            }
        })
        .collect();
    let mut total_bytes = 0_u64;
    for (index, entry) in entries.iter().enumerate() {
        let log_w = 1.max(header.pixel_width >> index);
        let log_h = 1.max(header.pixel_height >> index);
        let expected = expected_block_bytes(
            log_w,
            log_h,
            u32::from(block_width),
            u32::from(block_height),
            bytes_per_block,
        );
        if entry.byte_length != expected {
            eprintln!(
                "DecodeKtx2: level {index} byteLength {} != block grid {expected} for {log_w}x{log_h}",
                entry.byte_length
            );
            return false;
        }
        if entry.byte_offset > bytes.len() as u64
            || entry.byte_length > bytes.len() as u64 - entry.byte_offset
        {
            eprintln!("DecodeKtx2: level {index} out of buffer");
            return false;
        }
        total_bytes += entry.byte_length;
    }
    out.format = out_format;
    out.pixel_width = header.pixel_width;
    out.pixel_height = header.pixel_height;
    out.level_count = level_count;
    out.block_width = block_width;
    out.block_height = block_height;
    out.srgb = srgb;
    out.software_decoded = false;
    out.blocks.resize(total_bytes as usize, 0);
    let mut write_offset = 0;
    for entry in &entries {
        let start = entry.byte_offset as usize;
        let len = entry.byte_length as usize;
        out.blocks[write_offset..write_offset + len].copy_from_slice(&bytes[start..start + len]);
        write_offset += len;
    }
    let need_fallback = match out.format {
        GpuTextureFormat::Bc1
        | GpuTextureFormat::Bc2
        | GpuTextureFormat::Bc3
        | GpuTextureFormat::Bc7 => !hw_support.bc,
        GpuTextureFormat::Astc => !hw_support.astc,
        GpuTextureFormat::Etc2 => !hw_support.etc2,
        _ => false,
    };
    if need_fallback {
        let mut total_rgba = 0;
        for index in 0..level_count {
            let log_w = 1.max(out.pixel_width >> index);
            let log_h = 1.max(out.pixel_height >> index);
            total_rgba += log_w as usize * log_h as usize * 4;
        }
        let mut decoded = Vec::with_capacity(total_rgba);
        let mut src_offset = 0;
        for (index, entry) in entries.iter().enumerate() {
            let log_w = 1.max(out.pixel_width >> index);
            let log_h = 1.max(out.pixel_height >> index);
            let level_bytes = entry.byte_length as usize;
            let Some(mut bitmap) = decode_texture(
                &out.blocks[src_offset..src_offset + level_bytes],
                log_w,
                log_h,
                out.format,
                u32::from(out.block_width),
                u32::from(out.block_height),
            ) else {
                eprintln!(
                    "DecodeKtx2: HW lacks support for format {} and software decoder unavailable (level {index})",
                    out.format as u8
                );
                return false;
            };
            bitmap.set_pixel_format(PixelFormat::RgbaPremul);
            decoded.extend_from_slice(bitmap.bytes());
            src_offset += level_bytes;
        }
        out.blocks = decoded;
        out.format = GpuTextureFormat::Rgba32;
        out.block_width = 1;
        out.block_height = 1;
        out.srgb = false;
        out.software_decoded = true;
    }
    true
}
