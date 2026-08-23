//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_texture_gl.cpp` for `ORE_BACKEND_GL`.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::ore_texture_gl_decl::{TextureGL, TextureViewGL};
use nuxie_ore_metal::texture::TextureUploadError;
use nuxie_ore_metal::types::{textureFormatBytesPerTexel, TextureDataDesc, TextureFormat};
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_texture_gl.cpp");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompressedFormatConfiguration {
    pub(crate) s3tcConstantsDefined: bool,
    pub(crate) bptcConstantDefined: bool,
}

impl CompressedFormatConfiguration {
    // The pinned RIVE_WEBGL include path is GLES3/gl3.h plus
    // webgl/webgl2_ext.h. Emscripten 3.1.61 defines neither extension token in
    // those headers, so the source selects both RIVE_UNREACHABLE branches.
    pub(crate) const FROZEN_WEBGL2: Self = Self {
        s3tcConstantsDefined: false,
        bptcConstantDefined: false,
    };
}

pub(crate) fn oreFormatToGLInternalWithConfiguration(
    format: TextureFormat,
    configuration: CompressedFormatConfiguration,
) -> GLenum {
    match format {
        TextureFormat::r8unorm => GL_R8,
        TextureFormat::rg8unorm => GL_RG8,
        TextureFormat::rgba8unorm => GL_RGBA8,
        TextureFormat::rgba8snorm => GL_RGBA8_SNORM,
        // GLES3 has no BGRA8 internal format; the source deliberately uses
        // RGBA8 and requires upload callers to pre-swizzle.
        TextureFormat::bgra8unorm => GL_RGBA8,
        TextureFormat::rgba16float => GL_RGBA16F,
        TextureFormat::rg16float => GL_RG16F,
        TextureFormat::r16float => GL_R16F,
        TextureFormat::rgba32float => GL_RGBA32F,
        TextureFormat::rg32float => GL_RG32F,
        TextureFormat::r32float => GL_R32F,
        TextureFormat::rgb10a2unorm => GL_RGB10_A2,
        TextureFormat::r11g11b10float => GL_R11F_G11F_B10F,
        TextureFormat::depth16unorm => GL_DEPTH_COMPONENT16,
        TextureFormat::depth24plusStencil8 => GL_DEPTH24_STENCIL8,
        TextureFormat::depth32float => GL_DEPTH_COMPONENT32F,
        TextureFormat::depth32floatStencil8 => GL_DEPTH32F_STENCIL8,
        TextureFormat::bc1unorm if configuration.s3tcConstantsDefined => {
            GL_COMPRESSED_RGB_S3TC_DXT1_EXT
        }
        TextureFormat::bc3unorm if configuration.s3tcConstantsDefined => {
            GL_COMPRESSED_RGBA_S3TC_DXT5_EXT
        }
        TextureFormat::bc7unorm if configuration.bptcConstantDefined => {
            GL_COMPRESSED_RGBA_BPTC_UNORM
        }
        TextureFormat::bc1unorm | TextureFormat::bc3unorm | TextureFormat::bc7unorm => {
            panic!("compressed GL constant is absent in this source configuration")
        }
        TextureFormat::etc2rgb8 => GL_COMPRESSED_RGB8_ETC2,
        TextureFormat::etc2rgba8 => GL_COMPRESSED_RGBA8_ETC2_EAC,
        TextureFormat::astc4x4 => GL_COMPRESSED_RGBA_ASTC_4x4_KHR,
        TextureFormat::astc6x6 => GL_COMPRESSED_RGBA_ASTC_6x6_KHR,
        TextureFormat::astc8x8 => GL_COMPRESSED_RGBA_ASTC_8x8_KHR,
    }
}

pub(crate) fn oreFormatToGLInternal(format: TextureFormat) -> GLenum {
    oreFormatToGLInternalWithConfiguration(format, CompressedFormatConfiguration::FROZEN_WEBGL2)
}

pub(crate) fn oreFormatToGLFormat(format: TextureFormat) -> GLenum {
    match format {
        TextureFormat::r8unorm | TextureFormat::r16float | TextureFormat::r32float => GL_RED,
        TextureFormat::rg8unorm | TextureFormat::rg16float | TextureFormat::rg32float => GL_RG,
        TextureFormat::rgba8unorm
        | TextureFormat::rgba8snorm
        | TextureFormat::bgra8unorm
        | TextureFormat::rgba16float
        | TextureFormat::rgba32float
        | TextureFormat::rgb10a2unorm => GL_RGBA,
        TextureFormat::r11g11b10float => GL_RGB,
        TextureFormat::depth16unorm | TextureFormat::depth32float => GL_DEPTH_COMPONENT,
        TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8 => {
            GL_DEPTH_STENCIL
        }
        _ => panic!("compressed texture formats have no uncompressed GL format"),
    }
}

