//! Experimental native Metal renderer adapter.
//!
//! The Apple product does not select this adapter until UNIV-2092. The module
//! begins as the UNIV-2086 tracer and grows by mechanically porting the pinned
//! upstream Metal implementation behind the existing renderer seam.

mod buffer;
mod capabilities;
#[allow(dead_code)]
mod pipeline_names;

use super::{LogicalPaint, LogicalPath, LogicalShader, RendererError};
use buffer::NativeMetalBuffer;
use capabilities::{
    select_capabilities, ApplePlatform, MetalCapabilitySelection, MetalDeviceCapabilities,
};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, PathVerb,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer,
};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{msg_send, rc::Retained};
use objc2_foundation::{NSError, NSString};
use objc2_metal::{
    MTLBlendFactor, MTLClearColor, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandEncoder,
    MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLGPUFamily, MTLLibrary,
    MTLLoadAction, MTLOrigin, MTLPixelFormat, MTLPrimitiveType, MTLRegion, MTLRenderCommandEncoder,
    MTLRenderPassDescriptor, MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLSize,
    MTLStorageMode, MTLStoreAction, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;

const TRACER_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_metal_tracer.metallib"));
const INLINE_VERTEX_BYTE_LIMIT: usize = 4_096;

#[link(name = "System")]
extern "C" {
    fn dispatch_data_create(
        buffer: NonNull<c_void>,
        size: usize,
        queue: Option<NonNull<c_void>>,
        destructor: *mut c_void,
    ) -> *mut AnyObject;
    fn dispatch_release(object: *mut AnyObject);
}

struct NativeMetalContext {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    solid_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    capabilities: MetalCapabilitySelection,
}

/// Explicitly selected native Metal renderer domain.
pub struct NativeMetalFactory {
    context: Arc<NativeMetalContext>,
    width: u32,
    height: u32,
}

impl NativeMetalFactory {
    pub fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| RendererError::NativeMetal("no system Metal device".to_owned()))?;
        let capabilities = select_device_capabilities(&device);
        validate_extent(width, height, capabilities.max_texture_size)?;
        let queue = device.newCommandQueue().ok_or_else(|| {
            RendererError::NativeMetal("MTLDevice returned no command queue".to_owned())
        })?;
        let solid_pipeline = make_solid_pipeline(&device)?;
        Ok(Self {
            context: Arc::new(NativeMetalContext {
                device,
                queue,
                solid_pipeline,
                capabilities,
            }),
            width,
            height,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Replaces the dimensions used by subsequently created frames. Frames
    /// already handed to a caller retain their original size, matching the
    /// upstream target-owner boundary during an in-flight resize.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        validate_extent(width, height, self.context.capabilities.max_texture_size)?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn begin_frame(&self, clear_color: u32) -> NativeMetalFrame {
        NativeMetalFrame {
            context: Arc::clone(&self.context),
            width: self.width,
            height: self.height,
            clear_color,
            state: NativeMetalRenderState::default(),
            state_stack: Vec::new(),
            solid_draws: Vec::new(),
            unsupported: None,
        }
    }
}

impl Factory for NativeMetalFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(NativeMetalBuffer::new(buffer_type, flags, size_in_bytes))
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
        Box::new(LogicalShader::Linear {
            start: (sx, sy),
            end: (ex, ey),
            colors: colors.to_vec(),
            stops: stops.to_vec(),
        })
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(LogicalShader::Radial {
            center: (cx, cy),
            radius,
            colors: colors.to_vec(),
            stops: stops.to_vec(),
        })
    }

    fn make_render_path(
        &mut self,
        mut raw_path: RawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
        raw_path.renew_mutation_id();
        Box::new(LogicalPath {
            raw_path: Arc::new(raw_path),
            fill_rule,
            valid: true,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(LogicalPath {
            raw_path: Arc::new(RawPath::new()),
            fill_rule: FillRule::NonZero,
            valid: true,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(LogicalPaint::default())
    }

    fn decode_image(&mut self, _data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        Err(ImageDecodeError)
    }
}

/// One native Metal frame retained until submission and deterministic readback.
pub struct NativeMetalFrame {
    context: Arc<NativeMetalContext>,
    width: u32,
    height: u32,
    clear_color: u32,
    state: NativeMetalRenderState,
    state_stack: Vec<NativeMetalRenderState>,
    solid_draws: Vec<SolidDraw>,
    unsupported: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct NativeMetalRenderState {
    transform: Mat2D,
    opacity: f32,
}

impl Default for NativeMetalRenderState {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
        }
    }
}

struct SolidDraw {
    vertices: Vec<[f32; 2]>,
    premultiplied_color: [f32; 4],
}

impl Renderer for NativeMetalFrame {
    fn save(&mut self) {
        self.state_stack.push(self.state);
    }

    fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
    }

    fn transform(&mut self, transform: Mat2D) {
        self.state.transform = super::multiply(self.state.transform, transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let Some(path) = path.as_any().downcast_ref::<LogicalPath>() else {
            self.unsupported
                .get_or_insert("path from another renderer backend");
            return;
        };
        let Some(paint) = paint.as_any().downcast_ref::<LogicalPaint>() else {
            self.unsupported
                .get_or_insert("paint from another renderer backend");
            return;
        };
        if !path.valid
            || paint.style != RenderPaintStyle::Fill
            || paint.shader.is_some()
            || paint.invalid_shader
            || paint.feather != 0.0
            || paint.blend_mode != BlendMode::SrcOver
        {
            self.unsupported
                .get_or_insert("native Metal tracer only supports solid SrcOver fills");
            return;
        }
        let Some(vertices) = solid_triangle_fan(path, self.state.transform) else {
            self.unsupported.get_or_insert(
                "native Metal tracer requires one closed convex finite line contour",
            );
            return;
        };
        let Some(vertex_bytes) = vertices.len().checked_mul(std::mem::size_of::<[f32; 2]>()) else {
            self.unsupported
                .get_or_insert("native Metal tracer path exceeds inline vertex limit");
            return;
        };
        if vertex_bytes > INLINE_VERTEX_BYTE_LIMIT {
            self.unsupported
                .get_or_insert("native Metal tracer path exceeds inline vertex limit");
            return;
        }
        self.solid_draws.push(SolidDraw {
            vertices,
            premultiplied_color: premultiplied_color(paint.color, self.state.opacity),
        });
    }

    fn clip_path(&mut self, _path: &dyn RenderPath) {
        self.unsupported
            .get_or_insert("native Metal tracer does not support clipping yet");
    }

    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        self.unsupported
            .get_or_insert("native Metal tracer does not support images yet");
    }

    fn draw_image_mesh(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        self.unsupported
            .get_or_insert("native Metal tracer does not support image meshes yet");
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.state.opacity *= opacity;
    }
}

impl NativeMetalFrame {
    pub fn finish(self) -> Result<Vec<u8>, RendererError> {
        if let Some(reason) = self.unsupported {
            return Err(RendererError::Unsupported(reason));
        }
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                MTLPixelFormat::RGBA8Unorm,
                self.width as usize,
                self.height as usize,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::RenderTarget);
        let texture = self
            .context
            .device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| RendererError::NativeMetal("failed to allocate render target".into()))?;

        let command_buffer = self.context.queue.commandBuffer().ok_or_else(|| {
            RendererError::NativeMetal("MTLCommandQueue returned no command buffer".into())
        })?;
        let pass = MTLRenderPassDescriptor::renderPassDescriptor();
        let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        attachment.setTexture(Some(&texture));
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setStoreAction(MTLStoreAction::Store);
        attachment.setClearColor(clear_color(self.clear_color));
        let encoder = command_buffer
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or_else(|| {
                RendererError::NativeMetal("failed to create render command encoder".into())
            })?;
        if !self.solid_draws.is_empty() {
            encoder.setRenderPipelineState(&self.context.solid_pipeline);
            let viewport = [self.width as f32, self.height as f32];
            let viewport_pointer = NonNull::from(&viewport).cast::<c_void>();
            unsafe {
                encoder.setVertexBytes_length_atIndex(
                    viewport_pointer,
                    std::mem::size_of_val(&viewport),
                    1,
                );
            }
            for draw in &self.solid_draws {
                let vertex_pointer =
                    NonNull::new(draw.vertices.as_ptr().cast_mut().cast::<c_void>())
                        .expect("solid draw has triangle vertices");
                let color_pointer = NonNull::from(&draw.premultiplied_color).cast::<c_void>();
                unsafe {
                    encoder.setVertexBytes_length_atIndex(
                        vertex_pointer,
                        std::mem::size_of_val(draw.vertices.as_slice()),
                        0,
                    );
                    encoder.setFragmentBytes_length_atIndex(
                        color_pointer,
                        std::mem::size_of_val(&draw.premultiplied_color),
                        0,
                    );
                    encoder.drawPrimitives_vertexStart_vertexCount(
                        MTLPrimitiveType::Triangle,
                        0,
                        draw.vertices.len(),
                    );
                }
            }
        }
        encoder.endEncoding();
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
        if command_buffer.status() != MTLCommandBufferStatus::Completed {
            let detail = command_buffer
                .error()
                .map(|error| format!("{error:?}"))
                .unwrap_or_else(|| format!("status {:?}", command_buffer.status()));
            return Err(RendererError::NativeMetal(format!(
                "command buffer failed: {detail}"
            )));
        }

        let row_bytes = usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| RendererError::NativeMetal("readback row size overflow".into()))?;
        let byte_len = row_bytes
            .checked_mul(self.height as usize)
            .ok_or_else(|| RendererError::NativeMetal("readback size overflow".into()))?;
        let mut pixels = vec![0; byte_len];
        let pointer = NonNull::new(pixels.as_mut_ptr().cast::<c_void>())
            .ok_or_else(|| RendererError::NativeMetal("readback buffer is null".into()))?;
        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: self.width as usize,
                height: self.height as usize,
                depth: 1,
            },
        };
        unsafe {
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(pointer, row_bytes, region, 0)
        };
        Ok(pixels)
    }
}

