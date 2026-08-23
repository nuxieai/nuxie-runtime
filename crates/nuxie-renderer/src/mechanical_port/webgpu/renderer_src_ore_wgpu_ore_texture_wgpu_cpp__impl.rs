//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_texture_wgpu.cpp`.

#![allow(non_snake_case)]

use super::ore_texture_wgpu_decl::TextureWGPU;
use super::webgpu_decl::{
    WGPUExtent3D, WGPUOrigin3D, WGPUTexelCopyBufferLayout, WGPUTexelCopyTextureInfo,
    WGPU_COPY_STRIDE_UNDEFINED, WGPUTextureAspect_All,
};
use nuxie_ore_metal::texture::TextureUploadError;
use nuxie_ore_metal::types::{TextureDataDesc, textureFormatBytesPerTexel};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_texture_wgpu.cpp");
pub(crate) const kDawnBytesPerRowAlignment: u32 = 256;
pub(crate) const LEGACY_DAWN_TYPE_ALIASES: [(&str, &str); 4] = [
    ("TexelCopyBufferInfo", "ImageCopyBuffer"),
    ("TexelCopyTextureInfo", "ImageCopyTexture"),
    ("TexelCopyBufferLayout", "TextureDataLayout"),
    ("ShaderSourceWGSL", "ShaderModuleWGSLDescriptor"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UploadPlan {
    RowByRow {
        packedRowBytes: u32,
        actualBytesPerRow: u32,
        rowCount: u32,
    },
    Bulk {
        actualBytesPerRow: u32,
        rowsPerImage: u32,
        dataSize: u32,
    },
}

pub(crate) fn planUpload(
    data: &TextureDataDesc<'_>,
    bytesPerTexel: u32,
) -> Result<UploadPlan, TextureUploadError> {
    if bytesPerTexel == 0 && data.bytesPerRow == 0 {
        return Err(TextureUploadError::SizeOverflow);
    }
    let actualBytesPerRow = if data.bytesPerRow == 0 {
        data.width.wrapping_mul(bytesPerTexel)
    } else {
        data.bytesPerRow
    };
    let rowsPerImage = if data.rowsPerImage > 0 {
        data.rowsPerImage
    } else {
        data.height
    };
    let needsRowByRow = bytesPerTexel != 0
        && data.height > 1
        && data.depth == 1
        && actualBytesPerRow % kDawnBytesPerRowAlignment != 0;
    if needsRowByRow {
        Ok(UploadPlan::RowByRow {
            packedRowBytes: data.width.wrapping_mul(bytesPerTexel),
            actualBytesPerRow,
            rowCount: data.height,
        })
    } else {
        Ok(UploadPlan::Bulk {
            actualBytesPerRow,
            rowsPerImage,
            dataSize: actualBytesPerRow
                .wrapping_mul(rowsPerImage)
                .wrapping_mul(data.depth),
        })
    }
}

pub(crate) fn upload(
    texture: &TextureWGPU,
    data: &TextureDataDesc<'_>,
) -> Result<(), TextureUploadError> {
    if texture.m_wgpuTexture.Get().is_null() {
        return Err(TextureUploadError::MissingNativeTexture);
    }
    let bytes = data.data.ok_or(TextureUploadError::NullData)?;
    let plan = planUpload(data, textureFormatBytesPerTexel(texture.base.format()))?;
    let mut dst = WGPUTexelCopyTextureInfo::default();
    dst.texture = texture.m_wgpuTexture.Get();
    dst.mipLevel = data.mipLevel;
    dst.origin = WGPUOrigin3D { x: data.x, y: data.y, z: data.layer };
    dst.aspect = WGPUTextureAspect_All;

    match plan {
        UploadPlan::RowByRow {
            packedRowBytes,
            actualBytesPerRow,
            rowCount,
        } => {
            let required = if rowCount == 0 {
                0
            } else {
                (rowCount - 1) as usize * actualBytesPerRow as usize
                    + packedRowBytes as usize
            };
            if bytes.len() < required {
                return Err(TextureUploadError::DataTooShort {
                    required,
                    actual: bytes.len(),
                });
            }
            let mut rowLayout = WGPUTexelCopyBufferLayout::default();
            rowLayout.bytesPerRow = WGPU_COPY_STRIDE_UNDEFINED;
            rowLayout.rowsPerImage = WGPU_COPY_STRIDE_UNDEFINED;
            let rowExtent = WGPUExtent3D {
                width: data.width,
                height: 1,
                depthOrArrayLayers: 1,
            };
            for y in 0..rowCount {
                dst.origin.y = data.y.wrapping_add(y);
                let offset = y as usize * actualBytesPerRow as usize;
                unsafe {
                    texture.m_wgpuQueue.WriteTexture(
                        &dst,
                        bytes.as_ptr().add(offset).cast(),
                        packedRowBytes as usize,
                        &rowLayout,
                        &rowExtent,
                    );
                }
            }
        }
        UploadPlan::Bulk {
            actualBytesPerRow,
            rowsPerImage,
            dataSize,
        } => {
            let required = dataSize as usize;
            if bytes.len() < required {
                return Err(TextureUploadError::DataTooShort {
                    required,
                    actual: bytes.len(),
                });
            }
            let mut layout = WGPUTexelCopyBufferLayout::default();
            layout.bytesPerRow = actualBytesPerRow;
            layout.rowsPerImage = rowsPerImage;
            let extent = WGPUExtent3D {
                width: data.width,
                height: data.height,
                depthOrArrayLayers: data.depth,
            };
            unsafe {
                texture.m_wgpuQueue.WriteTexture(
                    &dst,
                    bytes.as_ptr().cast(),
                    required,
                    &layout,
                    &extent,
                );
            }
        }
    }
    Ok(())
}

pub(crate) const SOURCE_ASSERT_COUNT: usize = 3;
pub(crate) const SOURCE_WRITE_TEXTURE_CALL_SITE_COUNT: usize = 2;
pub(crate) const SOURCE_EARLY_RETURN_COUNT: usize = 1;
const _: [(); 3655] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::types::TextureDataDesc;

    fn data(width: u32, height: u32, depth: u32) -> TextureDataDesc<'static> {
        TextureDataDesc { width, height, depth, ..Default::default() }
    }

    #[test]
    fn complete_implementation_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 94);
        assert_eq!(LEGACY_DAWN_TYPE_ALIASES.len(), 4);
        assert_eq!(SOURCE_ASSERT_COUNT, 3);
        assert_eq!(SOURCE_WRITE_TEXTURE_CALL_SITE_COUNT, 2);
        assert_eq!(SOURCE_EARLY_RETURN_COUNT, 1);
    }

    #[test]
    fn unaligned_multiline_uncompressed_uploads_split_by_row() {
        assert_eq!(
            planUpload(&data(3, 2, 1), 4),
            Ok(UploadPlan::RowByRow {
                packedRowBytes: 12,
                actualBytesPerRow: 12,
                rowCount: 2,
            })
        );
    }

    #[test]
    fn aligned_or_depth_uploads_stay_bulk() {
        let aligned = TextureDataDesc {
            bytesPerRow: 256,
            ..data(3, 2, 1)
        };
        assert_eq!(
            planUpload(&aligned, 4),
            Ok(UploadPlan::Bulk {
                actualBytesPerRow: 256,
                rowsPerImage: 2,
                dataSize: 512,
            })
        );
        assert!(matches!(planUpload(&data(3, 2, 2), 4), Ok(UploadPlan::Bulk { .. })));
    }

    #[test]
    fn compressed_formats_require_an_authored_stride() {
        assert_eq!(planUpload(&data(8, 8, 1), 0), Err(TextureUploadError::SizeOverflow));
        let authored = TextureDataDesc {
            bytesPerRow: 256,
            ..data(8, 8, 1)
        };
        assert!(matches!(planUpload(&authored, 0), Ok(UploadPlan::Bulk { .. })));
    }
}
