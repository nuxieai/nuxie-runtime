use super::webgl2_limits::WebGl2FrameIntermediateBudget;
use super::{
    decode_image_rgba, draw, gr_triangulator, invert, multiply, transform_rect_to_new_space,
    RendererError,
};
use femtovg::renderer::OpenGl;
use femtovg::{
    Canvas, Color, CompositeOperation, FillRule as FemtovgFillRule,
    ImageFilter as FemtovgImageFilter, ImageFlags, ImageId, LineCap, LineJoin, Paint, Path,
    PixelFormat, RenderTarget, Transform2D,
};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPlan, GpuCanvasShader,
    ImageDecodeError, ImageFilter, ImageSampler, ImageWrap, Mat2D, PathVerb, RawPath, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin, Vec2D,
};
use rgb::RGBA8;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

type WebGl2Canvas = Canvas<OpenGl>;

/// WebGL2 renderer factory bound to one HTML canvas.
///
/// The compatibility path supports solid and gradient paths, strokes,
/// clockwise/nonzero/even-odd fills, transformed and arbitrary nested clips,
/// and nearest/bilinear clamp or repeat image sampling. Unsupported
/// capabilities fail when the frame is finished instead of rendering partial
/// output.
pub struct WebGl2Factory {
    element: web_sys::HtmlCanvasElement,
    canvas: Rc<RefCell<WebGl2Canvas>>,
    frame_active: Rc<Cell<bool>>,
    poisoned: Rc<Cell<bool>>,
    pending_mesh_texture_deletes: Rc<RefCell<Vec<web_sys::WebGlTexture>>>,
    owner: Rc<()>,
    mesh_renderer: Rc<RefCell<Option<WebGl2MeshRenderer>>>,
    imported_gpu_canvas: Option<super::gpu_canvas::ImportedWebGl2GpuCanvasRenderer>,
    gpu_canvas_targets: Arc<super::gpu_canvas::RetainedGpuCanvasTargetBudget>,
    width: u32,
    height: u32,
}

impl WebGl2Factory {
    /// Creates a WebGL2 renderer for `element` and configures its pixel size.
    pub fn new(
        element: &web_sys::HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::WebGl2(
                "render target dimensions must be nonzero".into(),
            ));
        }
        element.set_width(width);
        element.set_height(height);
        let renderer = OpenGl::new_from_html_canvas(element)
            .map_err(|error| RendererError::WebGl2(error.to_string()))?;
        let mut canvas =
            Canvas::new(renderer).map_err(|error| RendererError::WebGl2(error.to_string()))?;
        canvas.set_size(width, height, 1.0);
        Ok(Self {
            element: element.clone(),
            canvas: Rc::new(RefCell::new(canvas)),
            frame_active: Rc::new(Cell::new(false)),
            poisoned: Rc::new(Cell::new(false)),
            pending_mesh_texture_deletes: Rc::new(RefCell::new(Vec::new())),
            owner: Rc::new(()),
            mesh_renderer: Rc::new(RefCell::new(None)),
            imported_gpu_canvas: None,
            gpu_canvas_targets: Arc::new(
                super::gpu_canvas::RetainedGpuCanvasTargetBudget::default(),
            ),
            width,
            height,
        })
    }

    /// Begins a frame after validating the factory lifecycle.
    pub fn begin_frame(&self, clear_color: ColorInt) -> Result<WebGl2Frame, RendererError> {
        if self.poisoned.get() {
            return Err(RendererError::WebGl2(
                "renderer was poisoned by an abandoned frame".into(),
            ));
        }
        if self.frame_active.replace(true) {
            return Err(RendererError::WebGl2(
                "only one WebGL2 frame may be active at a time".into(),
            ));
        }
        let mut canvas = self.canvas.borrow_mut();
        if let Some(renderer) = self.mesh_renderer.borrow().as_ref() {
            delete_pending_mesh_textures(renderer, &self.pending_mesh_texture_deletes);
        }
        canvas.reset();
        canvas.reset_transform();
        canvas.set_global_alpha(1.0);
        canvas.clear_rect(0, 0, self.width, self.height, color(clear_color));
        drop(canvas);
        Ok(WebGl2Frame {
            canvas: Rc::clone(&self.canvas),
            frame_active: Rc::clone(&self.frame_active),
            poisoned: Rc::clone(&self.poisoned),
            pending_mesh_texture_deletes: Rc::clone(&self.pending_mesh_texture_deletes),
            owner: Rc::clone(&self.owner),
            element: self.element.clone(),
            mesh_renderer: Rc::clone(&self.mesh_renderer),
            stack: Vec::new(),
            state: WebGl2State::default(),
            clip_layers: Vec::new(),
            intermediate_budget: WebGl2FrameIntermediateBudget::default(),
            clear_color,
            width: self.width,
            height: self.height,
            unsupported: None,
            finished: false,
        })
    }

    /// Retargets future frames and the bound HTML canvas without recreating
    /// the WebGL context. Active or poisoned factories reject the operation
    /// before changing either extent.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::WebGl2(
                "render target dimensions must be nonzero".into(),
            ));
        }
        if self.poisoned.get() {
            return Err(RendererError::WebGl2(
                "renderer was poisoned by an abandoned frame".into(),
            ));
        }
        if self.frame_active.get() {
            return Err(RendererError::WebGl2(
                "cannot resize while a WebGL2 frame is active".into(),
            ));
        }
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        self.element.set_width(width);
        self.element.set_height(height);
        self.canvas.borrow_mut().set_size(width, height, 1.0);
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn make_rgba_image(
        &mut self,
        width: u32,
        height: u32,
        bytes: &[u8],
    ) -> Result<Box<dyn RenderImage>, RendererError> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| RendererError::WebGl2("image byte length overflow".into()))?;
        if bytes.len() != expected {
            return Err(RendererError::WebGl2(format!(
                "RGBA image contains {} bytes; expected {expected}",
                bytes.len()
            )));
        }
        let pixels = bytes
            .chunks_exact(4)
            .map(|pixel| RGBA8 {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
                a: pixel[3],
            })
            .collect::<Vec<_>>();
        Ok(Box::new(WebGl2Image {
            width,
            height,
            pixels,
            mesh_texture: RefCell::new(None),
            pending_mesh_texture_deletes: Rc::downgrade(&self.pending_mesh_texture_deletes),
            owner: Rc::downgrade(&self.owner),
        }))
    }
}