fn clear_color(color: u32) -> MTLClearColor {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let premultiply = |channel: u8| f64::from(u16::from(channel) * u16::from(alpha) / 255) / 255.0;
    MTLClearColor {
        red: premultiply(red),
        green: premultiply(green),
        blue: premultiply(blue),
        alpha: f64::from(alpha) / 255.0,
    }
}

fn validate_extent(width: u32, height: u32, max_texture_size: u32) -> Result<(), RendererError> {
    if width == 0 || height == 0 || width > max_texture_size || height > max_texture_size {
        return Err(RendererError::InvalidTextureExtent {
            label: "render target",
            width,
            height,
            max_dimension: max_texture_size,
        });
    }
    Ok(())
}

fn solid_triangle_fan(path: &LogicalPath, transform: Mat2D) -> Option<Vec<[f32; 2]>> {
    let verbs = path.raw_path.verbs();
    let line_count = verbs.iter().filter(|verb| **verb == PathVerb::Line).count();
    let point_count = path.raw_path.points().len();
    if verbs.first() != Some(&PathVerb::Move)
        || verbs.last() != Some(&PathVerb::Close)
        || line_count + 2 != verbs.len()
        || point_count != line_count + 1
    {
        return None;
    }
    let vertex_count = inline_triangle_fan_vertex_count(point_count)?;
    let points: Vec<[f32; 2]> = path
        .raw_path
        .points()
        .iter()
        .map(|point| {
            let point = transform.transform_point(*point);
            [point.x, point.y]
        })
        .collect();
    if !is_convex_finite_polygon(&points) {
        return None;
    }
    let mut vertices = Vec::with_capacity(vertex_count);
    for index in 1..points.len() - 1 {
        vertices.extend([points[0], points[index], points[index + 1]]);
    }
    Some(vertices)
}

