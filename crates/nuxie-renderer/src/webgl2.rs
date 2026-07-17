use super::{
    decode_image_rgba, draw, gr_triangulator, invert, multiply, transform_rect_to_new_space,
    RendererError,
};
use femtovg::renderer::OpenGl;
use femtovg::{
    Canvas, Color, FillRule as FemtovgFillRule, ImageFlags, ImageId, ImageSource, LineCap,
    LineJoin, Paint, Path, Transform2D,
};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, PathVerb,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
};
use rgb::RGBA8;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

type WebGl2Canvas = Canvas<OpenGl>;

/// WebGL2 renderer factory bound to one HTML canvas.
///
/// The compatibility path supports solid and gradient paths, strokes,
/// clockwise/nonzero/even-odd fills, transformed rectangular clips, and
/// linearly sampled images. Unsupported capabilities fail when the frame is
/// finished instead of rendering partial output.
pub struct WebGl2Factory {
    canvas: Rc<RefCell<WebGl2Canvas>>,
    frame_active: Rc<Cell<bool>>,
    poisoned: Rc<Cell<bool>>,
    pending_image_deletes: Rc<RefCell<Vec<ImageId>>>,
    owner: Rc<()>,
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
            canvas: Rc::new(RefCell::new(canvas)),
            frame_active: Rc::new(Cell::new(false)),
            poisoned: Rc::new(Cell::new(false)),
            pending_image_deletes: Rc::new(RefCell::new(Vec::new())),
            owner: Rc::new(()),
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
        delete_pending_images(&mut canvas, &self.pending_image_deletes);
        canvas.reset();
        canvas.reset_transform();
        canvas.set_global_alpha(1.0);
        canvas.clear_rect(0, 0, self.width, self.height, color(clear_color));
        drop(canvas);
        Ok(WebGl2Frame {
            canvas: Rc::clone(&self.canvas),
            frame_active: Rc::clone(&self.frame_active),
            poisoned: Rc::clone(&self.poisoned),
            pending_image_deletes: Rc::clone(&self.pending_image_deletes),
            owner: Rc::clone(&self.owner),
            stack: Vec::new(),
            state: WebGl2State::default(),
            clear_color,
            width: self.width,
            height: self.height,
            unsupported: None,
            finished: false,
        })
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
        let pixels = bytes
            .chunks_exact(4)
            .map(|pixel| RGBA8 {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
                a: pixel[3],
            })
            .collect::<Vec<_>>();
        let image = imgref::Img::new(pixels.as_slice(), width as usize, height as usize);
        let id = self
            .canvas
            .borrow_mut()
            .create_image(ImageSource::Rgba(image), ImageFlags::empty())
            .map_err(|_| ImageDecodeError)?;
        Ok(Box::new(WebGl2Image {
            width,
            height,
            id: Some(id),
            pending_deletes: Rc::downgrade(&self.pending_image_deletes),
            owner: Rc::downgrade(&self.owner),
        }))
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
    pending_image_deletes: Rc<RefCell<Vec<ImageId>>>,
    owner: Rc<()>,
    stack: Vec<WebGl2State>,
    state: WebGl2State,
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
}

impl Default for WebGl2State {
    fn default() -> Self {
        Self {
            transform: Mat2D::IDENTITY,
            opacity: 1.0,
            clip_matrix: None,
        }
    }
}

impl WebGl2Frame {
    /// Flushes WebGL2 work and returns the canvas contents as RGBA pixels.
    ///
    /// If replay requested an unsupported capability, this clears the queued
    /// frame and returns a named [`RendererError::Unsupported`] error.
    pub fn finish(mut self) -> Result<Vec<u8>, RendererError> {
        self.finished = true;
        self.frame_active.set(false);
        self.unwind_state_stack();
        let mut canvas = self.canvas.borrow_mut();
        if let Some(feature) = self.unsupported {
            canvas.reset();
            canvas.clear_rect(0, 0, self.width, self.height, color(self.clear_color));
            canvas.flush();
            delete_pending_images(&mut canvas, &self.pending_image_deletes);
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
        delete_pending_images(&mut canvas, &self.pending_image_deletes);
        Ok(pixels)
    }

    fn reject(&mut self, feature: &'static str) {
        self.unsupported.get_or_insert(feature);
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
        if paint.feather != 0.0 {
            self.reject("WebGL2 feathered path rendering");
            return;
        }
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
        let Some([left, top, right, bottom]) = path_rect(&path.raw_path) else {
            self.reject("non-rectangular WebGL2 clip paths");
            return;
        };
        if self.state.clip_matrix.is_some_and(|matrix| {
            transform_rect_to_new_space([left, top, right, bottom], self.state.transform, matrix)
                .is_none()
        }) {
            self.reject("incompatible transformed WebGL2 clip rectangles");
            return;
        }
        self.canvas
            .borrow_mut()
            .intersect_scissor(left, top, right - left, bottom - top);
        self.state.clip_matrix = Some(self.state.transform);
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
        if sampler != ImageSampler::LINEAR_CLAMP {
            self.reject("WebGL2 non-default image samplers");
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
        let Some(id) = image.id else {
            self.reject("WebGL2 image decode or upload failed");
            return;
        };
        let mut path = Path::new();
        path.rect(0.0, 0.0, image.width as f32, image.height as f32);
        let paint = Paint::image(
            id,
            0.0,
            0.0,
            image.width as f32,
            image.height as f32,
            0.0,
            opacity.max(0.0),
        );
        self.canvas.borrow_mut().fill_path(&path, &paint);
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
        if image.is_none() {
            return;
        }
        self.reject("WebGL2 image meshes");
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
        let mut path = Path::new();
        let mut point = 0;
        for verb in self.raw_path.verbs() {
            match verb {
                PathVerb::Move => {
                    let value = *self.raw_path.points().get(point)?;
                    point += 1;
                    path.move_to(value.x, value.y);
                }
                PathVerb::Line => {
                    let value = *self.raw_path.points().get(point)?;
                    point += 1;
                    path.line_to(value.x, value.y);
                }
                PathVerb::Quad => {
                    let control = *self.raw_path.points().get(point)?;
                    let end = *self.raw_path.points().get(point + 1)?;
                    point += 2;
                    path.quad_to(control.x, control.y, end.x, end.y);
                }
                PathVerb::Cubic => {
                    let first = *self.raw_path.points().get(point)?;
                    let second = *self.raw_path.points().get(point + 1)?;
                    let end = *self.raw_path.points().get(point + 2)?;
                    point += 3;
                    path.bezier_to(first.x, first.y, second.x, second.y, end.x, end.y);
                }
                PathVerb::Close => path.close(),
            }
        }
        Some(path)
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
    id: Option<ImageId>,
    pending_deletes: Weak<RefCell<Vec<ImageId>>>,
    owner: Weak<()>,
}

impl Drop for WebGl2Image {
    fn drop(&mut self) {
        let (Some(id), Some(pending_deletes)) = (self.id.take(), self.pending_deletes.upgrade())
        else {
            return;
        };
        pending_deletes.borrow_mut().push(id);
    }
}

fn delete_pending_images(canvas: &mut WebGl2Canvas, pending_image_deletes: &RefCell<Vec<ImageId>>) {
    for id in pending_image_deletes.borrow_mut().drain(..) {
        canvas.delete_image(id);
    }
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
        &mut self.bytes
    }

    fn unmap(&mut self) {}
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

fn clockwise_femtovg_path(raw_path: &RawPath, transform: Mat2D) -> Option<Path> {
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