impl Factory for WebGl2Factory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(WebGl2Buffer {
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
            owner: Rc::downgrade(&self.owner),
            unmapped: false,
        })
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
        let stops = gradient_stops(colors, stops);
        Box::new(WebGl2Shader::Linear {
            start: [sx, sy],
            end: [ex, ey],
            valid: stops.is_some(),
            stops: stops.unwrap_or_default(),
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
        let stops = gradient_stops(colors, stops);
        Box::new(WebGl2Shader::Radial {
            center: [cx, cy],
            radius,
            valid: stops.is_some() && radius.is_finite() && radius >= 0.0,
            stops: stops.unwrap_or_default(),
        })
    }

    fn make_render_path(
        &mut self,
        mut raw_path: RawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
        raw_path.renew_mutation_id();
        Box::new(WebGl2Path {
            raw_path,
            fill_rule,
            valid: true,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(WebGl2Path {
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
            valid: true,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(WebGl2Paint::default())
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let Some((width, height, bytes)) = decode_image_rgba(data) else {
            return Err(ImageDecodeError);
        };
        self.make_rgba_image(width, height, &bytes)
            .map_err(|_| ImageDecodeError)
    }

    fn make_gpu_canvas_image(
        &mut self,
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        super::gpu_canvas::validate_imported_gpu_canvas_plan(plan)
            .map_err(|error| GpuCanvasError::new(error.to_string()))?;
        let target_lease = self.gpu_canvas_targets.acquire(plan.width, plan.height)?;
        if self.imported_gpu_canvas.is_none() {
            self.imported_gpu_canvas = Some(
                super::gpu_canvas::ImportedWebGl2GpuCanvasRenderer::new(&self.element)?,
            );
        }
        let pixels = self
            .imported_gpu_canvas
            .as_mut()
            .expect("imported WebGL2 renderer was initialized")
            .render(shader, plan)?;
        let image = self
            .make_rgba_image(plan.width, plan.height, &pixels)
            .map_err(|error| GpuCanvasError::new(error.to_string()))?;
        Ok(super::gpu_canvas::retain_gpu_canvas_target(
            image,
            target_lease,
        ))
    }
}

/// In-progress WebGL2 frame created by [`WebGl2Factory::begin_frame`].
///
/// Dropping a frame without calling [`finish`](Self::finish) poisons its
/// factory so queued commands cannot leak into a later frame.
pub struct WebGl2Frame {
    canvas: Rc<RefCell<WebGl2Canvas>>,
    frame_active: Rc<Cell<bool>>,
    poisoned: Rc<Cell<bool>>,
    pending_mesh_texture_deletes: Rc<RefCell<Vec<web_sys::WebGlTexture>>>,
    owner: Rc<()>,
    element: web_sys::HtmlCanvasElement,
    mesh_renderer: Rc<RefCell<Option<WebGl2MeshRenderer>>>,
    stack: Vec<WebGl2State>,
    state: WebGl2State,
    clip_layers: Vec<WebGl2ClipLayer>,
    intermediate_budget: WebGl2FrameIntermediateBudget,
    clear_color: ColorInt,
    width: u32,
    height: u32,
    unsupported: Option<&'static str>,
    finished: bool,
}

#[derive(Clone, Copy)]
struct WebGl2State {
    transform: Mat2D,
    opacity: f32,
    clip_matrix: Option<Mat2D>,
    clip_rect: Option<[f32; 4]>,
    clip_depth: usize,
}

impl Default for WebGl2State {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
            clip_matrix: None,
            clip_rect: None,
            clip_depth: 0,
        }
    }
}

struct WebGl2ClipLayer {
    content: ImageId,
    mask: ImageId,
    parent_target: RenderTarget,
    reserved_intermediate_bytes: usize,
}

impl WebGl2Frame {
    /// Flushes WebGL2 work and returns the canvas contents as RGBA pixels.
    ///
    /// If replay requested an unsupported capability, this clears the queued
    /// frame and returns a named [`RendererError::Unsupported`] error.
    pub fn finish(mut self) -> Result<Vec<u8>, RendererError> {
        self.finished = true;
        self.frame_active.set(false);
        self.close_clip_layers_to(0);
        self.unwind_state_stack();
        let mut canvas = self.canvas.borrow_mut();
        if let Some(feature) = self.unsupported {
            canvas.reset();
            canvas.clear_rect(0, 0, self.width, self.height, color(self.clear_color));
            canvas.flush();
            return Err(RendererError::Unsupported(feature));
        }
        canvas.flush();
        let screenshot = canvas
            .screenshot()
            .map_err(|error| RendererError::WebGl2(error.to_string()))?;
        let mut pixels = Vec::with_capacity(screenshot.buf().len() * 4);
        for pixel in screenshot.buf() {
            pixels.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
        }
        if let Some(renderer) = self.mesh_renderer.borrow().as_ref() {
            delete_pending_mesh_textures(renderer, &self.pending_mesh_texture_deletes);
        }
        Ok(pixels)
    }

    fn reject(&mut self, feature: &'static str) {
        self.unsupported.get_or_insert(feature);
    }

    fn draw_feathered_path(
        &mut self,
        path: &WebGl2Path,
        paint: &WebGl2Paint,
        femtovg_path: &Path,
        femtovg_paint: &mut Paint,
    ) -> bool {
        if path.raw_path.points().is_empty() {
            return true;
        }
        let Some(plan) = feather_image_plan(path, paint, self.state.transform) else {
            return false;
        };
        let Some(reserved_intermediate_bytes) =
            self.intermediate_budget
                .try_reserve_rgba_images(plan.width, plan.height, 2)
        else {
            return false;
        };
        let parent_target = self
            .clip_layers
            .last()
            .map_or(RenderTarget::Screen, |layer| {
                RenderTarget::Image(layer.content)
            });
        let flags = ImageFlags::FLIP_Y | ImageFlags::PREMULTIPLIED;
        let mut canvas = self.canvas.borrow_mut();
        let Ok(source) =
            canvas.create_image_empty(plan.width, plan.height, PixelFormat::Rgba8, flags)
        else {
            drop(canvas);
            self.intermediate_budget
                .release(reserved_intermediate_bytes);
            return false;
        };
        let Ok(filtered) =
            canvas.create_image_empty(plan.width, plan.height, PixelFormat::Rgba8, flags)
        else {
            canvas.delete_image(source);
            drop(canvas);
            self.intermediate_budget
                .release(reserved_intermediate_bytes);
            return false;
        };
        // Render in path-local coordinates. Keeping the blur texture local to
        // the path makes a circular authored blur follow rotation, skew, and
        // non-uniform scale when it is composited through the caller's matrix.
        canvas.save();
        canvas.reset();
        canvas.set_render_target(RenderTarget::Image(source));
        canvas.clear_rect(
            0,
            0,
            plan.width as u32,
            plan.height as u32,
            Color::rgba(0, 0, 0, 0),
        );
        canvas.set_transform(&Transform2D([
            plan.scale,
            0.0,
            0.0,
            plan.scale,
            -plan.left * plan.scale,
            -plan.top * plan.scale,
        ]));
        match paint.style {
            RenderPaintStyle::Fill => canvas.fill_path(femtovg_path, femtovg_paint),
            RenderPaintStyle::Stroke => {
                femtovg_paint.set_line_width(paint.thickness);
                femtovg_paint.set_line_join(line_join(paint.join));
                femtovg_paint.set_line_cap(line_cap(paint.cap));
                canvas.stroke_path(femtovg_path, femtovg_paint);
            }
        }
        canvas.filter_image(
            filtered,
            FemtovgImageFilter::GaussianBlur { sigma: plan.sigma },
            source,
        );
        canvas.set_render_target(parent_target);
        canvas.restore();

        // The restored state carries the caller's scissor, alpha, and
        // source-over compositing. Drawing the local image through that state
        // applies clips after convolution and opacity exactly once.
        let mut image_path = Path::new();
        image_path.rect(plan.left, plan.top, plan.logical_width, plan.logical_height);
        canvas.fill_path(
            &image_path,
            &Paint::image(
                filtered,
                plan.left,
                plan.top,
                plan.logical_width,
                plan.logical_height,
                0.0,
                1.0,
            ),
        );
        canvas.flush();
        canvas.delete_image(source);
        canvas.delete_image(filtered);
        drop(canvas);
        self.intermediate_budget
            .release(reserved_intermediate_bytes);
        true
    }

    fn push_clip_layer(&mut self, path: &WebGl2Path) -> bool {
        let femtovg_path = if path.fill_rule == FillRule::Clockwise {
            clockwise_femtovg_path(&path.raw_path, self.state.transform)
        } else {
            path.to_femtovg()
        };
        let Some(femtovg_path) = femtovg_path else {
            return false;
        };
        let parent_target = self
            .clip_layers
            .last()
            .map_or(RenderTarget::Screen, |layer| {
                RenderTarget::Image(layer.content)
            });
        let Some(reserved_intermediate_bytes) = self.intermediate_budget.try_reserve_rgba_images(
            self.width as usize,
            self.height as usize,
            2,
        ) else {
            return false;
        };
        let flags = ImageFlags::FLIP_Y | ImageFlags::PREMULTIPLIED;
        let mut canvas = self.canvas.borrow_mut();
        let Ok(content) = canvas.create_image_empty(
            self.width as usize,
            self.height as usize,
            PixelFormat::Rgba8,
            flags,
        ) else {
            drop(canvas);
            self.intermediate_budget
                .release(reserved_intermediate_bytes);
            return false;
        };
        let Ok(mask) = canvas.create_image_empty(
            self.width as usize,
            self.height as usize,
            PixelFormat::Rgba8,
            flags,
        ) else {
            canvas.delete_image(content);
            drop(canvas);
            self.intermediate_budget
                .release(reserved_intermediate_bytes);
            return false;
        };

        canvas.save();
        canvas.reset();
        canvas.set_render_target(RenderTarget::Image(mask));
        canvas.clear_rect(0, 0, self.width, self.height, Color::rgba(0, 0, 0, 0));
        canvas.set_transform(&Transform2D(self.state.transform.0));
        let mut mask_paint = Paint::color(Color::white());
        mask_paint.set_fill_rule(match path.fill_rule {
            FillRule::EvenOdd => FemtovgFillRule::EvenOdd,
            FillRule::NonZero | FillRule::Clockwise => FemtovgFillRule::NonZero,
        });
        canvas.fill_path(&femtovg_path, &mask_paint);

        canvas.set_render_target(RenderTarget::Image(content));
        canvas.clear_rect(0, 0, self.width, self.height, Color::rgba(0, 0, 0, 0));
        canvas.restore();
        drop(canvas);

        self.clip_layers.push(WebGl2ClipLayer {
            content,
            mask,
            parent_target,
            reserved_intermediate_bytes,
        });
        self.state.clip_depth = self.clip_layers.len();
        true
    }

    fn close_clip_layers_to(&mut self, depth: usize) {
        while self.clip_layers.len() > depth {
            let layer = self
                .clip_layers
                .pop()
                .expect("clip layer depth was checked");
            let mut canvas = self.canvas.borrow_mut();
            canvas.save();
            canvas.reset();

            let mut frame_path = Path::new();
            frame_path.rect(0.0, 0.0, self.width as f32, self.height as f32);
            canvas.set_render_target(RenderTarget::Image(layer.content));
            canvas.global_composite_operation(CompositeOperation::DestinationIn);
            canvas.fill_path(
                &frame_path,
                &Paint::image(
                    layer.mask,
                    0.0,
                    0.0,
                    self.width as f32,
                    self.height as f32,
                    0.0,
                    1.0,
                ),
            );

            canvas.set_render_target(layer.parent_target);
            canvas.global_composite_operation(CompositeOperation::SourceOver);
            canvas.fill_path(
                &frame_path,
                &Paint::image(
                    layer.content,
                    0.0,
                    0.0,
                    self.width as f32,
                    self.height as f32,
                    0.0,
                    1.0,
                ),
            );
            canvas.restore();
            canvas.flush();
            canvas.delete_image(layer.mask);
            canvas.delete_image(layer.content);
            drop(canvas);
            self.intermediate_budget
                .release(layer.reserved_intermediate_bytes);
        }
        self.state.clip_depth = self.clip_layers.len();
    }

    fn unwind_state_stack(&mut self) {
        let mut canvas = self.canvas.borrow_mut();
        for _ in self.stack.drain(..) {
            canvas.restore();
        }
        self.state = WebGl2State::default();
    }
}

impl Drop for WebGl2Frame {
    fn drop(&mut self) {
        if !self.finished {
            self.frame_active.set(false);
            self.poisoned.set(true);
        }
    }
}

impl Renderer for WebGl2Frame {
    fn save(&mut self) {
        self.stack.push(self.state);
        self.canvas.borrow_mut().save();
    }

    fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.close_clip_layers_to(state.clip_depth);
            self.state = state;
            self.canvas.borrow_mut().restore();
        }
    }

    fn transform(&mut self, transform: Mat2D) {
        self.state.transform = multiply(self.state.transform, transform);
        let mut canvas = self.canvas.borrow_mut();
        canvas.reset_transform();
        canvas.set_transform(&Transform2D(self.state.transform.0));
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let Some(path) = path.as_any().downcast_ref::<WebGl2Path>() else {
            self.reject("WebGL2 path from another renderer backend");
            return;
        };
        let Some(paint) = paint.as_any().downcast_ref::<WebGl2Paint>() else {
            self.reject("WebGL2 paint from another renderer backend");
            return;
        };
        if paint.blend_mode != BlendMode::SrcOver {
            self.reject("WebGL2 advanced blend modes");
            return;
        }
        if !path.valid {
            self.reject("WebGL2 path contains resources from another renderer backend");
            return;
        }
        if paint.invalid_shader {
            self.reject("WebGL2 paint shader from another renderer backend");
            return;
        }
        let femtovg_path =
            if paint.style == RenderPaintStyle::Fill && path.fill_rule == FillRule::Clockwise {
                clockwise_femtovg_path(&path.raw_path, self.state.transform)
            } else {
                path.to_femtovg()
            };
        let Some(femtovg_path) = femtovg_path else {
            self.reject("malformed WebGL2 path data");
            return;
        };
        let mut femtovg_paint = paint.to_femtovg(path.fill_rule);
        if paint.feather != 0.0 {
            if !paint.feather.is_finite() {
                self.reject("non-finite WebGL2 feather");
            } else if !self.draw_feathered_path(path, paint, &femtovg_path, &mut femtovg_paint) {
                self.reject("WebGL2 feather layer allocation or bounds");
            }
            return;
        }
        if paint.style == RenderPaintStyle::Fill
            && (path_is_axis_aligned_pixel_rect(&path.raw_path, self.state.transform)
                || path_is_axis_aligned_pixel_rounded_rect(&path.raw_path, self.state.transform))
        {
            // Femtovg's one-pixel antialias fringe expands an already
            // pixel-aligned box by a full device pixel on every edge. Browsers
            // and Rive's GPU backends keep the half-open box boundary exact,
            // so skip the redundant fringe for plain and uniformly rounded
            // rectangles. Rounded corners retain their tessellated curve;
            // only the extra fringe is omitted.
            femtovg_paint.set_anti_alias(false);
        }
        match paint.style {
            RenderPaintStyle::Fill => self
                .canvas
                .borrow_mut()
                .fill_path(&femtovg_path, &femtovg_paint),
            RenderPaintStyle::Stroke => {
                femtovg_paint.set_line_width(paint.thickness);
                femtovg_paint.set_line_join(line_join(paint.join));
                femtovg_paint.set_line_cap(line_cap(paint.cap));
                self.canvas
                    .borrow_mut()
                    .stroke_path(&femtovg_path, &femtovg_paint);
            }
        }
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let Some(path) = path.as_any().downcast_ref::<WebGl2Path>() else {
            self.reject("WebGL2 clip path from another renderer backend");
            return;
        };
        if !path.valid {
            self.reject("WebGL2 clip path contains resources from another renderer backend");
            return;
        }
        if let Some([left, top, right, bottom]) = path_rect(&path.raw_path) {
            let base_rect = [left, top, right, bottom];
            let compatible_rect = match self.state.clip_matrix {
                None => Some(base_rect),
                Some(matrix) => transform_rect_to_new_space(
                    [left, top, right, bottom],
                    self.state.transform,
                    matrix,
                ),
            };
            if let Some(compatible_rect) = compatible_rect {
                self.canvas
                    .borrow_mut()
                    .intersect_scissor(left, top, right - left, bottom - top);
                if self.state.clip_matrix.is_none() {
                    self.state.clip_matrix = Some(self.state.transform);
                }
                self.state.clip_rect = Some(match self.state.clip_rect {
                    None => compatible_rect,
                    Some(existing) => [
                        existing[0].max(compatible_rect[0]),
                        existing[1].max(compatible_rect[1]),
                        existing[2].min(compatible_rect[2]),
                        existing[3].min(compatible_rect[3]),
                    ],
                });
                return;
            }
        }
        if !self.push_clip_layer(path) {
            self.reject("WebGL2 clip layer allocation or path construction");
        }
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let Some(image) = image else {
            return;
        };
        if blend_mode != BlendMode::SrcOver {
            self.reject("WebGL2 advanced image blend modes");
            return;
        }
        let Some(image) = image.as_any().downcast_ref::<WebGl2Image>() else {
            self.reject("WebGL2 image from another renderer backend");
            return;
        };
        let Some(image_owner) = image.owner.upgrade() else {
            self.reject("WebGL2 image owner is no longer available");
            return;
        };
        if !Rc::ptr_eq(&self.owner, &image_owner) {
            self.reject("WebGL2 image belongs to another renderer factory");
            return;
        }
        if !webgl2_sampler_supported(sampler) {
            self.reject("WebGL2 mirrored image wrapping");
            return;
        }

        // Femtovg's image-pattern matrix does not have Rive's drawImage
        // contract: with translation plus non-uniform scale it applies the
        // current matrix to the pattern and rectangle differently. Submit the
        // ordinary image through the same explicit textured-quad path as an
        // image mesh so the world transform and DPR are each applied once.
        let local = [
            Vec2D::new(0.0, 0.0),
            Vec2D::new(image.width as f32, 0.0),
            Vec2D::new(image.width as f32, image.height as f32),
            Vec2D::new(0.0, image.height as f32),
        ];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let mut interleaved = Vec::with_capacity(16);
        for (point, uv) in local.into_iter().zip(uvs) {
            let point = self.state.transform.transform_point(point);
            interleaved.extend([point.x, point.y, uv[0], uv[1]]);
        }
        if interleaved.iter().any(|value| !value.is_finite()) {
            self.reject("non-finite WebGL2 image transform");
            return;
        }
        let clip = match (self.state.clip_matrix, self.state.clip_rect) {
            (Some(matrix), Some(rect)) => {
                let Some(inverse) = invert(matrix) else {
                    self.reject("non-invertible WebGL2 image clip");
                    return;
                };
                Some((inverse, rect))
            }
            _ => None,
        };

        self.canvas.borrow_mut().flush();
        let mut slot = self.mesh_renderer.borrow_mut();
        if slot.is_none() {
            let Ok(renderer) = WebGl2MeshRenderer::new(&self.element) else {
                drop(slot);
                self.reject("WebGL2 image pipeline allocation");
                return;
            };
            *slot = Some(renderer);
        }
        let renderer = slot
            .as_ref()
            .expect("WebGL2 image renderer was initialized");
        delete_pending_mesh_textures(renderer, &self.pending_mesh_texture_deletes);
        let texture = match renderer.texture_for(image, sampler) {
            Ok(texture) => texture,
            Err(()) => {
                drop(slot);
                self.reject("WebGL2 image texture allocation");
                return;
            }
        };
        let indices = [0u16, 1, 2, 0, 2, 3];
        let draw = WebGl2MeshDraw {
            vertices: bytemuck::cast_slice(&interleaved),
            indices: bytemuck::cast_slice(&indices),
            index_count: indices.len() as i32,
            texture: &texture,
            opacity: (opacity * self.state.opacity).max(0.0),
            clip,
            width: self.width,
            height: self.height,
        };
        if renderer.draw(draw).is_err() {
            drop(slot);
            self.reject("WebGL2 image draw submission");
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
        let Some(image) = image else {
            return;
        };
        if blend_mode != BlendMode::SrcOver {
            self.reject("WebGL2 advanced image blend modes");
            return;
        }
        if !webgl2_sampler_supported(sampler) {
            self.reject("WebGL2 mirrored image wrapping");
            return;
        }
        let Some(image) = image.as_any().downcast_ref::<WebGl2Image>() else {
            self.reject("WebGL2 image from another renderer backend");
            return;
        };
        let Some(image_owner) = image.owner.upgrade() else {
            self.reject("WebGL2 image owner is no longer available");
            return;
        };
        if !Rc::ptr_eq(&self.owner, &image_owner) {
            self.reject("WebGL2 image belongs to another renderer factory");
            return;
        }
        let (Some(vertices), Some(uvs), Some(indices)) = (
            vertices.and_then(webgl2_buffer),
            uv_coords.and_then(webgl2_buffer),
            indices.and_then(webgl2_buffer),
        ) else {
            self.reject("invalid WebGL2 image mesh buffers");
            return;
        };
        let buffers_belong_to_factory = [&vertices.owner, &uvs.owner, &indices.owner]
            .into_iter()
            .all(|owner| {
                owner
                    .upgrade()
                    .is_some_and(|owner| Rc::ptr_eq(&self.owner, &owner))
            });
        let required_vertex_bytes = usize::try_from(vertex_count)
            .ok()
            .and_then(|count| count.checked_mul(8));
        let required_index_bytes = usize::try_from(index_count)
            .ok()
            .and_then(|count| count.checked_mul(2));
        if !buffers_belong_to_factory
            || vertices.buffer_type != RenderBufferType::Vertex
            || uvs.buffer_type != RenderBufferType::Vertex
            || indices.buffer_type != RenderBufferType::Index
            || required_vertex_bytes
                .is_none_or(|size| vertices.bytes.len() < size || uvs.bytes.len() < size)
            || required_index_bytes.is_none_or(|size| indices.bytes.len() < size)
        {
            self.reject("malformed WebGL2 image mesh buffers");
            return;
        }
        if !vertices.unmapped || !uvs.unmapped || !indices.unmapped {
            self.reject("unmapped WebGL2 image mesh buffers");
            return;
        }
        let Some(vertex_count) = usize::try_from(vertex_count).ok() else {
            self.reject("WebGL2 image mesh vertex count overflow");
            return;
        };
        let Some(index_count_i32) = i32::try_from(index_count).ok() else {
            self.reject("WebGL2 image mesh index count overflow");
            return;
        };
        let mut interleaved = Vec::with_capacity(vertex_count.saturating_mul(4));
        for index in 0..vertex_count {
            let Some(position) = read_f32_pair(&vertices.bytes, index) else {
                self.reject("malformed WebGL2 image mesh vertex data");
                return;
            };
            let Some(uv) = read_f32_pair(&uvs.bytes, index) else {
                self.reject("malformed WebGL2 image mesh UV data");
                return;
            };
            let position = self
                .state
                .transform
                .transform_point(Vec2D::new(position[0], position[1]));
            interleaved.extend([position.x, position.y, uv[0], uv[1]]);
        }
        if interleaved.iter().any(|value| !value.is_finite()) {
            self.reject("non-finite WebGL2 image mesh data");
            return;
        }
        let clip = match (self.state.clip_matrix, self.state.clip_rect) {
            (Some(matrix), Some(rect)) => {
                let Some(inverse) = invert(matrix) else {
                    self.reject("non-invertible WebGL2 image mesh clip");
                    return;
                };
                Some((inverse, rect))
            }
            _ => None,
        };

        self.canvas.borrow_mut().flush();
        let mut slot = self.mesh_renderer.borrow_mut();
        if slot.is_none() {
            let Ok(renderer) = WebGl2MeshRenderer::new(&self.element) else {
                drop(slot);
                self.reject("WebGL2 image mesh pipeline allocation");
                return;
            };
            *slot = Some(renderer);
        }
        let renderer = slot
            .as_ref()
            .expect("WebGL2 image mesh renderer was initialized");
        delete_pending_mesh_textures(renderer, &self.pending_mesh_texture_deletes);
        let texture = match renderer.texture_for(image, sampler) {
            Ok(texture) => texture,
            Err(()) => {
                drop(slot);
                self.reject("WebGL2 image mesh texture allocation");
                return;
            }
        };
        let draw = WebGl2MeshDraw {
            vertices: bytemuck::cast_slice(&interleaved),
            indices: &indices.bytes[..required_index_bytes.unwrap_or(0)],
            index_count: index_count_i32,
            texture: &texture,
            opacity: (opacity * self.state.opacity).max(0.0),
            clip,
            width: self.width,
            height: self.height,
        };
        if renderer.draw(draw).is_err() {
            drop(slot);
            self.reject("WebGL2 image mesh draw submission");
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.state.opacity *= opacity;
        self.canvas
            .borrow_mut()
            .set_global_alpha(self.state.opacity);
    }
}

#[derive(Clone)]
struct WebGl2Path {
    raw_path: RawPath,
    fill_rule: FillRule,
    valid: bool,
}

impl WebGl2Path {
    fn to_femtovg(&self) -> Option<Path> {
        raw_path_to_femtovg(&self.raw_path)
    }
}

impl RenderPath for WebGl2Path {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        if let Some(path) = path.as_any().downcast_ref::<Self>() {
            self.raw_path.add_path(&path.raw_path, transform);
        } else {
            self.valid = false;
        }
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        if let Some(path) = path.as_any().downcast_ref::<Self>() {
            self.raw_path.add_path_backwards(&path.raw_path, transform);
        } else {
            self.valid = false;
        }
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

#[derive(Clone)]
enum WebGl2Shader {
    Linear {
        start: [f32; 2],
        end: [f32; 2],
        stops: Vec<(f32, Color)>,
        valid: bool,
    },
    Radial {
        center: [f32; 2],
        radius: f32,
        stops: Vec<(f32, Color)>,
        valid: bool,
    },
}

impl WebGl2Shader {
    fn is_valid(&self) -> bool {
        match self {
            Self::Linear {
                start, end, valid, ..
            } => *valid && start.iter().chain(end).all(|value| value.is_finite()),
            Self::Radial {
                center,
                radius,
                valid,
                ..
            } => *valid && center.iter().all(|value| value.is_finite()) && radius.is_finite(),
        }
    }
}

impl RenderShader for WebGl2Shader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
struct WebGl2Paint {
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader: Option<WebGl2Shader>,
    invalid_shader: bool,
}

impl Default for WebGl2Paint {
    fn default() -> Self {
        Self {
            style: RenderPaintStyle::Fill,
            color: 0xff00_0000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader: None,
            invalid_shader: false,
        }
    }
}

impl WebGl2Paint {
    fn to_femtovg(&self, fill_rule: FillRule) -> Paint {
        let mut paint = match &self.shader {
            Some(WebGl2Shader::Linear {
                start, end, stops, ..
            }) => Paint::linear_gradient_stops(
                start[0],
                start[1],
                end[0],
                end[1],
                stops.iter().copied(),
            ),
            Some(WebGl2Shader::Radial {
                center,
                radius,
                stops,
                ..
            }) => Paint::radial_gradient_stops(
                center[0],
                center[1],
                0.0,
                *radius,
                stops.iter().copied(),
            ),
            None => Paint::color(color(self.color)),
        };
        paint.set_fill_rule(match fill_rule {
            FillRule::EvenOdd => FemtovgFillRule::EvenOdd,
            FillRule::NonZero | FillRule::Clockwise => FemtovgFillRule::NonZero,
        });
        paint
    }
}

impl RenderPaint for WebGl2Paint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value.abs();
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value.abs();
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        match shader {
            Some(shader) => {
                self.shader = shader.as_any().downcast_ref::<WebGl2Shader>().cloned();
                self.invalid_shader = self.shader.as_ref().is_none_or(|shader| !shader.is_valid());
            }
            None => {
                self.shader = None;
                self.invalid_shader = false;
            }
        }
    }

    fn invalidate_stroke(&mut self) {}
}

struct WebGl2Image {
    width: u32,
    height: u32,
    pixels: Vec<RGBA8>,
    mesh_texture: RefCell<Option<web_sys::WebGlTexture>>,
    pending_mesh_texture_deletes: Weak<RefCell<Vec<web_sys::WebGlTexture>>>,
    owner: Weak<()>,
}

impl Drop for WebGl2Image {
    fn drop(&mut self) {
        if let Some(texture) = self.mesh_texture.get_mut().take() {
            if let Some(pending_deletes) = self.pending_mesh_texture_deletes.upgrade() {
                pending_deletes.borrow_mut().push(texture);
            }
        }
    }
}

fn webgl2_sampler_supported(sampler: ImageSampler) -> bool {
    sampler.wrap_x != ImageWrap::Mirror && sampler.wrap_y != ImageWrap::Mirror
}

impl RenderImage for WebGl2Image {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

struct WebGl2Buffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
    owner: Weak<()>,
    unmapped: bool,
}

impl RenderBuffer for WebGl2Buffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }

    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }

    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        self.unmapped = false;
        &mut self.bytes
    }

    fn unmap(&mut self) {
        self.unmapped = true;
    }
}