fn inline_triangle_fan_vertex_count(point_count: usize) -> Option<usize> {
    let vertex_count = point_count.checked_sub(2)?.checked_mul(3)?;
    let vertex_bytes = vertex_count.checked_mul(std::mem::size_of::<[f32; 2]>())?;
    (point_count >= 3 && vertex_bytes <= INLINE_VERTEX_BYTE_LIMIT).then_some(vertex_count)
}

fn is_convex_finite_polygon(points: &[[f32; 2]]) -> bool {
    if points.len() < 3
        || points
            .iter()
            .flatten()
            .any(|coordinate| !coordinate.is_finite())
    {
        return false;
    }

    for first in 0..points.len() {
        let first_next = (first + 1) % points.len();
        for second in first + 1..points.len() {
            let second_next = (second + 1) % points.len();
            if first == second_next || first_next == second {
                continue;
            }
            if line_segments_intersect(
                points[first],
                points[first_next],
                points[second],
                points[second_next],
            ) {
                return false;
            }
        }
    }

    let mut winding = 0.0_f32;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        let c = points[(index + 2) % points.len()];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross.abs() <= f32::EPSILON {
            continue;
        }
        if winding != 0.0 && cross.signum() != winding.signum() {
            return false;
        }
        winding = cross;
    }
    winding != 0.0
}