pub(crate) fn oreFormatToGLType(format: TextureFormat) -> GLenum {
    match format {
        TextureFormat::r8unorm
        | TextureFormat::rg8unorm
        | TextureFormat::rgba8unorm
        | TextureFormat::bgra8unorm => GL_UNSIGNED_BYTE,
        TextureFormat::rgba8snorm => GL_BYTE,
        TextureFormat::rgba16float | TextureFormat::rg16float | TextureFormat::r16float => {
            GL_HALF_FLOAT
        }
        TextureFormat::rgba32float
        | TextureFormat::rg32float
        | TextureFormat::r32float
        | TextureFormat::depth32float => GL_FLOAT,
        TextureFormat::rgb10a2unorm => GL_UNSIGNED_INT_2_10_10_10_REV,
        TextureFormat::r11g11b10float => GL_UNSIGNED_INT_10F_11F_11F_REV,
        TextureFormat::depth16unorm => GL_UNSIGNED_SHORT,
        TextureFormat::depth24plusStencil8 => GL_UNSIGNED_INT_24_8,
        TextureFormat::depth32floatStencil8 => GL_FLOAT_32_UNSIGNED_INT_24_8_REV,
        _ => panic!("compressed texture formats have no uncompressed GL type"),
    }
}

pub(crate) const fn isCompressedFormat(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::bc1unorm
            | TextureFormat::bc3unorm
            | TextureFormat::bc7unorm
            | TextureFormat::etc2rgb8
            | TextureFormat::etc2rgba8
            | TextureFormat::astc4x4
            | TextureFormat::astc6x6
            | TextureFormat::astc8x8
    )
}

fn sourceBytes<'a>(data: &TextureDataDesc<'a>) -> Result<&'a [u8], TextureUploadError> {
    data.data.ok_or(TextureUploadError::NullData)
}

pub(crate) fn upload(
    texture: &TextureGL,
    data: &TextureDataDesc<'_>,
) -> Result<(), TextureUploadError> {
    assert!(texture.m_glTexture != 0);
    let bytes = sourceBytes(data)?;
    assert!(
        texture.base.format() != TextureFormat::bgra8unorm,
        "GLES3 cannot upload BGRA pixels — pre-swizzle to rgba8unorm or use a different format"
    );

    recordGLCommand(GLCommand::ActiveTexture(GL_TEXTURE0));
    recordGLCommand(GLCommand::BindTexture(
        texture.m_glTarget,
        texture.m_glTexture,
    ));

    let internalFormat = oreFormatToGLInternal(texture.base.format());
    if isCompressedFormat(texture.base.format()) {
        let imageSize = data
            .bytesPerRow
            .wrapping_mul(if data.rowsPerImage > 0 {
                data.rowsPerImage
            } else {
                data.height
            })
            .wrapping_mul(if data.depth > 0 { data.depth } else { 1 });
        let image = bytes
            .get(..imageSize as usize)
            .ok_or(TextureUploadError::DataTooShort {
                required: imageSize as usize,
                actual: bytes.len(),
            })?
            .to_vec();
        if texture.m_glTarget == GL_TEXTURE_3D || texture.m_glTarget == GL_TEXTURE_2D_ARRAY {
            recordGLCommand(GLCommand::CompressedTexSubImage3D {
                target: texture.m_glTarget,
                level: data.mipLevel,
                x: data.x,
                y: data.y,
                z: data.layer,
                width: data.width,
                height: data.height,
                depth: data.depth,
                format: internalFormat,
                data: image,
            });
        } else {
            let target = if texture.m_glTarget == GL_TEXTURE_CUBE_MAP {
                GL_TEXTURE_CUBE_MAP_POSITIVE_X + data.layer
            } else {
                texture.m_glTarget
            };
            recordGLCommand(GLCommand::CompressedTexSubImage2D {
                target,
                level: data.mipLevel,
                x: data.x,
                y: data.y,
                width: data.width,
                height: data.height,
                format: internalFormat,
                data: image,
            });
        }
        recordGLCommand(GLCommand::BindTexture(texture.m_glTarget, 0));
        return Ok(());
    }

    let format = oreFormatToGLFormat(texture.base.format());
    let type_ = oreFormatToGLType(texture.base.format());
    let savedRowLength = allocateGLQuerySlot();
    let savedImageHeight = allocateGLQuerySlot();
    recordGLCommand(GLCommand::GetInteger(GL_UNPACK_ROW_LENGTH, savedRowLength));
    recordGLCommand(GLCommand::GetInteger(
        GL_UNPACK_IMAGE_HEIGHT,
        savedImageHeight,
    ));
    let bytesPerTexel = textureFormatBytesPerTexel(texture.base.format());
    if data.bytesPerRow != 0 && bytesPerTexel != 0 && data.bytesPerRow % bytesPerTexel == 0 {
        recordGLCommand(GLCommand::PixelStore(
            GL_UNPACK_ROW_LENGTH,
            (data.bytesPerRow / bytesPerTexel) as i32,
        ));
    }
    if data.rowsPerImage != 0 {
        recordGLCommand(GLCommand::PixelStore(
            GL_UNPACK_IMAGE_HEIGHT,
            data.rowsPerImage as i32,
        ));
    }

    if texture.m_glTarget == GL_TEXTURE_3D || texture.m_glTarget == GL_TEXTURE_2D_ARRAY {
        recordGLCommand(GLCommand::TexSubImage3D {
            target: texture.m_glTarget,
            level: data.mipLevel,
            x: data.x,
            y: data.y,
            z: data.layer,
            width: data.width,
            height: data.height,
            depth: data.depth,
            format,
            type_,
            data: bytes.to_vec(),
        });
    } else {
        let target = if texture.m_glTarget == GL_TEXTURE_CUBE_MAP {
            GL_TEXTURE_CUBE_MAP_POSITIVE_X + data.layer
        } else {
            texture.m_glTarget
        };
        recordGLCommand(GLCommand::TexSubImage2D {
            target,
            level: data.mipLevel,
            x: data.x,
            y: data.y,
            width: data.width,
            height: data.height,
            format,
            type_,
            data: bytes.to_vec(),
        });
    }
    recordGLCommand(GLCommand::PixelStoreFromQuery(
        GL_UNPACK_ROW_LENGTH,
        savedRowLength,
    ));
    recordGLCommand(GLCommand::PixelStoreFromQuery(
        GL_UNPACK_IMAGE_HEIGHT,
        savedImageHeight,
    ));
    recordGLCommand(GLCommand::BindTexture(texture.m_glTarget, 0));
    Ok(())
}