fn webgl2_buffer(buffer: &dyn RenderBuffer) -> Option<&WebGl2Buffer> {
    buffer.as_any().downcast_ref::<WebGl2Buffer>()
}

fn read_f32_pair(bytes: &[u8], index: usize) -> Option<[f32; 2]> {
    let offset = index.checked_mul(8)?;
    let x = f32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    let y = f32::from_le_bytes(bytes.get(offset + 4..offset + 8)?.try_into().ok()?);
    Some([x, y])
}

struct WebGl2MeshRenderer {
    gl: web_sys::WebGl2RenderingContext,
    program: web_sys::WebGlProgram,
    vertex_array: web_sys::WebGlVertexArrayObject,
    vertex_buffer: web_sys::WebGlBuffer,
    index_buffer: web_sys::WebGlBuffer,
    view_size: web_sys::WebGlUniformLocation,
    opacity: web_sys::WebGlUniformLocation,
    image: web_sys::WebGlUniformLocation,
    clip_enabled: web_sys::WebGlUniformLocation,
    clip_inverse: web_sys::WebGlUniformLocation,
    clip_rect: web_sys::WebGlUniformLocation,
}

struct WebGl2MeshDraw<'a> {
    vertices: &'a [u8],
    indices: &'a [u8],
    index_count: i32,
    texture: &'a web_sys::WebGlTexture,
    opacity: f32,
    clip: Option<(Mat2D, [f32; 4])>,
    width: u32,
    height: u32,
}