fn line_segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    fn orientation(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }

    fn on_segment(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> bool {
        point[0] >= a[0].min(b[0])
            && point[0] <= a[0].max(b[0])
            && point[1] >= a[1].min(b[1])
            && point[1] <= a[1].max(b[1])
    }

    let ac = orientation(a, b, c);
    let ad = orientation(a, b, d);
    let ca = orientation(c, d, a);
    let cb = orientation(c, d, b);
    if ac * ad < 0.0 && ca * cb < 0.0 {
        return true;
    }
    (ac == 0.0 && on_segment(a, b, c))
        || (ad == 0.0 && on_segment(a, b, d))
        || (ca == 0.0 && on_segment(c, d, a))
        || (cb == 0.0 && on_segment(c, d, b))
}

fn select_device_capabilities(device: &ProtocolObject<dyn MTLDevice>) -> MetalCapabilitySelection {
    let device_capabilities = MetalDeviceCapabilities {
        supports_apple1: device.supportsFamily(MTLGPUFamily::Apple1),
        supports_apple2: device.supportsFamily(MTLGPUFamily::Apple2),
        supports_apple3: device.supportsFamily(MTLGPUFamily::Apple3),
        supports_common2: device.supportsFamily(MTLGPUFamily::Common2),
        supports_mac2: device.supportsFamily(MTLGPUFamily::Mac2),
        raster_order_groups: device.areRasterOrderGroupsSupported(),
    };

    #[cfg(target_os = "macos")]
    let platform = ApplePlatform::MacOs;
    #[cfg(all(target_os = "ios", target_abi = "sim"))]
    let platform = ApplePlatform::IosSimulator {
        host_is_arm64: cfg!(target_arch = "aarch64"),
    };
    #[cfg(all(target_os = "ios", not(target_abi = "sim")))]
    let platform = ApplePlatform::IosDevice {
        is_apple_silicon: device.supportsFamily(MTLGPUFamily::Apple4),
    };

    select_capabilities(platform, device_capabilities, false)
}

