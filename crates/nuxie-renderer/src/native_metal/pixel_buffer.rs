//! Decoded video frames arrive from AVFoundation as `CVPixelBuffer`s backed by
//! IOSurfaces. Importing one wraps its IOSurface as a Metal texture in place,
//! so a frame reaches the renderer without a CPU copy.

use std::ffi::c_void;
use std::ptr::NonNull;

use nuxie_render_api::RenderImage;
use objc2::encode::{Encoding, RefEncode};
use objc2::ffi::{objc_setAssociatedObject, OBJC_ASSOCIATION_RETAIN_NONATOMIC};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_metal::{MTLPixelFormat, MTLTexture, MTLTextureDescriptor, MTLTextureUsage};

use super::NativeMetalFactory;
use crate::RendererError;

/// `kCVPixelFormatType_32BGRA`: the format AVFoundation converts decoded video
/// to on request, and the one whose bytes match the RGBA upload path.
const PIXEL_FORMAT_32BGRA: u32 = u32::from_be_bytes(*b"BGRA");

/// The pointee of `IOSurfaceRef`, encoded the way Metal's selector declares it.
#[repr(C)]
struct IOSurface {
    _private: [u8; 0],
}

unsafe impl RefEncode for IOSurface {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("__IOSurface", &[]));
}

#[link(name = "CoreVideo", kind = "framework")]
extern "C" {
    fn CVPixelBufferGetPixelFormatType(pixel_buffer: *const c_void) -> u32;
    fn CVPixelBufferGetWidth(pixel_buffer: *const c_void) -> usize;
    fn CVPixelBufferGetHeight(pixel_buffer: *const c_void) -> usize;
    fn CVPixelBufferGetIOSurface(pixel_buffer: *const c_void) -> *mut IOSurface;
}

/// Its address is the key under which a texture holds its pixel buffer.
static PIXEL_BUFFER_KEY: u8 = 0;