impl WebGl2MeshRenderer {
    fn new(element: &web_sys::HtmlCanvasElement) -> Result<Self, RendererError> {
        use wasm_bindgen::JsCast as _;
        use web_sys::WebGl2RenderingContext as Gl;

        let context = element
            .get_context("webgl2")
            .map_err(|error| webgl2_js_error("image mesh context", error))?
            .ok_or_else(|| RendererError::WebGl2("WebGL2 is unavailable".into()))?;
        let gl = context
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| RendererError::WebGl2("browser returned a non-WebGL2 context".into()))?;
        let vertex = compile_mesh_shader(
            &gl,
            Gl::VERTEX_SHADER,
            r#"#version 300 es
precision highp float;
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
uniform vec2 view_size;
out vec2 texture_uv;
void main() {
    vec2 clip = vec2(
        position.x / view_size.x * 2.0 - 1.0,
        1.0 - position.y / view_size.y * 2.0
    );
    gl_Position = vec4(clip, 0.0, 1.0);
    texture_uv = uv;
}
"#,
            "image mesh vertex",
        )?;
        let fragment = match compile_mesh_shader(
            &gl,
            Gl::FRAGMENT_SHADER,
            r#"#version 300 es
precision highp float;
uniform sampler2D image;
uniform float opacity;
uniform vec2 view_size;
uniform bool clip_enabled;
uniform mat3 clip_inverse;
uniform vec4 clip_rect;
in vec2 texture_uv;
layout(location = 0) out vec4 color;
void main() {
    if (clip_enabled) {
        vec2 canvas_position = vec2(gl_FragCoord.x, view_size.y - gl_FragCoord.y);
        vec2 clip_position = (clip_inverse * vec3(canvas_position, 1.0)).xy;
        if (clip_position.x < clip_rect.x || clip_position.y < clip_rect.y
            || clip_position.x >= clip_rect.z || clip_position.y >= clip_rect.w) {
            discard;
        }
    }
    vec4 sampled = texture(image, texture_uv);
    float alpha = sampled.a * opacity;
    color = vec4(sampled.rgb * opacity, alpha);
}
"#,
            "image mesh fragment",
        ) {
            Ok(shader) => shader,
            Err(error) => {
                gl.delete_shader(Some(&vertex));
                return Err(error);
            }
        };
        let program = gl
            .create_program()
            .ok_or_else(|| RendererError::WebGl2("failed to allocate image mesh program".into()))?;
        gl.attach_shader(&program, &vertex);
        gl.attach_shader(&program, &fragment);
        gl.link_program(&program);
        let linked = gl
            .get_program_parameter(&program, Gl::LINK_STATUS)
            .as_bool()
            .unwrap_or(false);
        let link_log = gl.get_program_info_log(&program).unwrap_or_default();
        gl.detach_shader(&program, &vertex);
        gl.detach_shader(&program, &fragment);
        gl.delete_shader(Some(&vertex));
        gl.delete_shader(Some(&fragment));
        if !linked {
            gl.delete_program(Some(&program));
            return Err(RendererError::WebGl2(format!(
                "image mesh shader program failed to link: {link_log}"
            )));
        }
        let uniform = |name| {
            gl.get_uniform_location(&program, name).ok_or_else(|| {
                RendererError::WebGl2(format!("linked image mesh shader omitted uniform '{name}'"))
            })
        };
        let view_size = uniform("view_size")?;
        let opacity = uniform("opacity")?;
        let image = uniform("image")?;
        let clip_enabled = uniform("clip_enabled")?;
        let clip_inverse = uniform("clip_inverse")?;
        let clip_rect = uniform("clip_rect")?;
        let vertex_array = gl
            .create_vertex_array()
            .ok_or_else(|| RendererError::WebGl2("failed to allocate image mesh VAO".into()))?;
        let vertex_buffer = gl.create_buffer().ok_or_else(|| {
            RendererError::WebGl2("failed to allocate image mesh vertex buffer".into())
        })?;
        let index_buffer = gl.create_buffer().ok_or_else(|| {
            RendererError::WebGl2("failed to allocate image mesh index buffer".into())
        })?;