fn premultiplied_color(color: ColorInt, opacity: f32) -> [f32; 4] {
    let [alpha, red, green, blue] = color.to_be_bytes();
    let alpha = f32::from(alpha) / 255.0 * opacity.clamp(0.0, 1.0);
    [
        f32::from(red) / 255.0 * alpha,
        f32::from(green) / 255.0 * alpha,
        f32::from(blue) / 255.0 * alpha,
        alpha,
    ]
}

fn make_solid_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let library = new_library_from_metallib_bytes(device, TRACER_METALLIB)?;
    let vertex = library
        .newFunctionWithName(&NSString::from_str("nuxie_tracer_solid_vertex"))
        .ok_or_else(|| RendererError::NativeMetal("tracer vertex function is absent".into()))?;
    let fragment = library
        .newFunctionWithName(&NSString::from_str("nuxie_tracer_solid_fragment"))
        .ok_or_else(|| RendererError::NativeMetal("tracer fragment function is absent".into()))?;
    let descriptor = MTLRenderPipelineDescriptor::new();
    descriptor.setVertexFunction(Some(&vertex));
    descriptor.setFragmentFunction(Some(&fragment));
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
    attachment.setBlendingEnabled(true);
    attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
    attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
    attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    device
        .newRenderPipelineStateWithDescriptor_error(&descriptor)
        .map_err(|error| RendererError::NativeMetal(format!("create solid pipeline: {error:?}")))
}

fn new_library_from_metallib_bytes(
    device: &ProtocolObject<dyn MTLDevice>,
    bytes: &[u8],
) -> Result<Retained<ProtocolObject<dyn MTLLibrary>>, RendererError> {
    let buffer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>())
        .ok_or_else(|| RendererError::NativeMetal("embedded metallib is empty".into()))?;
    let data = unsafe { dispatch_data_create(buffer, bytes.len(), None, std::ptr::null_mut()) };
    if data.is_null() {
        return Err(RendererError::NativeMetal(
            "failed to create dispatch data for metallib".into(),
        ));
    }
    let result: Result<Retained<ProtocolObject<dyn MTLLibrary>>, Retained<NSError>> =
        unsafe { msg_send![device, newLibraryWithData: data, error: _] };
    unsafe { dispatch_release(data) };
    result.map_err(|error| RendererError::NativeMetal(format!("load tracer metallib: {error:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_filter_accepts_both_convex_windings() {
        assert!(is_convex_finite_polygon(&[
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [0.0, 1.0],
        ]));
        assert!(is_convex_finite_polygon(&[
            [0.0, 1.0],
            [2.0, 1.0],
            [2.0, 0.0],
            [0.0, 0.0],
        ]));
    }

    #[test]
    fn polygon_filter_rejects_geometry_the_triangle_fan_cannot_render_exactly() {
        assert!(!is_convex_finite_polygon(&[
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, 0.5],
            [2.0, 1.0],
            [0.0, 1.0],
        ]));
        assert!(!is_convex_finite_polygon(&[
            [0.0, 0.0],
            [f32::NAN, 0.0],
            [0.0, 1.0],
        ]));
        assert!(!is_convex_finite_polygon(&[
            [1.0, 0.0],
            [-0.809_017, 0.587_785],
            [0.309_017, -0.951_057],
            [0.309_017, 0.951_057],
            [-0.809_017, -0.587_785],
        ]));
    }

    #[test]
    fn extent_validation_reports_the_selected_device_limit() {
        assert!(validate_extent(8_192, 8_192, 8_192).is_ok());
        assert!(matches!(
            validate_extent(8_193, 1, 8_192),
            Err(RendererError::InvalidTextureExtent {
                max_dimension: 8_192,
                ..
            })
        ));
    }

    #[test]
    fn triangle_fan_size_is_bounded_before_allocating() {
        assert_eq!(inline_triangle_fan_vertex_count(172), Some(510));
        assert_eq!(inline_triangle_fan_vertex_count(173), None);
        assert_eq!(inline_triangle_fan_vertex_count(usize::MAX), None);
    }
}
