//! Shared public-seam adapter for the mechanically translated render contexts.
//!
//! Native device discovery, targets, submission, and readback remain owned by
//! each concrete backend. This module only adapts the pinned generic
//! `RenderContext -> RiveRenderFactory -> RiveRenderer` chain to
//! `nuxie-render-api`.

use std::any::Any;
use std::cell::RefCell;
#[cfg(feature = "rive-decoders")]
use std::io::Cursor;
use std::pin::Pin;
use std::rc::Rc;

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, RawPath,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader, Renderer,
};

use crate::mechanical_port::source::include::rive::renderer_hpp::RendererContract;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::RenderContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::{
    RenderResourceDomain, RiveRenderBufferHandle,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_renderer_hpp::RiveRenderer;
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaintHandle;
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPathHandle;
use crate::{RenderMode, RendererError};

#[cfg(feature = "rive-decoders")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    BitmapDecodeResult, BitmapDecoderContract, BitmapPixelFormat,
};

#[cfg(feature = "rive-decoders")]
struct ExactBitmapDecoder;

#[cfg(feature = "rive-decoders")]
fn decode_source_png(encoded: &[u8]) -> Option<BitmapDecodeResult> {
    let mut decoder = png::Decoder::new(Cursor::new(encoded));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut decoded = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut decoded).ok()?;
    decoded.truncate(info.buffer_size());
    let mut bytes = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => decoded,
        (png::ColorType::Rgb, png::BitDepth::Eight) => decoded
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        (png::ColorType::Grayscale, png::BitDepth::Eight) => decoded
            .into_iter()
            .flat_map(|gray| [gray, gray, gray, 255])
            .collect(),
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => decoded
            .chunks_exact(2)
            .flat_map(|gray_alpha| [gray_alpha[0], gray_alpha[0], gray_alpha[0], gray_alpha[1]])
            .collect(),
        _ => return None,
    };
    for pixel in bytes.chunks_exact_mut(4) {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel[..3] {
            *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
    Some(BitmapDecodeResult {
        width: info.width,
        height: info.height,
        pixel_format: BitmapPixelFormat::rgbaPremul,
        bytes,
    })
}

#[cfg(feature = "rive-decoders")]
impl BitmapDecoderContract for ExactBitmapDecoder {
    fn decodeBitmap(&mut self, encoded: &[u8]) -> Option<BitmapDecodeResult> {
        if encoded.starts_with(b"\x89PNG\r\n\x1a\n") {
            return decode_source_png(encoded);
        }
        let dimensions = nuxie_image_codec::preflight_encoded_image(encoded)?;
        let decoded = nuxie_image_codec::decode_image_rgba(encoded)?;
        if decoded.width != dimensions.width || decoded.height != dimensions.height {
            return None;
        }
        Some(BitmapDecodeResult {
            width: decoded.width,
            height: decoded.height,
            pixel_format: BitmapPixelFormat::rgbaPremul,
            bytes: decoded.pixels,
        })
    }

    fn convertToRGBAPremul(&mut self, bitmap: &mut BitmapDecodeResult) {
        bitmap.pixel_format = BitmapPixelFormat::rgbaPremul;
    }
}

#[cfg(feature = "rive-decoders")]
pub(crate) fn install_bitmap_decoder(mut context: Pin<&mut RenderContext>) {
    unsafe { Pin::get_unchecked_mut(context.as_mut()) }
        .installBitmapDecoder(Box::new(ExactBitmapDecoder));
}

pub(crate) trait ExactSourceBackend: 'static {
    fn context_mut(&mut self) -> Pin<&mut RenderContext>;
    fn begin_frame(&mut self, clear_color: u32, mode: RenderMode) -> Result<u64, RendererError>;
    fn finish_frame(&mut self, frame_number: u64) -> Result<Vec<u8>, RendererError>;
    fn abort_frame(&mut self);
}

pub(crate) struct ExactSourceFactoryCore<B: ExactSourceBackend> {
    backend: Rc<RefCell<B>>,
    resource_domain: RenderResourceDomain,
}