        Ok(Self {
            gl,
            program,
            vertex_array,
            vertex_buffer,
            index_buffer,
            view_size,
            opacity,
            image,
            clip_enabled,
            clip_inverse,
            clip_rect,
        })
    }

    fn texture_for(
        &self,
        image: &WebGl2Image,
        sampler: ImageSampler,
    ) -> Result<web_sys::WebGlTexture, ()> {
        use web_sys::WebGl2RenderingContext as Gl;

        let wrap_x = match sampler.wrap_x {
            ImageWrap::Clamp => Gl::CLAMP_TO_EDGE,
            ImageWrap::Repeat => Gl::REPEAT,
            ImageWrap::Mirror => return Err(()),
        };
        let wrap_y = match sampler.wrap_y {
            ImageWrap::Clamp => Gl::CLAMP_TO_EDGE,
            ImageWrap::Repeat => Gl::REPEAT,
            ImageWrap::Mirror => return Err(()),
        };
        let filter = match sampler.filter {
            ImageFilter::Bilinear => Gl::LINEAR,
            ImageFilter::Nearest => Gl::NEAREST,
        };

        let existing = image.mesh_texture.borrow().as_ref().cloned();
        let is_new = existing.is_none();
        let texture = match existing {
            Some(texture) => texture,
            None => self.gl.create_texture().ok_or(())?,
        };
        self.gl.active_texture(Gl::TEXTURE0);
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, wrap_x as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, wrap_y as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, filter as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, filter as i32);
        if check_mesh_gl_error(&self.gl).is_err() {
            if is_new {
                self.gl.delete_texture(Some(&texture));
            }
            return Err(());
        }

        if is_new {
            self.gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);
            let bytes: &[u8] = bytemuck::cast_slice(&image.pixels);
            if self
                .gl
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    Gl::TEXTURE_2D,
                    0,
                    Gl::RGBA as i32,
                    image.width as i32,
                    image.height as i32,
                    0,
                    Gl::RGBA,
                    Gl::UNSIGNED_BYTE,
                    Some(bytes),
                )
                .is_err()
                || check_mesh_gl_error(&self.gl).is_err()
            {
                self.gl.delete_texture(Some(&texture));
                return Err(());
            }
            *image.mesh_texture.borrow_mut() = Some(texture.clone());
        }
        Ok(texture)
    }

    fn draw(&self, draw: WebGl2MeshDraw<'_>) -> Result<(), RendererError> {
        use web_sys::WebGl2RenderingContext as Gl;

        for _ in 0..16 {
            if self.gl.get_error() == Gl::NO_ERROR {
                break;
            }
        }
        self.gl.use_program(Some(&self.program));
        self.gl
            .viewport(0, 0, draw.width as i32, draw.height as i32);
        self.gl.disable(Gl::CULL_FACE);
        self.gl.disable(Gl::DEPTH_TEST);
        self.gl.disable(Gl::SCISSOR_TEST);
        self.gl.disable(Gl::STENCIL_TEST);
        self.gl.color_mask(true, true, true, true);
        self.gl.enable(Gl::BLEND);
        self.gl.blend_equation(Gl::FUNC_ADD);
        self.gl.blend_func_separate(
            Gl::ONE,
            Gl::ONE_MINUS_SRC_ALPHA,
            Gl::ONE,
            Gl::ONE_MINUS_SRC_ALPHA,
        );

        self.gl.bind_vertex_array(Some(&self.vertex_array));
        self.gl
            .bind_buffer(Gl::ARRAY_BUFFER, Some(&self.vertex_buffer));
        self.gl
            .buffer_data_with_u8_array(Gl::ARRAY_BUFFER, draw.vertices, Gl::DYNAMIC_DRAW);
        self.gl.enable_vertex_attrib_array(0);
        self.gl
            .vertex_attrib_pointer_with_i32(0, 2, Gl::FLOAT, false, 16, 0);
        self.gl.enable_vertex_attrib_array(1);
        self.gl
            .vertex_attrib_pointer_with_i32(1, 2, Gl::FLOAT, false, 16, 8);
        self.gl
            .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        self.gl
            .buffer_data_with_u8_array(Gl::ELEMENT_ARRAY_BUFFER, draw.indices, Gl::DYNAMIC_DRAW);

        self.gl
            .uniform2f(Some(&self.view_size), draw.width as f32, draw.height as f32);
        self.gl.uniform1f(Some(&self.opacity), draw.opacity);
        self.gl.uniform1i(Some(&self.image), 0);
        match draw.clip {
            Some((inverse, rect)) => {
                let [xx, yx, xy, yy, tx, ty] = inverse.0;
                self.gl.uniform1i(Some(&self.clip_enabled), 1);
                self.gl.uniform_matrix3fv_with_f32_array(
                    Some(&self.clip_inverse),
                    false,
                    &[xx, yx, 0.0, xy, yy, 0.0, tx, ty, 1.0],
                );
                self.gl
                    .uniform4f(Some(&self.clip_rect), rect[0], rect[1], rect[2], rect[3]);
            }
            None => self.gl.uniform1i(Some(&self.clip_enabled), 0),
        }
        self.gl.active_texture(Gl::TEXTURE0);
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(draw.texture));
        self.gl
            .draw_elements_with_i32(Gl::TRIANGLES, draw.index_count, Gl::UNSIGNED_SHORT, 0);
        check_mesh_gl_error(&self.gl)?;

        self.gl.bind_vertex_array(None);
        self.gl.bind_buffer(Gl::ARRAY_BUFFER, None);
        self.gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, None);
        self.gl.bind_texture(Gl::TEXTURE_2D, None);
        self.gl.use_program(None);
        Ok(())
    }
}