impl NativeMetalFactory {
    /// Wraps a decoded frame as a renderer image without copying its pixels:
    /// the image samples the pixel buffer's IOSurface directly.
    ///
    /// The buffer must be 32BGRA and IOSurface-backed, which AVFoundation
    /// provides to outputs that request `kCVPixelBufferMetalCompatibilityKey`.
    /// Its bytes are read the way `upload_rgba8_premul_srgb` reads RGBA:
    /// gamma-encoded, alpha treated as premultiplied, top row first.
    ///
    /// The texture retains the buffer, so the decoder's pool cannot reuse the
    /// surface while the image, or GPU work sampling it, still needs it. The
    /// caller may release its own reference once this returns.
    ///
    /// # Safety
    /// `pixel_buffer` must be a valid `CVPixelBufferRef` for this call.
    pub unsafe fn import_pixel_buffer(
        &self,
        pixel_buffer: NonNull<c_void>,
    ) -> Result<Box<dyn RenderImage>, RendererError> {
        let buffer = pixel_buffer.as_ptr().cast_const();
        let format = unsafe { CVPixelBufferGetPixelFormatType(buffer) };
        if format != PIXEL_FORMAT_32BGRA {
            return Err(RendererError::InvalidImageUpload(format!(
                "pixel buffer format {format:#010x} is not 32BGRA"
            )));
        }
        let surface =
            NonNull::new(unsafe { CVPixelBufferGetIOSurface(buffer) }).ok_or_else(|| {
                RendererError::InvalidImageUpload(
                    "pixel buffer is not backed by an IOSurface".into(),
                )
            })?;
        let extent = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
        let width = extent(unsafe { CVPixelBufferGetWidth(buffer) });
        let height = extent(unsafe { CVPixelBufferGetHeight(buffer) });
        let max_dimension = self.source_capabilities().max_texture_size;
        if width == 0 || height == 0 || width > max_dimension || height > max_dimension {
            return Err(RendererError::InvalidTextureExtent {
                label: "pixel buffer",
                width,
                height,
                max_dimension,
            });
        }
        // BGRA8Unorm samples exactly what the RGBA path uploads: the same
        // gamma-encoded bytes, put in order by the format instead of the CPU.
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                MTLPixelFormat::BGRA8Unorm,
                width as usize,
                height as usize,
                false,
            )
        };
        descriptor.setUsage(MTLTextureUsage::ShaderRead);
        let texture: Option<Retained<ProtocolObject<dyn MTLTexture>>> = unsafe {
            msg_send![
                &*self.device,
                newTextureWithDescriptor: &*descriptor,
                iosurface: surface.as_ptr(),
                plane: 0usize
            ]
        };
        let texture = texture.ok_or_else(|| {
            RendererError::NativeMetal("Metal could not wrap the pixel buffer's IOSurface".into())
        })?;
        // Metal keeps a texture alive until the command buffers sampling it
        // complete, so holding the buffer on the texture, not the image,
        // covers GPU work still in flight when the image is released.
        unsafe {
            objc_setAssociatedObject(
                Retained::as_ptr(&texture).cast_mut().cast::<AnyObject>(),
                std::ptr::addr_of!(PIXEL_BUFFER_KEY).cast(),
                pixel_buffer.as_ptr().cast::<AnyObject>(),
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            );
        }
        self.adopt_metal_image_texture(texture, width, height)
            .ok_or_else(|| {
                RendererError::NativeMetal("the pixel buffer texture could not be adopted".into())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_metal::{NativeMetalContextOptions, ShaderCompilationMode};
    use nuxie_render_api::{BlendMode, ImageSampler, Renderer};

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFTypeDictionaryKeyCallBacks: [usize; 6];
        static kCFTypeDictionaryValueCallBacks: [usize; 5];
        static kCFBooleanTrue: *const c_void;
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            count: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFGetRetainCount(object: *const c_void) -> isize;
        fn CFRelease(object: *const c_void);
    }

    #[link(name = "CoreVideo", kind = "framework")]
    extern "C" {
        static kCVPixelBufferIOSurfacePropertiesKey: *const c_void;
        static kCVPixelBufferMetalCompatibilityKey: *const c_void;
        fn CVPixelBufferCreate(
            allocator: *const c_void,
            width: usize,
            height: usize,
            format: u32,
            attributes: *const c_void,
            out: *mut *mut c_void,
        ) -> i32;
        fn CVPixelBufferLockBaseAddress(pixel_buffer: *mut c_void, flags: u64) -> i32;
        fn CVPixelBufferUnlockBaseAddress(pixel_buffer: *mut c_void, flags: u64) -> i32;
        fn CVPixelBufferGetBaseAddress(pixel_buffer: *mut c_void) -> *mut u8;
        fn CVPixelBufferGetBytesPerRow(pixel_buffer: *mut c_void) -> usize;
    }

    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    /// Owns one pixel buffer reference.
    struct PixelBuffer(NonNull<c_void>);

    impl PixelBuffer {
        fn new(format: u32, surface_backed: bool) -> Self {
            unsafe {
                let empty = CFDictionaryCreate(
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    std::ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
                    std::ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
                );
                let (keys, values) = if surface_backed {
                    (
                        vec![
                            kCVPixelBufferIOSurfacePropertiesKey,
                            kCVPixelBufferMetalCompatibilityKey,
                        ],
                        vec![empty, kCFBooleanTrue],
                    )
                } else {
                    (Vec::new(), Vec::new())
                };
                let attributes = CFDictionaryCreate(
                    std::ptr::null(),
                    keys.as_ptr(),
                    values.as_ptr(),
                    keys.len() as isize,
                    std::ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
                    std::ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
                );
                let mut buffer = std::ptr::null_mut();
                let status = CVPixelBufferCreate(
                    std::ptr::null(),
                    WIDTH,
                    HEIGHT,
                    format,
                    attributes,
                    &mut buffer,
                );
                CFRelease(attributes);
                CFRelease(empty);
                assert_eq!(status, 0, "CVPixelBufferCreate");
                Self(NonNull::new(buffer).expect("pixel buffer"))
            }
        }

        /// Writes `rgba` (tightly packed) into the buffer as BGRA rows.
        fn fill_from_rgba(&self, rgba: &[u8]) {
            unsafe {
                assert_eq!(CVPixelBufferLockBaseAddress(self.0.as_ptr(), 0), 0);
                let base = CVPixelBufferGetBaseAddress(self.0.as_ptr());
                let stride = CVPixelBufferGetBytesPerRow(self.0.as_ptr());
                for (y, row) in rgba.chunks_exact(WIDTH * 4).enumerate() {
                    let target = std::slice::from_raw_parts_mut(base.add(y * stride), WIDTH * 4);
                    for (to, from) in target.chunks_exact_mut(4).zip(row.chunks_exact(4)) {
                        to.copy_from_slice(&[from[2], from[1], from[0], from[3]]);
                    }
                }
                assert_eq!(CVPixelBufferUnlockBaseAddress(self.0.as_ptr(), 0), 0);
            }
        }

        fn retain_count(&self) -> isize {
            unsafe { CFGetRetainCount(self.0.as_ptr()) }
        }
    }

    impl Drop for PixelBuffer {
        fn drop(&mut self) {
            unsafe { CFRelease(self.0.as_ptr()) };
        }
    }

    /// Premultiplied RGBA whose four quadrants and alpha ramp would all show
    /// a flipped, mirrored, swizzled or re-premultiplied import.
    fn asymmetric_rgba() -> Vec<u8> {
        let mut rgba = Vec::with_capacity(WIDTH * HEIGHT * 4);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let alpha = 16 + (x * 238 / (WIDTH - 1)) as u32;
                let [r, g, b] = match (x < WIDTH / 2, y < HEIGHT / 2) {
                    (true, true) => [255, 32, 0],
                    (false, true) => [0, 200, 64],
                    (true, false) => [40, 90, 255],
                    (false, false) => [128, 128, 128],
                };
                let premultiply = |channel: u32| ((channel * alpha + 127) / 255) as u8;
                rgba.extend([premultiply(r), premultiply(g), premultiply(b), alpha as u8]);
            }
        }
        rgba
    }

    fn render(factory: &NativeMetalFactory, image: &dyn RenderImage) -> Vec<u8> {
        let mut frame = factory.begin_frame(0xff20_2020).expect("begin frame");
        frame.draw_image(
            Some(image),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            1.0,
        );
        frame.finish().expect("render and read back")
    }

    #[test]
    fn imported_pixel_buffer_draws_the_same_pixels_as_the_rgba_upload() {
        let rgba = asymmetric_rgba();
        let buffer = PixelBuffer::new(PIXEL_FORMAT_32BGRA, true);
        buffer.fill_from_rgba(&rgba);
        // Each mode fixes the shader, so both images draw through the same one.
        for mode in [
            ShaderCompilationMode::AlwaysSynchronous,
            ShaderCompilationMode::OnlyUbershaders,
        ] {
            let factory = NativeMetalFactory::new_with_context_options(
                WIDTH as u32,
                HEIGHT as u32,
                NativeMetalContextOptions {
                    shader_compilation_mode: mode,
                    ..Default::default()
                },
            )
            .expect("live Metal factory");
            let uploaded = factory
                .upload_rgba8_premul_srgb(WIDTH as u32, HEIGHT as u32, (WIDTH * 4) as u32, &rgba)
                .expect("RGBA upload");
            let imported =
                unsafe { factory.import_pixel_buffer(buffer.0) }.expect("pixel buffer import");
            assert_eq!(
                (imported.width(), imported.height()),
                (WIDTH as u32, HEIGHT as u32)
            );

            let expected = render(&factory, uploaded.as_ref());
            let actual = render(&factory, imported.as_ref());
            assert!(
                expected
                    .chunks_exact(4)
                    .any(|pixel| pixel != [32, 32, 32, 255]),
                "{mode:?}: the reference frame must show the image, not only the clear color"
            );
            let differing = expected
                .chunks_exact(4)
                .zip(actual.chunks_exact(4))
                .filter(|(expected, actual)| expected != actual)
                .count();
            assert_eq!(
                differing, 0,
                "{mode:?}: imported frame differs from the RGBA upload"
            );
        }
    }

    #[test]
    fn an_imported_image_holds_its_pixel_buffer_until_released() {
        let factory =
            NativeMetalFactory::new(WIDTH as u32, HEIGHT as u32).expect("live Metal factory");
        let buffer = PixelBuffer::new(PIXEL_FORMAT_32BGRA, true);
        let before = buffer.retain_count();
        let image = unsafe { factory.import_pixel_buffer(buffer.0) }.expect("pixel buffer import");
        assert_eq!(
            buffer.retain_count(),
            before + 1,
            "the texture retains the buffer"
        );
        drop(image);
        assert_eq!(
            buffer.retain_count(),
            before,
            "releasing the image releases the buffer"
        );
    }

    #[test]
    fn other_formats_and_memory_backed_buffers_are_rejected() {
        let factory =
            NativeMetalFactory::new(WIDTH as u32, HEIGHT as u32).expect("live Metal factory");
        let biplanar = PixelBuffer::new(u32::from_be_bytes(*b"420f"), true);
        assert!(matches!(
            unsafe { factory.import_pixel_buffer(biplanar.0) },
            Err(RendererError::InvalidImageUpload(_))
        ));
        let memory = PixelBuffer::new(PIXEL_FORMAT_32BGRA, false);
        assert!(matches!(
            unsafe { factory.import_pixel_buffer(memory.0) },
            Err(RendererError::InvalidImageUpload(_))
        ));
    }
}