impl<B: ExactSourceBackend> ExactSourceFactoryCore<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self {
            backend: Rc::new(RefCell::new(backend)),
            resource_domain: RenderResourceDomain::new(),
        }
    }

    pub(crate) fn begin_frame(
        &self,
        clear_color: u32,
        mode: RenderMode,
    ) -> Result<ExactSourceFrameCore<B>, RendererError> {
        let (renderer, frame_number) = {
            let mut backend = self.backend.borrow_mut();
            let frame_number = backend.begin_frame(clear_color, mode)?;
            let context = unsafe { Pin::get_unchecked_mut(backend.context_mut()) };
            (
                unsafe { RiveRenderer::new_from_context(context) },
                frame_number,
            )
        };
        Ok(ExactSourceFrameCore {
            renderer,
            backend: Rc::clone(&self.backend),
            resource_domain: self.resource_domain.clone(),
            frame_number,
            finished: false,
        })
    }

    fn execution_anchor(&self) -> Rc<dyn Any> {
        Rc::clone(&self.backend) as Rc<dyn Any>
    }

    fn with_context<R>(&self, callback: impl FnOnce(&mut RenderContext) -> R) -> R {
        let mut backend = self.backend.borrow_mut();
        let context = unsafe { Pin::get_unchecked_mut(backend.context_mut()) };
        callback(context)
    }
}

impl<B: ExactSourceBackend> Factory for ExactSourceFactoryCore<B> {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let source = self.with_context(|context| {
            context.makeRenderBufferHandle(
                source_buffer_type(buffer_type),
                source_buffer_flags(flags),
                size_in_bytes,
            )
        });
        let source = source
            .unwrap_or_else(|| panic!("exact source render-buffer owner was not valid"))
            .with_execution_domain(self.resource_domain.clone(), self.execution_anchor());
        Box::new(source)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(
            self.with_context(|context| {
                context
                    .riveRenderFactoryMut()
                    .makeLinearGradientHandle(sx, sy, ex, ey, colors, stops)
            })
            .unwrap_or_else(|| panic!("exact source linear-gradient owner was not valid")),
        )
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(
            self.with_context(|context| {
                context
                    .riveRenderFactoryMut()
                    .makeRadialGradientHandle(cx, cy, radius, colors, stops)
            })
            .unwrap_or_else(|| panic!("exact source radial-gradient owner was not valid")),
        )
    }

    fn make_render_path(&mut self, mut path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(
            self.with_context(|context| {
                context
                    .riveRenderFactoryMut()
                    .makeRenderPathHandle(&mut path, fill_rule)
            })
            .unwrap_or_else(|| panic!("exact source render-path owner was not valid")),
        )
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(
            self.with_context(|context| context.riveRenderFactoryMut().makeEmptyRenderPathHandle())
                .unwrap_or_else(|| panic!("exact source empty-path owner was not valid")),
        )
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(
            self.with_context(|context| context.riveRenderFactoryMut().makeRenderPaintHandle())
                .unwrap_or_else(|| panic!("exact source paint owner was not valid")),
        )
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.with_context(|context| context.decodeImageHandle(data))
            .map(|image| {
                image.with_execution_domain(self.resource_domain.clone(), self.execution_anchor())
            })
            .map(|image| Box::new(image) as Box<dyn RenderImage>)
            .ok_or(ImageDecodeError)
    }
}

pub(crate) struct ExactSourceFrameCore<B: ExactSourceBackend> {
    renderer: RiveRenderer,
    backend: Rc<RefCell<B>>,
    resource_domain: RenderResourceDomain,
    frame_number: u64,
    finished: bool,
}

impl<B: ExactSourceBackend> ExactSourceFrameCore<B> {
    pub(crate) fn finish(mut self) -> Result<Vec<u8>, RendererError> {
        let result = self.backend.borrow_mut().finish_frame(self.frame_number);
        self.finished = true;
        result
    }
}

impl<B: ExactSourceBackend> Drop for ExactSourceFrameCore<B> {
    fn drop(&mut self) {
        if !self.finished {
            self.backend.borrow_mut().abort_frame();
        }
    }
}