impl Drop for WebGl2MeshRenderer {
    fn drop(&mut self) {
        self.gl.delete_buffer(Some(&self.vertex_buffer));
        self.gl.delete_buffer(Some(&self.index_buffer));
        self.gl.delete_vertex_array(Some(&self.vertex_array));
        self.gl.delete_program(Some(&self.program));
    }
}

fn compile_mesh_shader(
    gl: &web_sys::WebGl2RenderingContext,
    kind: u32,
    source: &str,
    label: &str,
) -> Result<web_sys::WebGlShader, RendererError> {
    let shader = gl
        .create_shader(kind)
        .ok_or_else(|| RendererError::WebGl2(format!("failed to allocate {label} shader")))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, web_sys::WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        gl.delete_shader(Some(&shader));
        Err(RendererError::WebGl2(format!(
            "{label} shader failed to compile: {log}"
        )))
    }
}

fn check_mesh_gl_error(gl: &web_sys::WebGl2RenderingContext) -> Result<(), RendererError> {
    let code = gl.get_error();
    if code == web_sys::WebGl2RenderingContext::NO_ERROR {
        Ok(())
    } else {
        Err(RendererError::WebGl2(format!(
            "image mesh operation produced GL error 0x{code:04x}"
        )))
    }
}

fn webgl2_js_error(operation: &str, value: wasm_bindgen::JsValue) -> RendererError {
    RendererError::WebGl2(format!(
        "{operation} failed: {}",
        value.as_string().unwrap_or_else(|| format!("{value:?}"))
    ))
}

