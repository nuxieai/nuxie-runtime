//! Native Metal image-texture leaf.
//!
//! Mechanical translation of the pinned upstream declaration and
//! implementation in
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:139-153`
//! and
//! `renderer/src/metal/render_context_metal_impl.mm:830-984` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! Intent-preserving divergence: pinned C++ computes ASTC pixel formats as a
//! base value plus footprint index, but Metal reserves enum value 209. This
//! port uses an explicit 14-format table so every requested footprint keeps
//! its intended Metal format across that gap.

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;

use nuxie_render_api::RenderImage;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLBlitCommandEncoder, MTLCommandBuffer, MTLCommandEncoder, MTLDevice, MTLOrigin,
    MTLPixelFormat, MTLRegion, MTLSize, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

use crate::RendererError;

/// The GPU-uploadable formats realized by the pinned Metal leaf.
///
/// The upstream `GPUTextureFormat` also has BC1/BC2/BC3 variants, but the
/// pinned Metal switch handles only these four formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeMetalTextureFormat {
    Rgba32,
    Bc7,
    Astc,
    Etc2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextureFormatPolicy {
    pixel_format: MTLPixelFormat,
    bytes_per_block: usize,
    compressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MipLevelLayout {
    pub(crate) level: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) blocks_x: usize,
    pub(crate) blocks_y: usize,
    pub(crate) bytes_per_row: usize,
    pub(crate) offset: usize,
    pub(crate) size: usize,
}

#[cfg(test)]
const ASTC_LDR_BASE: usize = 204;
const ASTC_FOOTPRINTS: [(u8, u8); 14] = [
    (4, 4),
    (5, 4),
    (5, 5),
    (6, 5),
    (6, 6),
    (8, 5),
    (8, 6),
    (8, 8),
    (10, 5),
    (10, 6),
    (10, 8),
    (10, 10),
    (12, 10),
    (12, 12),
];

const ASTC_LDR_PIXEL_FORMATS: [MTLPixelFormat; 14] = [
    MTLPixelFormat::ASTC_4x4_LDR,
    MTLPixelFormat::ASTC_5x4_LDR,
    MTLPixelFormat::ASTC_5x5_LDR,
    MTLPixelFormat::ASTC_6x5_LDR,
    MTLPixelFormat::ASTC_6x6_LDR,
    MTLPixelFormat::ASTC_8x5_LDR,
    MTLPixelFormat::ASTC_8x6_LDR,
    MTLPixelFormat::ASTC_8x8_LDR,
    MTLPixelFormat::ASTC_10x5_LDR,
    MTLPixelFormat::ASTC_10x6_LDR,
    MTLPixelFormat::ASTC_10x8_LDR,
    MTLPixelFormat::ASTC_10x10_LDR,
    MTLPixelFormat::ASTC_12x10_LDR,
    MTLPixelFormat::ASTC_12x12_LDR,
];

/// Return the canonical ASTC footprint index used by the upstream decoder.
pub(crate) fn astc_footprint_index(block_width: u8, block_height: u8) -> Option<usize> {
    ASTC_FOOTPRINTS
        .iter()
        .position(|&(width, height)| width == block_width && height == block_height)
}

fn format_policy(
    format: NativeMetalTextureFormat,
    block_width: u8,
    block_height: u8,
) -> Result<TextureFormatPolicy, RendererError> {
    match format {
        NativeMetalTextureFormat::Rgba32 => {
            if block_width != 1 || block_height != 1 {
                return Err(RendererError::NativeMetal(
                    "rgba32 textures require a 1x1 block footprint".to_owned(),
                ));
            }
            Ok(TextureFormatPolicy {
                pixel_format: MTLPixelFormat::RGBA8Unorm,
                bytes_per_block: 4,
                compressed: false,
            })
        }
        NativeMetalTextureFormat::Bc7 => {
            #[cfg(target_os = "ios")]
            {
                let _ = (block_width, block_height);
                Err(RendererError::NativeMetal(
                    "bc7 textures are unavailable on iOS Metal".to_owned(),
                ))
            }
            #[cfg(not(target_os = "ios"))]
            {
                Ok(TextureFormatPolicy {
                    pixel_format: MTLPixelFormat::BC7_RGBAUnorm,
                    bytes_per_block: 16,
                    compressed: true,
                })
            }
        }
        NativeMetalTextureFormat::Astc => {
            let index = astc_footprint_index(block_width, block_height).ok_or_else(|| {
                RendererError::NativeMetal(format!(
                    "unsupported ASTC block footprint {block_width}x{block_height}"
                ))
            })?;
            Ok(TextureFormatPolicy {
                pixel_format: ASTC_LDR_PIXEL_FORMATS[index],
                bytes_per_block: 16,
                compressed: true,
            })
        }
        NativeMetalTextureFormat::Etc2 => Ok(TextureFormatPolicy {
            pixel_format: MTLPixelFormat::EAC_RGBA8,
            bytes_per_block: 16,
            compressed: true,
        }),
    }
}

fn level_dimension(dimension: u32, level: u32) -> u32 {
    dimension.checked_shr(level).unwrap_or(0).max(1)
}

/// Compute tight, largest-first mip offsets using block-rounded dimensions.
///
/// This is the layout implied by upstream lines 859-877: each level's row
/// pitch is `ceil(width / blockWidth) * bytesPerBlock`, and no inter-level
/// padding is inserted.
pub(crate) fn mip_layout(
    width: u32,
    height: u32,
    mip_level_count: u32,
    block_width: u8,
    block_height: u8,
    bytes_per_block: usize,
) -> Result<Vec<MipLevelLayout>, RendererError> {
    if width == 0 || height == 0 || mip_level_count == 0 {
        return Err(RendererError::NativeMetal(
            "image texture dimensions and mip count must be nonzero".to_owned(),
        ));
    }
    if block_width == 0 || block_height == 0 || bytes_per_block == 0 {
        return Err(RendererError::NativeMetal(
            "image texture block dimensions and byte size must be nonzero".to_owned(),
        ));
    }

    let block_width = usize::from(block_width);
    let block_height = usize::from(block_height);
    let mut offset = 0usize;
    let mut levels = Vec::with_capacity(mip_level_count as usize);
    for level in 0..mip_level_count {
        let level_width = level_dimension(width, level);
        let level_height = level_dimension(height, level);
        let blocks_x = (level_width as usize)
            .checked_add(block_width - 1)
            .ok_or_else(|| RendererError::NativeMetal("image mip block count overflow".into()))?
            / block_width;
        let blocks_y = (level_height as usize)
            .checked_add(block_height - 1)
            .ok_or_else(|| RendererError::NativeMetal("image mip block count overflow".into()))?
            / block_height;
        let bytes_per_row = blocks_x
            .checked_mul(bytes_per_block)
            .ok_or_else(|| RendererError::NativeMetal("image mip row size overflow".to_owned()))?;
        let size = bytes_per_row
            .checked_mul(blocks_y)
            .ok_or_else(|| RendererError::NativeMetal("image mip size overflow".to_owned()))?;
        levels.push(MipLevelLayout {
            level,
            width: level_width,
            height: level_height,
            blocks_x,
            blocks_y,
            bytes_per_row,
            offset,
            size,
        });
        offset = offset
            .checked_add(size)
            .ok_or_else(|| RendererError::NativeMetal("image mip offset overflow".to_owned()))?;
    }
    Ok(levels)
}

/// Retained wrapper for a shader-readable Metal texture.
pub(crate) struct NativeMetalImageTexture {
    width: u32,
    height: u32,
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    mips_dirty: Cell<bool>,
}

impl NativeMetalImageTexture {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        width: u32,
        height: u32,
        mip_level_count: u32,
        format: NativeMetalTextureFormat,
        image_data: &[u8],
        block_width: u8,
        block_height: u8,
        srgb: bool,
        generate_remaining_mips: bool,
    ) -> Result<Self, RendererError> {
        let _ = srgb; // Pinned upstream marks `srgb` maybe_unused.
        let policy = format_policy(format, block_width, block_height)?;
        if generate_remaining_mips && policy.compressed {
            return Err(RendererError::NativeMetal(
                "Metal mip generation is undefined for compressed textures".to_owned(),
            ));
        }
        let layouts = mip_layout(
            width,
            height,
            mip_level_count,
            block_width,
            block_height,
            policy.bytes_per_block,
        )?;
        let levels_to_upload = if generate_remaining_mips {
            1
        } else {
            mip_level_count
        };
        let required_bytes = layouts
            .get(levels_to_upload as usize - 1)
            .and_then(|level| level.offset.checked_add(level.size))
            .ok_or_else(|| RendererError::NativeMetal("invalid mip upload range".to_owned()))?;
        if image_data.len() < required_bytes {
            return Err(RendererError::NativeMetal(format!(
                "image data is too short for mip upload ({} < {required_bytes})",
                image_data.len()
            )));
        }

        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(policy.pixel_format);
        // SAFETY: dimensions and mip count were checked nonzero and widen
        // losslessly to NSUInteger; Metal validates the format combination.
        unsafe {
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setMipmapLevelCount(mip_level_count as usize);
        }
        descriptor.setUsage(MTLTextureUsage::ShaderRead);
        descriptor.setTextureType(MTLTextureType::Type2D);
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| RendererError::NativeMetal("failed to allocate image texture".into()))?;

        for layout in layouts.iter().take(levels_to_upload as usize) {
            let bytes = &image_data[layout.offset..layout.offset + layout.size];
            let pointer = NonNull::new(bytes.as_ptr() as *mut c_void).ok_or_else(|| {
                RendererError::NativeMetal("image mip upload has no data pointer".to_owned())
            })?;
            let region = MTLRegion {
                origin: MTLOrigin { x: 0, y: 0, z: 0 },
                size: MTLSize {
                    width: layout.width as usize,
                    height: layout.height as usize,
                    depth: 1,
                },
            };
            // SAFETY: `bytes` is a live slice with at least `bytes_per_row`
            // bytes per row and the region matches this mip's dimensions.
            unsafe {
                texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                    region,
                    layout.level as usize,
                    pointer,
                    layout.bytes_per_row,
                );
            }
        }

        Ok(Self {
            width,
            height,
            texture,
            mips_dirty: Cell::new(generate_remaining_mips && mip_level_count > 1),
        })
    }

    /// Adopt an externally-created texture without allocating or uploading.
    /// The retained handle keeps the texture alive for all future samples.
    pub(crate) fn adopt(
        texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
        width: u32,
        height: u32,
    ) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            texture: texture?,
            mips_dirty: Cell::new(false),
        })
    }

    /// Queue one mip-generation blit, exactly once, when requested at create.
    pub(crate) fn ensure_mipmaps(
        &self,
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    ) -> Result<(), RendererError> {
        if !self.mips_dirty.get() {
            return Ok(());
        }
        let encoder = command_buffer.blitCommandEncoder().ok_or_else(|| {
            RendererError::NativeMetal("failed to create mip blit encoder".into())
        })?;
        encoder.generateMipmapsForTexture(&self.texture);
        encoder.endEncoding();
        self.mips_dirty.set(false);
        Ok(())
    }

    pub(crate) fn texture(&self) -> &ProtocolObject<dyn MTLTexture> {
        &self.texture
    }

    pub(crate) fn mips_dirty(&self) -> bool {
        self.mips_dirty.get()
    }
}

impl RenderImage for NativeMetalImageTexture {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astc_footprints_match_pinned_canonical_order() {
        let expected = [
            (4, 4),
            (5, 4),
            (5, 5),
            (6, 5),
            (6, 6),
            (8, 5),
            (8, 6),
            (8, 8),
            (10, 5),
            (10, 6),
            (10, 8),
            (10, 10),
            (12, 10),
            (12, 12),
        ];
        for (index, footprint) in expected.into_iter().enumerate() {
            assert_eq!(astc_footprint_index(footprint.0, footprint.1), Some(index));
        }
        assert_eq!(astc_footprint_index(4, 5), None);
        assert_eq!(astc_footprint_index(8, 4), None);
    }

    #[test]
    fn astc_footprints_use_their_exact_metal_pixel_formats_across_enum_gap() {
        for (index, &(block_width, block_height)) in ASTC_FOOTPRINTS.iter().enumerate() {
            let policy =
                format_policy(NativeMetalTextureFormat::Astc, block_width, block_height).unwrap();
            assert_eq!(policy.pixel_format, ASTC_LDR_PIXEL_FORMATS[index]);
        }
        assert_eq!(ASTC_LDR_PIXEL_FORMATS[4], MTLPixelFormat(ASTC_LDR_BASE + 4));
        assert_eq!(ASTC_LDR_PIXEL_FORMATS[5], MTLPixelFormat(ASTC_LDR_BASE + 6));
        assert_ne!(ASTC_LDR_PIXEL_FORMATS[5], MTLPixelFormat(ASTC_LDR_BASE + 5));
    }

    #[test]
    fn format_policy_preserves_rgba8_bc7_astc_and_etc2_mapping() {
        let rgba = format_policy(NativeMetalTextureFormat::Rgba32, 1, 1).unwrap();
        assert_eq!(rgba.pixel_format, MTLPixelFormat::RGBA8Unorm);
        assert_eq!(rgba.bytes_per_block, 4);
        assert!(!rgba.compressed);

        let astc = format_policy(NativeMetalTextureFormat::Astc, 8, 8).unwrap();
        assert_eq!(astc.pixel_format, MTLPixelFormat::ASTC_8x8_LDR);
        assert_eq!(astc.bytes_per_block, 16);
        assert!(astc.compressed);

        let etc2 = format_policy(NativeMetalTextureFormat::Etc2, 4, 4).unwrap();
        assert_eq!(etc2.pixel_format, MTLPixelFormat::EAC_RGBA8);
        assert_eq!(etc2.bytes_per_block, 16);
        assert!(etc2.compressed);

        #[cfg(not(target_os = "ios"))]
        {
            let bc7 = format_policy(NativeMetalTextureFormat::Bc7, 4, 4).unwrap();
            assert_eq!(bc7.pixel_format, MTLPixelFormat::BC7_RGBAUnorm);
            assert_eq!(bc7.bytes_per_block, 16);
            assert!(bc7.compressed);
        }
        #[cfg(target_os = "ios")]
        assert!(format_policy(NativeMetalTextureFormat::Bc7, 4, 4).is_err());
    }

    #[test]
    fn rgba32_rejects_non_unit_blocks_and_compressed_mip_generation() {
        assert!(format_policy(NativeMetalTextureFormat::Rgba32, 2, 1).is_err());
        assert!(format_policy(NativeMetalTextureFormat::Rgba32, 1, 2).is_err());
        // The pure policy has enough information to enforce the upstream
        // assertion that compressed mip generation is undefined.
        for format in [
            NativeMetalTextureFormat::Bc7,
            NativeMetalTextureFormat::Astc,
            NativeMetalTextureFormat::Etc2,
        ] {
            assert!(format_policy(format, 4, 4).unwrap().compressed);
        }
    }

    #[test]
    fn mip_layout_rounds_each_level_to_compression_blocks() {
        let levels = mip_layout(9, 7, 4, 4, 4, 16).unwrap();
        assert_eq!(
            levels,
            vec![
                MipLevelLayout {
                    level: 0,
                    width: 9,
                    height: 7,
                    blocks_x: 3,
                    blocks_y: 2,
                    bytes_per_row: 48,
                    offset: 0,
                    size: 96,
                },
                MipLevelLayout {
                    level: 1,
                    width: 4,
                    height: 3,
                    blocks_x: 1,
                    blocks_y: 1,
                    bytes_per_row: 16,
                    offset: 96,
                    size: 16,
                },
                MipLevelLayout {
                    level: 2,
                    width: 2,
                    height: 1,
                    blocks_x: 1,
                    blocks_y: 1,
                    bytes_per_row: 16,
                    offset: 112,
                    size: 16,
                },
                MipLevelLayout {
                    level: 3,
                    width: 1,
                    height: 1,
                    blocks_x: 1,
                    blocks_y: 1,
                    bytes_per_row: 16,
                    offset: 128,
                    size: 16,
                },
            ]
        );
    }

    #[test]
    fn rgba_upload_layout_is_tight_and_noncompressed() {
        let levels = mip_layout(3, 2, 3, 1, 1, 4).unwrap();
        assert_eq!(levels[0].bytes_per_row, 12);
        assert_eq!(levels[0].size, 24);
        assert_eq!(levels[1].offset, 24);
        assert_eq!(levels[1].size, 4);
        assert_eq!(levels[2].offset, 28);
        assert_eq!(levels[2].size, 4);
    }

    #[test]
    fn mip_layout_rejects_invalid_inputs_and_overflows() {
        for input in [(0, 1, 1, 1, 1, 4), (1, 0, 1, 1, 1, 4), (1, 1, 0, 1, 1, 4)] {
            assert!(mip_layout(input.0, input.1, input.2, input.3, input.4, input.5).is_err());
        }
        assert!(mip_layout(1, 1, 1, 0, 1, 4).is_err());
        assert!(mip_layout(u32::MAX, u32::MAX, 1, 1, 1, usize::MAX).is_err());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_rgba_upload_generates_remaining_mips_exactly_once() {
        use objc2_metal::MTLCommandQueue;

        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let image = NativeMetalImageTexture::new(
            &device,
            4,
            4,
            3,
            NativeMetalTextureFormat::Rgba32,
            &[0x7f; 4 * 4 * 4],
            1,
            1,
            false,
            true,
        )
        .unwrap();
        assert!(image.mips_dirty());
        assert_eq!(image.texture().mipmapLevelCount(), 3);

        let queue = device.newCommandQueue().unwrap();
        let command_buffer = queue.commandBuffer().unwrap();
        image.ensure_mipmaps(&command_buffer).unwrap();
        assert!(!image.mips_dirty());
        image.ensure_mipmaps(&command_buffer).unwrap();
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_adopted_texture_retains_identity_and_dimensions() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        // SAFETY: The test supplies nonzero, NSUInteger-representable 2x2
        // dimensions and one mip level, matching the validated descriptor
        // invariants used by the production constructor.
        unsafe {
            descriptor.setWidth(2);
            descriptor.setHeight(2);
            descriptor.setMipmapLevelCount(1);
        }
        descriptor.setUsage(MTLTextureUsage::ShaderRead);
        descriptor.setTextureType(MTLTextureType::Type2D);
        let texture = device.newTextureWithDescriptor(&descriptor).unwrap();
        let raw = texture.as_ref() as *const ProtocolObject<dyn MTLTexture>;
        let image = NativeMetalImageTexture::adopt(Some(texture), 2, 2).unwrap();
        assert_eq!((image.width(), image.height()), (2, 2));
        assert!(std::ptr::eq(raw, image.texture() as *const _));
        assert!(!image.mips_dirty());
    }
}