impl<B: ExactSourceBackend> Renderer for ExactSourceFrameCore<B> {
    fn save(&mut self) {
        <RiveRenderer as RendererContract>::save(&mut self.renderer);
    }

    fn restore(&mut self) {
        <RiveRenderer as RendererContract>::restore(&mut self.renderer);
    }

    fn transform(&mut self, transform: Mat2D) {
        <RiveRenderer as RendererContract>::transform(&mut self.renderer, &transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let Some(path) = path.as_any().downcast_ref::<RiveRenderPathHandle>() else {
            return;
        };
        let Some(paint) = paint.as_any().downcast_ref::<RiveRenderPaintHandle>() else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::drawPath(
                &mut self.renderer,
                path.source_base() as *const _ as *mut _,
                paint.source_base() as *const _ as *mut _,
            );
        }
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let Some(path) = path.as_any().downcast_ref::<RiveRenderPathHandle>() else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::clipPath(
                &mut self.renderer,
                path.source_base() as *const _ as *mut _,
            );
        }
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) =
            image.and_then(|value| value.as_any().downcast_ref::<RiveRenderImageHandle>())
        else {
            return;
        };
        let Some(image) = image.source_base_for(&self.resource_domain) else {
            return;
        };
        unsafe {
            <RiveRenderer as RendererContract>::drawImage(
                &mut self.renderer,
                image as *const _,
                source_image_sampler(sampler),
                blend_mode,
                opacity,
            );
        }
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) =
            image.and_then(|value| value.as_any().downcast_ref::<RiveRenderImageHandle>())
        else {
            return;
        };
        let Some(vertices) =
            vertices.and_then(|value| value.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(uv_coords) =
            uv_coords.and_then(|value| value.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(indices) =
            indices.and_then(|value| value.as_any().downcast_ref::<RiveRenderBufferHandle>())
        else {
            return;
        };
        let Some(image) = image.source_base_for(&self.resource_domain) else {
            return;
        };
        if !vertices.belongs_to(&self.resource_domain)
            || !uv_coords.belongs_to(&self.resource_domain)
            || !indices.belongs_to(&self.resource_domain)
        {
            return;
        }
        unsafe {
            <RiveRenderer as RendererContract>::drawImageMesh(
                &mut self.renderer,
                image as *const _,
                source_image_sampler(sampler),
                vertices.source_owner_unchecked(),
                uv_coords.source_owner_unchecked(),
                indices.source_owner_unchecked(),
                vertex_count,
                index_count,
                blend_mode,
                opacity,
            );
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        <RiveRenderer as RendererContract>::modulateOpacity(&mut self.renderer, opacity);
    }
}

fn source_buffer_type(
    value: RenderBufferType,
) -> crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType {
    match value {
        RenderBufferType::Index => {
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType::index
        }
        RenderBufferType::Vertex => {
            crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType::vertex
        }
    }
}

fn source_buffer_flags(
    value: RenderBufferFlags,
) -> crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags {
    match value {
        RenderBufferFlags::None => crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags::none,
        RenderBufferFlags::MappedOnceAtInitialization => crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags::mappedOnceAtInitialization,
    }
}

fn source_image_sampler(
    value: ImageSampler,
) -> crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler {
    use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
        ImageFilter, ImageSampler as SourceImageSampler, ImageWrap,
    };
    SourceImageSampler {
        wrapX: match value.wrap_x {
            nuxie_render_api::ImageWrap::Clamp => ImageWrap::clamp,
            nuxie_render_api::ImageWrap::Repeat => ImageWrap::repeat,
            nuxie_render_api::ImageWrap::Mirror => ImageWrap::mirror,
        },
        wrapY: match value.wrap_y {
            nuxie_render_api::ImageWrap::Clamp => ImageWrap::clamp,
            nuxie_render_api::ImageWrap::Repeat => ImageWrap::repeat,
            nuxie_render_api::ImageWrap::Mirror => ImageWrap::mirror,
        },
        filter: match value.filter {
            nuxie_render_api::ImageFilter::Bilinear => ImageFilter::bilinear,
            nuxie_render_api::ImageFilter::Nearest => ImageFilter::nearest,
        },
    }
}