fn delete_pending_mesh_textures(
    renderer: &WebGl2MeshRenderer,
    pending: &RefCell<Vec<web_sys::WebGlTexture>>,
) {
    for texture in pending.borrow_mut().drain(..) {
        renderer.gl.delete_texture(Some(&texture));
    }
}

fn gradient_stops(colors: &[ColorInt], stops: &[f32]) -> Option<Vec<(f32, Color)>> {
    const HARD_STOP_EPSILON: f32 = 1.0 / 4096.0;
    if colors.len() != stops.len()
        || colors.is_empty()
        || stops
            .iter()
            .any(|stop| !stop.is_finite() || !(0.0..=1.0).contains(stop))
        || stops.windows(2).any(|pair| pair[0] > pair[1])
    {
        return None;
    }

    let first_color = color(colors[0]);
    let last_color = color(*colors.last()?);
    let first_stop = stops[0];
    if stops.iter().all(|stop| *stop == first_stop) {
        return Some(if first_stop <= HARD_STOP_EPSILON {
            vec![(0.0, last_color), (1.0, last_color)]
        } else if first_stop >= 1.0 - HARD_STOP_EPSILON {
            vec![
                (0.0, first_color),
                (1.0 - HARD_STOP_EPSILON, first_color),
                (1.0, last_color),
            ]
        } else {
            vec![
                (0.0, first_color),
                (first_stop - HARD_STOP_EPSILON, first_color),
                (first_stop, last_color),
                (1.0, last_color),
            ]
        });
    }

    Some(
        colors
            .iter()
            .zip(stops)
            .map(|(&value, &stop)| (stop, color(value)))
            .collect(),
    )
}

fn color(value: ColorInt) -> Color {
    Color::rgba(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
        (value >> 24) as u8,
    )
}

fn line_join(join: StrokeJoin) -> LineJoin {
    match join {
        StrokeJoin::Miter => LineJoin::Miter,
        StrokeJoin::Round => LineJoin::Round,
        StrokeJoin::Bevel => LineJoin::Bevel,
    }
}

fn line_cap(cap: StrokeCap) -> LineCap {
    match cap {
        StrokeCap::Butt => LineCap::Butt,
        StrokeCap::Round => LineCap::Round,
        StrokeCap::Square => LineCap::Square,
    }
}

fn path_rect(path: &RawPath) -> Option<[f32; 4]> {
    let points = match path.verbs() {
        [PathVerb::Move, PathVerb::Line, PathVerb::Line, PathVerb::Line, PathVerb::Close]
            if path.points().len() == 4 =>
        {
            path.points()
        }
        [PathVerb::Move, PathVerb::Line, PathVerb::Line, PathVerb::Line, PathVerb::Line, PathVerb::Close]
            if path.points().len() == 5 && path.points().first() == path.points().last() =>
        {
            &path.points()[..4]
        }
        _ => return None,
    };
    let [p0, p1, p2, p3] = points else {
        return None;
    };
    let is_rect = (p0.x == p3.x && p0.y == p1.y && p2.x == p1.x && p2.y == p3.y)
        || (p0.x == p1.x && p0.y == p3.y && p2.x == p3.x && p2.y == p1.y);
    is_rect.then_some([
        p0.x.min(p2.x),
        p0.y.min(p2.y),
        p0.x.max(p2.x),
        p0.y.max(p2.y),
    ])
}

fn path_is_axis_aligned_pixel_rect(path: &RawPath, transform: Mat2D) -> bool {
    let Some([left, top, right, bottom]) = path_rect(path) else {
        return false;
    };
    let [xx, yx, xy, yy, tx, ty] = transform.0;
    if yx.abs() > 1e-5 || xy.abs() > 1e-5 {
        return false;
    }
    [
        left * xx + tx,
        top * yy + ty,
        right * xx + tx,
        bottom * yy + ty,
    ]
    .into_iter()
    .all(|value| value.is_finite() && (value - value.round()).abs() <= 1e-5)
}