impl Drop for TextureGL {
    fn drop(&mut self) {
        if self.m_glRenderbuffer != 0 && self.m_glOwnsTexture {
            recordGLCommand(GLCommand::DeleteRenderbuffer(self.m_glRenderbuffer));
        }
        if self.m_glTexture != 0 && self.m_glOwnsTexture {
            recordGLCommand(GLCommand::DeleteTexture(self.m_glTexture));
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

impl Drop for TextureViewGL {
    fn drop(&mut self) {
        if self.m_glTextureView != 0 {
            recordGLCommand(GLCommand::DeleteTexture(self.m_glTextureView));
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::types::TextureDesc;

    #[test]
    fn complete_implementation_denominator_is_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 349);
    }

    #[test]
    fn all_25_texture_formats_follow_the_frozen_preprocessor_configuration() {
        let supported_formats = [
            TextureFormat::r8unorm,
            TextureFormat::rg8unorm,
            TextureFormat::rgba8unorm,
            TextureFormat::rgba8snorm,
            TextureFormat::bgra8unorm,
            TextureFormat::rgba16float,
            TextureFormat::rg16float,
            TextureFormat::r16float,
            TextureFormat::rgba32float,
            TextureFormat::rg32float,
            TextureFormat::r32float,
            TextureFormat::rgb10a2unorm,
            TextureFormat::r11g11b10float,
            TextureFormat::depth16unorm,
            TextureFormat::depth24plusStencil8,
            TextureFormat::depth32float,
            TextureFormat::depth32floatStencil8,
            TextureFormat::etc2rgb8,
            TextureFormat::etc2rgba8,
            TextureFormat::astc4x4,
            TextureFormat::astc6x6,
            TextureFormat::astc8x8,
        ];
        assert_eq!(supported_formats.len(), 22);
        for format in supported_formats {
            assert_ne!(oreFormatToGLInternal(format), 0);
        }
        for format in [
            TextureFormat::bc1unorm,
            TextureFormat::bc3unorm,
            TextureFormat::bc7unorm,
        ] {
            assert!(std::panic::catch_unwind(|| oreFormatToGLInternal(format)).is_err());
        }

        let constants_defined = CompressedFormatConfiguration {
            s3tcConstantsDefined: true,
            bptcConstantDefined: true,
        };
        assert_eq!(
            oreFormatToGLInternalWithConfiguration(TextureFormat::bc1unorm, constants_defined),
            GL_COMPRESSED_RGB_S3TC_DXT1_EXT
        );
        assert_eq!(
            oreFormatToGLInternalWithConfiguration(TextureFormat::bc3unorm, constants_defined),
            GL_COMPRESSED_RGBA_S3TC_DXT5_EXT
        );
        assert_eq!(
            oreFormatToGLInternalWithConfiguration(TextureFormat::bc7unorm, constants_defined),
            GL_COMPRESSED_RGBA_BPTC_UNORM
        );
    }

    #[test]
    fn owned_renderbuffer_then_texture_are_deleted_before_the_base() {
        resetGLCommandStream();
        let mut texture = TextureGL::new(&TextureDesc::default());
        texture.m_glTexture = 31;
        texture.m_glRenderbuffer = 32;
        texture.m_glOwnsTexture = true;
        drop(texture);
        assert_eq!(
            takeGLCommands(),
            vec![
                GLCommand::DeleteRenderbuffer(32),
                GLCommand::DeleteTexture(31),
            ]
        );
    }
}