fn path_is_axis_aligned_pixel_rounded_rect(path: &RawPath, transform: Mat2D) -> bool {
    const ROUNDED_RECT_VERBS: [PathVerb; 10] = [
        PathVerb::Move,
        PathVerb::Cubic,
        PathVerb::Line,
        PathVerb::Cubic,
        PathVerb::Line,
        PathVerb::Cubic,
        PathVerb::Line,
        PathVerb::Cubic,
        PathVerb::Line,
        PathVerb::Close,
    ];
    if path.verbs() != ROUNDED_RECT_VERBS {
        return false;
    }
    let [xx, yx, xy, yy, tx, ty] = transform.0;
    if yx.abs() > 1e-5 || xy.abs() > 1e-5 {
        return false;
    }
    let mut left = f32::INFINITY;
    let mut top = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::NEG_INFINITY;
    for point in path.points() {
        left = left.min(point.x);
        top = top.min(point.y);
        right = right.max(point.x);
        bottom = bottom.max(point.y);
    }
    [
        left * xx + tx,
        top * yy + ty,
        right * xx + tx,
        bottom * yy + ty,
    ]
    .into_iter()
    .all(|value| value.is_finite() && (value - value.round()).abs() <= 1e-5)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FeatherImagePlan {
    left: f32,
    top: f32,
    logical_width: f32,
    logical_height: f32,
    width: usize,
    height: usize,
    scale: f32,
    sigma: f32,
}

fn feather_image_plan(
    path: &WebGl2Path,
    paint: &WebGl2Paint,
    transform: Mat2D,
) -> Option<FeatherImagePlan> {
    const FEATHER_STDDEVS: f32 = 3.0;
    // Rive caps a feather atlas to a 16-pixel radius and scales the result up
    // for larger blurs. Femtovg's WebGL Gaussian kernel is bounded as well, so
    // use the same resolution policy instead of silently truncating its blur.
    const MAX_FILTER_RADIUS: f32 = 16.0;
    const MIN_DEVICE_SIGMA: f32 = 1.0 / 3.0;

    if !paint.feather.is_finite() || paint.feather <= 0.0 {
        return None;
    }
    let matrix_scale = webgl2_max_matrix_scale(transform);
    if !matrix_scale.is_finite() || matrix_scale <= 0.0 {
        return None;
    }
    let authored_sigma = paint.feather * 0.5;
    let device_radius = authored_sigma * FEATHER_STDDEVS * matrix_scale;
    let atlas_scale = (MAX_FILTER_RADIUS / device_radius.max(MAX_FILTER_RADIUS)).min(1.0);
    let scale = matrix_scale * atlas_scale;
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let sigma = (authored_sigma * scale).max(MIN_DEVICE_SIGMA);

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for point in path.raw_path.points() {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    let stroke_outset = if paint.style == RenderPaintStyle::Stroke {
        let radius = paint.thickness * 0.5;
        if !radius.is_finite() {
            return None;
        }
        if paint.join == StrokeJoin::Miter {
            radius * 4.0
        } else if paint.cap == StrokeCap::Square {
            radius * std::f32::consts::SQRT_2
        } else {
            radius
        }
    } else {
        0.0
    };
    // The Gaussian target includes the complete +/-3 sigma Rive feather
    // domain, plus one source antialias pixel before filtering.
    let padding = stroke_outset + (sigma * FEATHER_STDDEVS + 1.0) / scale;
    let pixel_left = ((min_x - padding) * scale).floor();
    let pixel_top = ((min_y - padding) * scale).floor();
    let pixel_right = ((max_x + padding) * scale).ceil();
    let pixel_bottom = ((max_y + padding) * scale).ceil();
    let pixel_width = pixel_right - pixel_left;
    let pixel_height = pixel_bottom - pixel_top;
    if !pixel_left.is_finite()
        || !pixel_top.is_finite()
        || !pixel_width.is_finite()
        || !pixel_height.is_finite()
        || pixel_width < 1.0
        || pixel_height < 1.0
        || pixel_width > u32::MAX as f32
        || pixel_height > u32::MAX as f32
    {
        return None;
    }
    let width = pixel_width as usize;
    let height = pixel_height as usize;
    Some(FeatherImagePlan {
        left: pixel_left / scale,
        top: pixel_top / scale,
        logical_width: width as f32 / scale,
        logical_height: height as f32 / scale,
        width,
        height,
        scale,
        sigma,
    })
}

fn webgl2_max_matrix_scale(transform: Mat2D) -> f32 {
    let [xx, yx, xy, yy, _, _] = transform.0;
    if xy == 0.0 && yx == 0.0 {
        return xx.abs().max(yy.abs());
    }
    let a = xx * xx + xy * xy;
    let b = xx * yx + yy * xy;
    let c = yx * yx + yy * yy;
    let result = if b * b <= f32::EPSILON * f32::EPSILON {
        a.max(c)
    } else {
        (a + c) * 0.5 + ((a - c) * (a - c) + 4.0 * b * b).sqrt() * 0.5
    };
    if result.is_finite() {
        result.max(0.0).sqrt()
    } else {
        0.0
    }
}

fn clockwise_femtovg_path(raw_path: &RawPath, transform: Mat2D) -> Option<Path> {
    let authored_contours = draw::flatten_path(raw_path, Mat2D::IDENTITY);
    if clockwise_is_nonzero_compatible(&authored_contours) {
        // Ordinary Rive paths (including glyphs with nested counters) already
        // have the exact contour winding Femtovg's NonZero fill needs. Keep
        // their authored curves so the font outline is not replaced by a
        // triangle soup with visible internal antialias edges.
        return raw_path_to_femtovg(raw_path);
    }

    // Clockwise means positive winding only, while Femtovg exposes NonZero
    // and EvenOdd. Use the Rust planar triangulator only when a negative
    // contour contributes pixels outside a positive contour (or a contour is
    // self-cancelling), where NonZero would paint pixels Rive leaves empty.
    let inverse = invert(transform)?;
    let mut linear_path = RawPath::new();
    for contour in draw::flatten_path(raw_path, transform) {
        let mut points = contour.points.into_iter();
        let first = inverse.transform_point(points.next()?);
        linear_path.move_to(first.x, first.y);
        for point in points {
            let point = inverse.transform_point(point);
            linear_path.line_to(point.x, point.y);
        }
        linear_path.close();
    }

    let triangulator = gr_triangulator::InnerFanTriangulator::new(
        &linear_path,
        transform,
        gr_triangulator::SweepDirection::Vertical,
        FillRule::NonZero,
    );
    let triangles = triangulator.triangles(0, gr_triangulator::WindingFaces::Positive);
    let mut path = Path::new();
    for triangle in triangles.chunks_exact(3) {
        path.move_to(triangle[0].point[0], triangle[0].point[1]);
        path.line_to(triangle[1].point[0], triangle[1].point[1]);
        path.line_to(triangle[2].point[0], triangle[2].point[1]);
        path.close();
    }
    Some(path)
}

fn raw_path_to_femtovg(raw_path: &RawPath) -> Option<Path> {
    let mut path = Path::new();
    let mut point = 0;
    for verb in raw_path.verbs() {
        match verb {
            PathVerb::Move => {
                let value = *raw_path.points().get(point)?;
                point += 1;
                path.move_to(value.x, value.y);
            }
            PathVerb::Line => {
                let value = *raw_path.points().get(point)?;
                point += 1;
                path.line_to(value.x, value.y);
            }
            PathVerb::Quad => {
                let control = *raw_path.points().get(point)?;
                let end = *raw_path.points().get(point + 1)?;
                point += 2;
                path.quad_to(control.x, control.y, end.x, end.y);
            }
            PathVerb::Cubic => {
                let first = *raw_path.points().get(point)?;
                let second = *raw_path.points().get(point + 1)?;
                let end = *raw_path.points().get(point + 2)?;
                point += 3;
                path.bezier_to(first.x, first.y, second.x, second.y, end.x, end.y);
            }
            PathVerb::Close => path.close(),
        }
    }
    (point == raw_path.points().len()).then_some(path)
}

fn clockwise_is_nonzero_compatible(contours: &[draw::Contour]) -> bool {
    let signed_area = |contour: &draw::Contour| {
        contour
            .points
            .iter()
            .zip(contour.points.iter().cycle().skip(1))
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f32>()
    };
    let positive = contours
        .iter()
        .filter(|contour| signed_area(contour) > f32::EPSILON)
        .collect::<Vec<_>>();
    contours.iter().all(|contour| {
        let area = signed_area(contour);
        if area > f32::EPSILON {
            return true;
        }
        if area.abs() <= f32::EPSILON || positive.is_empty() {
            return false;
        }
        contour
            .points
            .iter()
            .zip(contour.points.iter().cycle().skip(1))
            .flat_map(|(a, b)| [*a, Vec2D::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)])
            .all(|point| {
                positive
                    .iter()
                    .any(|candidate| contour_contains_point(candidate, point))
            })
    })
}

fn contour_contains_point(contour: &draw::Contour, point: Vec2D) -> bool {
    let mut winding = 0i32;
    for (a, b) in contour
        .points
        .iter()
        .zip(contour.points.iter().cycle().skip(1))
    {
        let cross = (b.x - a.x) * (point.y - a.y) - (point.x - a.x) * (b.y - a.y);
        let on_segment = cross.abs() <= 1e-5
            && point.x >= a.x.min(b.x) - 1e-5
            && point.x <= a.x.max(b.x) + 1e-5
            && point.y >= a.y.min(b.y) - 1e-5
            && point.y <= a.y.max(b.y) + 1e-5;
        if on_segment {
            return true;
        }
        if a.y <= point.y {
            if b.y > point.y && cross > 0.0 {
                winding += 1;
            }
        } else if b.y <= point.y && cross < 0.0 {
            winding -= 1;
        }
    }
    winding != 0
}
