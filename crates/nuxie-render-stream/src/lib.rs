//! Typed parsing and renderer-neutral replay for `rive-golden-stream-v1`.

use nuxie_render_api::{
    Aabb, BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageFilter, ImageSampler,
    ImageWrap, Mat2D, RawPath, RenderBufferFlags, RenderBufferType, RenderPaintStyle, Renderer,
    StrokeCap, StrokeJoin,
};
mod opacity;
mod opacity_canvas;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderStream {
    pub frame_size: Option<(u32, u32)>,
    pub clear_color: Option<ColorInt>,
    pub resources: Vec<Resource>,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resource {
    TiledPremultipliedLinearGradient {
        id: u64, start: (f32, f32), end: (f32, f32), tile: [f32; 4],
        stops: Vec<GradientStop>,
    },
    PremultipliedLinearGradient {
        id: u64,
        start: (f32, f32),
        end: (f32, f32),
        stops: Vec<GradientStop>,
    },
    LinearGradient {
        id: u64,
        start: (f32, f32),
        end: (f32, f32),
        stops: Vec<GradientStop>,
    },
    RadialGradient {
        id: u64,
        center: (f32, f32),
        radius: f32,
        stops: Vec<GradientStop>,
    },
    Image {
        id: u64,
        data: Vec<u8>,
    },
    Buffer {
        id: u64,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: ColorInt,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Isolate subsequent draws; requires preparation before the parent frame opens.
    BeginOpacity(f32),
    EndOpacity,
    Save,
    Restore,
    Transform(Mat2D),
    DrawPath {
        path: Path,
        paint: Paint,
    },
    ClipPath(Path),
    ClipOutRect(Aabb),
    ClipAxis { horizontal: bool, min: f32, max: f32, local: Option<Mat2D> },
    DrawImage {
        image: u64,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    },
    DrawImageMesh {
        image: u64,
        sampler: ImageSampler,
        vertices: u64,
        uvs: u64,
        indices: u64,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    },
    ModulateOpacity(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub fill_rule: FillRule,
    pub raw_path: RawPath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    pub style: RenderPaintStyle,
    pub color: ColorInt,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
    pub blend_mode: BlendMode,
    pub shader: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamError {
    line: usize,
    message: String,
}

impl StreamError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "render stream line {}: {}", self.line, self.message)
    }
}

impl Error for StreamError {}

impl RenderStream {
    pub fn parse(input: &str) -> Result<Self, StreamError> {
        let mut lines = input.lines().enumerate();
        match lines.next() {
            Some((_, "rive-golden-stream-v1")) => {}
            Some((_, header)) => return Err(StreamError::new(1, format!("bad header `{header}`"))),
            None => return Err(StreamError::new(1, "empty stream")),
        }

        let mut frame_size = None;
        let mut clear_color = None;
        let mut resources = Vec::new();
        let mut frames = Vec::new();
        let mut commands = Vec::new();
        let mut buffers: HashMap<u64, (RenderBufferType, RenderBufferFlags, usize)> =
            HashMap::new();

        for (zero_line, line) in lines {
            let line_number = zero_line + 1;
            if line.is_empty()
                || line.starts_with("source ")
                || line.starts_with("sample ")
                || line.starts_with("input ")
                || line.starts_with("makeRenderPaint ")
                || line.starts_with("makeRenderPath ")
                || line.starts_with("makeEmptyRenderPath ")
            {
                continue;
            }
            if line == "frame" {
                frames.push(Frame {
                    commands: std::mem::take(&mut commands),
                });
                continue;
            }
            if line == "save" {
                commands.push(Command::Save);
                continue;
            }
            if let Some(value) = line.strip_prefix("beginOpacity opacity=") {
                commands.push(Command::BeginOpacity(parse(value, line_number, "group opacity")?));
                continue;
            }
            if line == "endOpacity" {
                commands.push(Command::EndOpacity);
                continue;
            }
            if line == "restore" {
                commands.push(Command::Restore);
                continue;
            }
            if let Some(value) = line.strip_prefix("frameSize ") {
                frame_size = Some((
                    parse_field(value, "width", line_number)?,
                    parse_field(value, "height", line_number)?,
                ));
                continue;
            }
            if let Some(value) = line.strip_prefix("clearColor value=") {
                clear_color = Some(parse_hex_u32(value, line_number)?);
                continue;
            }
            if let Some(value) = line.strip_prefix("transform matrix=") {
                commands.push(Command::Transform(Mat2D(parse_array6(value, line_number)?)));
                continue;
            }
            if let Some(value) = line.strip_prefix("modulateOpacity opacity=") {
                commands.push(Command::ModulateOpacity(parse(
                    value,
                    line_number,
                    "opacity",
                )?));
                continue;
            }
            if let Some(value) = line.strip_prefix("clipAxis axis=") {
                let (value, local) = if let Some((value, matrix)) = value.split_once(" matrix=") {
                    let matrix = Mat2D(parse_array6(matrix, line_number)?);
                    if matrix.0.iter().any(|v| !v.is_finite()) {
                        return Err(StreamError::new(line_number, "axis clip matrix must be finite"));
                    }
                    (value, Some(matrix))
                } else { (value, None) };
                let (axis, range) = value.split_once(" range=")
                    .ok_or_else(|| StreamError::new(line_number, "bad axis clip"))?;
                let horizontal = match axis {
                    "x" => true,
                    "y" => false,
                    _ => return Err(StreamError::new(line_number, "axis must be x or y")),
                };
                let range = range.strip_prefix('[').and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| StreamError::new(line_number, "bad axis clip range"))?;
                let values = range.split(',').map(|v| parse::<f32>(v,line_number,"axis clip edge"))
                    .collect::<Result<Vec<_>,_>>()?;
                let [min,max]: [f32;2] = values.try_into()
                    .map_err(|_| StreamError::new(line_number,"axis clip needs two edges"))?;
                if !min.is_finite() || !max.is_finite() {
                    return Err(StreamError::new(line_number,"axis clip must be finite"));
                }
                commands.push(Command::ClipAxis {horizontal,min,max,local});
                continue;
            }
            if let Some(value) = line.strip_prefix("clipOutRect rect=") {
                let values = value
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| StreamError::new(line_number, "bad clip rectangle"))?
                    .split(',')
                    .map(|v| parse::<f32>(v, line_number, "clip edge"))
                    .collect::<Result<Vec<_>, _>>()?;
                let [left, top, right, bottom]: [f32; 4] = values.try_into().map_err(|_| {
                    StreamError::new(line_number, "clip rectangle needs four edges")
                })?;
                if [left, top, right, bottom].iter().any(|v| !v.is_finite()) {
                    return Err(StreamError::new(
                        line_number,
                        "clip rectangle must be finite",
                    ));
                }
                commands.push(Command::ClipOutRect(Aabb::new(left, top, right, bottom)));
                continue;
            }
            if let Some(value) = line.strip_prefix("clipPath path=") {
                commands.push(Command::ClipPath(parse_path(value, line_number)?));
                continue;
            }
            if let Some(value) = line.strip_prefix("drawPath path=") {
                let split = value
                    .find(" paint=")
                    .ok_or_else(|| StreamError::new(line_number, "drawPath has no paint"))?;
                commands.push(Command::DrawPath {
                    path: parse_path(&value[..split], line_number)?,
                    paint: parse_paint(&value[split + 7..], line_number)?,
                });
                continue;
            }
            if let Some(value) = line.strip_prefix("makeTiledPremultipliedLinearGradient ") {
                resources.push(parse_tiled_premultiplied_linear_gradient(value, line_number)?);
                continue;
            }
            if let Some(value) = line.strip_prefix("makePremultipliedLinearGradient ") {
                let Resource::LinearGradient { id, start, end, stops } = parse_linear_gradient(value, line_number)? else { unreachable!() };
                resources.push(Resource::PremultipliedLinearGradient { id, start, end, stops });
                continue;
            }
            if let Some(value) = line.strip_prefix("makeLinearGradient ") {
                resources.push(parse_linear_gradient(value, line_number)?);
                continue;
            }
            if let Some(value) = line.strip_prefix("makeRadialGradient ") {
                resources.push(parse_radial_gradient(value, line_number)?);
                continue;
            }
            if let Some(value) = line.strip_prefix("decodeImage ") {
                resources.push(Resource::Image {
                    id: parse_field(value, "id", line_number)?,
                    data: parse_hex(field(value, "data", line_number)?, line_number)?,
                });
                continue;
            }
            if let Some(value) = line.strip_prefix("makeRenderBuffer ") {
                let id = parse_field(value, "id", line_number)?;
                buffers.insert(
                    id,
                    (
                        parse_buffer_type(parse_field(value, "type", line_number)?, line_number)?,
                        parse_buffer_flags(parse_field(value, "flags", line_number)?, line_number)?,
                        parse_field(value, "size", line_number)?,
                    ),
                );
                continue;
            }
            if let Some(value) = line.strip_prefix("bufferData ") {
                let id = parse_field(value, "id", line_number)?;
                let (buffer_type, flags, size) = buffers.remove(&id).ok_or_else(|| {
                    StreamError::new(
                        line_number,
                        format!("bufferData references unknown id {id}"),
                    )
                })?;
                let data = parse_hex(field(value, "data", line_number)?, line_number)?;
                if data.len() != size {
                    return Err(StreamError::new(
                        line_number,
                        format!("buffer {id} has {} bytes, expected {size}", data.len()),
                    ));
                }
                resources.push(Resource::Buffer {
                    id,
                    buffer_type,
                    flags,
                    size,
                    data,
                });
                continue;
            }
            if let Some(value) = line.strip_prefix("drawImage image=") {
                commands.push(parse_draw_image(value, line_number)?);
                continue;
            }
            if let Some(value) = line.strip_prefix("drawImageMesh image=") {
                commands.push(parse_draw_image_mesh(value, line_number)?);
                continue;
            }
            return Err(StreamError::new(
                line_number,
                format!("unsupported command `{line}`"),
            ));
        }
        if !commands.is_empty() {
            frames.push(Frame { commands });
        }
        Ok(Self {
            frame_size,
            clear_color,
            resources,
            frames,
        })
    }

    pub fn replay_frame(
        &self,
        frame_index: usize,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
    ) -> Result<(), ReplayError> {
        let frame = self.frames.get(frame_index)
            .ok_or(ReplayError::MissingFrame(frame_index))?;
        // Never open an offscreen frame while the caller's frame is active.
        // Validate the entire group structure before allocating or painting.
        let groups = opacity::plan(frame)?;
        if !groups.groups.is_empty() {
            return Err(ReplayError::UnsupportedOperation("prepared opacity groups"));
        }
        let resources = LoadedResources::new(&self.resources, factory)?;
        for command in &frame.commands {
            resources.execute(command, factory, renderer)?;
        }
        Ok(())
    }
}

struct LoadedResources {
    shaders: HashMap<u64, Box<dyn nuxie_render_api::RenderShader>>,
    images: HashMap<u64, Box<dyn nuxie_render_api::RenderImage>>,
    buffers: HashMap<u64, Box<dyn nuxie_render_api::RenderBuffer>>,
}

impl LoadedResources {
    fn new(resources: &[Resource], factory: &mut dyn Factory) -> Result<Self, ReplayError> {
        let mut shaders = HashMap::new();
        let mut images = HashMap::new();
        let mut buffers = HashMap::new();
        for resource in resources {
            match resource {
                Resource::TiledPremultipliedLinearGradient { id, start, end, tile, stops } => {
                    let (colors, offsets) = split_stops(stops);
                    let shader = factory.make_tiled_premultiplied_linear_gradient(
                        start.0, start.1, end.0, end.1, *tile, &colors, &offsets)
                        .ok_or(ReplayError::UnsupportedOperation("tiled premultiplied linear gradients"))?;
                    shaders.insert(*id, shader);
                }
                Resource::PremultipliedLinearGradient { id, start, end, stops } => {
                    let (colors, offsets) = split_stops(stops);
                    let shader = factory.make_premultiplied_linear_gradient(start.0, start.1, end.0, end.1, &colors, &offsets)
                        .ok_or(ReplayError::UnsupportedOperation("premultiplied linear gradients"))?;
                    shaders.insert(*id, shader);
                }
                Resource::LinearGradient {
                    id,
                    start,
                    end,
                    stops,
                } => {
                    let (colors, offsets) = split_stops(stops);
                    shaders.insert(
                        *id,
                        factory.make_linear_gradient(
                            start.0, start.1, end.0, end.1, &colors, &offsets,
                        ),
                    );
                }
                Resource::RadialGradient {
                    id,
                    center,
                    radius,
                    stops,
                } => {
                    let (colors, offsets) = split_stops(stops);
                    shaders.insert(
                        *id,
                        factory
                            .make_radial_gradient(center.0, center.1, *radius, &colors, &offsets),
                    );
                }
                Resource::Image { id, data } => {
                    let image = factory
                        .decode_image(data)
                        .map_err(|source| ReplayError::ImageDecode { id: *id, source })?;
                    images.insert(*id, image);
                }
                Resource::Buffer {
                    id,
                    buffer_type,
                    flags,
                    size,
                    data,
                } => {
                    let mut buffer = factory.make_render_buffer(*buffer_type, *flags, *size);
                    buffer.map_mut().copy_from_slice(data);
                    buffer.unmap();
                    buffers.insert(*id, buffer);
                }
            }
        }

        Ok(Self { shaders, images, buffers })
    }

    fn execute(&self, command: &Command, factory: &mut dyn Factory, renderer: &mut dyn Renderer) -> Result<(), ReplayError> {
            match command {
                Command::BeginOpacity(_) | Command::EndOpacity => return Err(ReplayError::UnsupportedOperation("prepared opacity groups")),
                Command::Save => renderer.save(),
                Command::Restore => renderer.restore(),
                Command::Transform(matrix) => renderer.transform(*matrix),
                Command::ClipAxis {horizontal,min,max,local} => {
                    let accepted = match local {
                        Some(matrix) => renderer.clip_axis_transformed(*horizontal,*min,*max,*matrix),
                        None => renderer.clip_axis(*horizontal,*min,*max),
                    };
                    if !accepted {
                        return Err(ReplayError::UnsupportedOperation("clipAxis"));
                    }
                }
                Command::ClipOutRect(rect) => {
                    if !renderer.clip_out_rect(*rect) {
                        return Err(ReplayError::UnsupportedOperation("clipOutRect"));
                    }
                }
                Command::ClipPath(path) => {
                    let path = factory.make_render_path(path.raw_path.clone(), path.fill_rule);
                    renderer.clip_path(path.as_ref());
                }
                Command::DrawPath { path, paint } => {
                    let path = factory.make_render_path(path.raw_path.clone(), path.fill_rule);
                    let mut render_paint = factory.make_render_paint();
                    render_paint.style(paint.style);
                    render_paint.color(paint.color);
                    render_paint.thickness(paint.thickness);
                    render_paint.join(paint.join);
                    render_paint.cap(paint.cap);
                    render_paint.feather(paint.feather);
                    render_paint.blend_mode(paint.blend_mode);
                    if paint.shader != 0 {
                        let shader = self.shaders
                            .get(&paint.shader)
                            .ok_or(ReplayError::MissingResource("shader", paint.shader))?;
                        render_paint.shader(Some(shader.as_ref()));
                    }
                    renderer.draw_path(path.as_ref(), render_paint.as_ref());
                }
                Command::DrawImage {
                    image,
                    sampler,
                    blend_mode,
                    opacity,
                } => renderer.draw_image(
                    resource_ref(&self.images, "image", *image)?,
                    *sampler,
                    *blend_mode,
                    *opacity,
                ),
                Command::DrawImageMesh {
                    image,
                    sampler,
                    vertices,
                    uvs,
                    indices,
                    vertex_count,
                    index_count,
                    blend_mode,
                    opacity,
                } => renderer.draw_image_mesh(
                    resource_ref(&self.images, "image", *image)?,
                    *sampler,
                    buffer_ref(&self.buffers, *vertices)?,
                    buffer_ref(&self.buffers, *uvs)?,
                    buffer_ref(&self.buffers, *indices)?,
                    *vertex_count,
                    *index_count,
                    *blend_mode,
                    *opacity,
                ),
                Command::ModulateOpacity(opacity) => renderer.modulate_opacity(*opacity),
            }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidOpacityGroup { command: usize, reason: &'static str },
    Canvas(nuxie_render_api::RenderCanvasError),
    UnsupportedOperation(&'static str),
    MissingFrame(usize),
    MissingResource(&'static str, u64),
    ImageDecode { id: u64, source: ImageDecodeError },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canvas(error) => write!(f, "opacity canvas: {error}"),
            Self::InvalidOpacityGroup { command, reason } => write!(f, "opacity group at command {command}: {reason}"),
            Self::UnsupportedOperation(name) => write!(f, "renderer does not support {name}"),
            Self::MissingFrame(index) => write!(f, "render stream has no frame {index}"),
            Self::MissingResource(kind, id) => write!(f, "missing {kind} resource {id}"),
            Self::ImageDecode { id, .. } => write!(f, "failed to decode image resource {id}"),
        }
    }
}

impl Error for ReplayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ImageDecode { source, .. } => Some(source),
            Self::Canvas(error) => Some(error),
            Self::InvalidOpacityGroup { .. } | Self::MissingFrame(_) | Self::MissingResource(_, _) | Self::UnsupportedOperation(_) => {
                None
            }
        }
    }
}

fn split_stops(stops: &[GradientStop]) -> (Vec<ColorInt>, Vec<f32>) {
    stops.iter().map(|stop| (stop.color, stop.offset)).unzip()
}

fn resource_ref<'a, T: ?Sized>(
    resources: &'a HashMap<u64, Box<T>>,
    kind: &'static str,
    id: u64,
) -> Result<Option<&'a T>, ReplayError> {
    if id == 0 {
        Ok(None)
    } else {
        resources
            .get(&id)
            .map(|resource| Some(resource.as_ref()))
            .ok_or(ReplayError::MissingResource(kind, id))
    }
}

fn buffer_ref(
    resources: &HashMap<u64, Box<dyn nuxie_render_api::RenderBuffer>>,
    id: u64,
) -> Result<Option<&dyn nuxie_render_api::RenderBuffer>, ReplayError> {
    resource_ref(resources, "buffer", id)
}

fn parse_tiled_premultiplied_linear_gradient(value: &str, line: usize) -> Result<Resource, StreamError> {
    let Resource::LinearGradient { id, start, end, stops } = parse_linear_gradient(value, line)? else { unreachable!() };
    let text = field(value, "tile", line)?;
    let components = text.strip_prefix('(').and_then(|v| v.strip_suffix(')'))
        .ok_or_else(|| StreamError::new(line, "tile must be a parenthesized rectangle"))?
        .split(',').map(|v| parse::<f32>(v, line, "tile component"))
        .collect::<Result<Vec<_>, _>>()?;
    let tile: [f32; 4] = components.try_into()
        .map_err(|_| StreamError::new(line, "tile needs origin x, origin y, width and height"))?;
    if !nuxie_render_api::css_gradient_tile_is_representable(tile)
        || ![start.0, start.1, end.0, end.1].into_iter().all(f32::is_finite)
        || stops.len() < 2
        || !stops.iter().all(|s| s.offset.is_finite() && (0. ..=1.).contains(&s.offset))
        || !stops.windows(2).all(|pair| pair[0].offset <= pair[1].offset) {
        return Err(StreamError::new(line, "invalid tiled premultiplied linear gradient"));
    }
    Ok(Resource::TiledPremultipliedLinearGradient { id, start, end, tile, stops })
}

fn parse_linear_gradient(value: &str, line: usize) -> Result<Resource, StreamError> {
    Ok(Resource::LinearGradient {
        id: parse_field(value, "id", line)?,
        start: parse_pair(field(value, "start", line)?, line)?,
        end: parse_pair(field(value, "end", line)?, line)?,
        stops: parse_stops(field(value, "stops", line)?, line)?,
    })
}

fn parse_radial_gradient(value: &str, line: usize) -> Result<Resource, StreamError> {
    Ok(Resource::RadialGradient {
        id: parse_field(value, "id", line)?,
        center: parse_pair(field(value, "center", line)?, line)?,
        radius: parse_field(value, "radius", line)?,
        stops: parse_stops(field(value, "stops", line)?, line)?,
    })
}

fn parse_draw_image(value: &str, line: usize) -> Result<Command, StreamError> {
    Ok(Command::DrawImage {
        image: parse_prefix_value(value, line, "image")?,
        sampler: parse_sampler(field(value, "sampler", line)?, line)?,
        blend_mode: parse_blend(parse_field(value, "blendMode", line)?, line)?,
        opacity: parse_field(value, "opacity", line)?,
    })
}

fn parse_draw_image_mesh(value: &str, line: usize) -> Result<Command, StreamError> {
    Ok(Command::DrawImageMesh {
        image: parse_prefix_value(value, line, "image")?,
        sampler: parse_sampler(field(value, "sampler", line)?, line)?,
        vertices: parse_field(value, "vertices", line)?,
        uvs: parse_field(value, "uvs", line)?,
        indices: parse_field(value, "indices", line)?,
        vertex_count: parse_field(value, "vertexCount", line)?,
        index_count: parse_field(value, "indexCount", line)?,
        blend_mode: parse_blend(parse_field(value, "blendMode", line)?, line)?,
        opacity: parse_field(value, "opacity", line)?,
    })
}

fn parse_path(value: &str, line: usize) -> Result<Path, StreamError> {
    let fill_rule = parse_fill_rule(parse_field(value, "fillRule", line)?, line)?;
    let path_start = value
        .find("path={verbs=[")
        .ok_or_else(|| StreamError::new(line, "path snapshot has no raw path"))?
        + "path=".len();
    let raw = &value[path_start..value.len().saturating_sub(1)];
    let verb_start = raw
        .strip_prefix("{verbs=[")
        .ok_or_else(|| StreamError::new(line, "bad raw path"))?;
    let split = verb_start
        .find("],points=[")
        .ok_or_else(|| StreamError::new(line, "bad raw path points"))?;
    let verbs = &verb_start[..split];
    let points = verb_start[split + 10..]
        .strip_suffix("}")
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| StreamError::new(line, "bad raw path suffix"))?;
    let mut raw_path = RawPath::new();
    let mut parsed_points = parse_pairs(points, line)?.into_iter();
    if !verbs.is_empty() {
        for verb in verbs.split(',') {
            match verb {
                "move" => {
                    let point = next_point(&mut parsed_points, line, verb)?;
                    raw_path.move_to(point.0, point.1);
                }
                "line" => {
                    let point = next_point(&mut parsed_points, line, verb)?;
                    raw_path.line_to(point.0, point.1);
                }
                "quad" => {
                    let control = next_point(&mut parsed_points, line, verb)?;
                    let point = next_point(&mut parsed_points, line, verb)?;
                    raw_path.quad_to(control.0, control.1, point.0, point.1);
                }
                "cubic" => {
                    let outer = next_point(&mut parsed_points, line, verb)?;
                    let inner = next_point(&mut parsed_points, line, verb)?;
                    let point = next_point(&mut parsed_points, line, verb)?;
                    raw_path.cubic_to(outer.0, outer.1, inner.0, inner.1, point.0, point.1);
                }
                "close" => raw_path.close(),
                other => return Err(StreamError::new(line, format!("unknown path verb {other}"))),
            }
        }
    }
    if parsed_points.next().is_some() {
        return Err(StreamError::new(line, "raw path has unused points"));
    }
    Ok(Path {
        fill_rule,
        raw_path,
    })
}

fn next_point(
    points: &mut impl Iterator<Item = (f32, f32)>,
    line: usize,
    verb: &str,
) -> Result<(f32, f32), StreamError> {
    points
        .next()
        .ok_or_else(|| StreamError::new(line, format!("{verb} has too few points")))
}

fn parse_paint(value: &str, line: usize) -> Result<Paint, StreamError> {
    Ok(Paint {
        style: match field(value, "style", line)? {
            "fill" => RenderPaintStyle::Fill,
            "stroke" => RenderPaintStyle::Stroke,
            other => return Err(StreamError::new(line, format!("bad paint style {other}"))),
        },
        color: parse_hex_u32(field(value, "color", line)?, line)?,
        thickness: parse_field(value, "thickness", line)?,
        join: parse_join(parse_field(value, "join", line)?, line)?,
        cap: parse_cap(parse_field(value, "cap", line)?, line)?,
        feather: parse_field(value, "feather", line)?,
        blend_mode: parse_blend(parse_field(value, "blendMode", line)?, line)?,
        shader: parse_field(value, "shader", line)?,
    })
}

fn parse_stops(value: &str, line: usize) -> Result<Vec<GradientStop>, StreamError> {
    let value = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| StreamError::new(line, "bad gradient stops"))?;
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split("},{")
        .map(|stop| {
            let stop = stop.trim_matches(|ch| ch == '{' || ch == '}');
            let (color, offset) = stop
                .split_once(",stop=")
                .ok_or_else(|| StreamError::new(line, "bad gradient stop"))?;
            Ok(GradientStop {
                color: parse_hex_u32(
                    color
                        .strip_prefix("color=")
                        .ok_or_else(|| StreamError::new(line, "gradient stop has no color"))?,
                    line,
                )?,
                offset: parse(offset, line, "gradient stop")?,
            })
        })
        .collect()
}

fn field<'a>(value: &'a str, name: &str, line: usize) -> Result<&'a str, StreamError> {
    let marker = format!("{name}=");
    let start = value
        .find(&marker)
        .ok_or_else(|| StreamError::new(line, format!("missing field {name}")))?
        + marker.len();
    let rest = &value[start..];
    let end = balanced_field_end(rest);
    Ok(rest[..end].trim_end_matches('}'))
}

fn balanced_field_end(value: &str) -> usize {
    let mut round = 0usize;
    let mut square = 0usize;
    let mut curly = 0usize;
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'(' => round += 1,
            b')' => round = round.saturating_sub(1),
            b'[' => square += 1,
            b']' => square = square.saturating_sub(1),
            b'{' => curly += 1,
            b'}' if curly > 0 => curly -= 1,
            b' ' | b',' if round == 0 && square == 0 && curly == 0 => return index,
            _ => {}
        }
    }
    value.len()
}

fn parse_field<T: std::str::FromStr>(
    value: &str,
    name: &str,
    line: usize,
) -> Result<T, StreamError> {
    parse(field(value, name, line)?, line, name)
}

fn parse_prefix_value<T: std::str::FromStr>(
    value: &str,
    line: usize,
    name: &str,
) -> Result<T, StreamError> {
    let token = value.split_whitespace().next().unwrap_or(value);
    parse(token, line, name)
}

fn parse<T: std::str::FromStr>(value: &str, line: usize, name: &str) -> Result<T, StreamError> {
    value
        .parse()
        .map_err(|_| StreamError::new(line, format!("bad {name} value `{value}`")))
}

fn parse_array6(value: &str, line: usize) -> Result<[f32; 6], StreamError> {
    let values = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| StreamError::new(line, "bad matrix"))?
        .split(',')
        .map(|value| parse(value, line, "matrix component"))
        .collect::<Result<Vec<_>, _>>()?;
    values
        .try_into()
        .map_err(|_| StreamError::new(line, "matrix must contain six values"))
}

fn parse_pair(value: &str, line: usize) -> Result<(f32, f32), StreamError> {
    let value = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| StreamError::new(line, "bad point"))?;
    let (x, y) = value
        .split_once(',')
        .ok_or_else(|| StreamError::new(line, "point needs two values"))?;
    Ok((parse(x, line, "x")?, parse(y, line, "y")?))
}

fn parse_pairs(value: &str, line: usize) -> Result<Vec<(f32, f32)>, StreamError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split("),(")
        .map(|pair| {
            parse_pair(
                &format!("({})", pair.trim_matches(|ch| ch == '(' || ch == ')')),
                line,
            )
        })
        .collect()
}

fn parse_sampler(value: &str, line: usize) -> Result<ImageSampler, StreamError> {
    Ok(ImageSampler {
        wrap_x: parse_wrap(parse_field(value, "wrapX", line)?, line)?,
        wrap_y: parse_wrap(parse_field(value, "wrapY", line)?, line)?,
        filter: parse_filter(parse_field(value, "filter", line)?, line)?,
    })
}

fn parse_fill_rule(value: u8, line: usize) -> Result<FillRule, StreamError> {
    match value {
        0 => Ok(FillRule::NonZero),
        1 => Ok(FillRule::EvenOdd),
        2 => Ok(FillRule::Clockwise),
        _ => Err(StreamError::new(line, format!("bad fill rule {value}"))),
    }
}

fn parse_blend(value: u8, line: usize) -> Result<BlendMode, StreamError> {
    match value {
        3 => Ok(BlendMode::SrcOver),
        14 => Ok(BlendMode::Screen),
        15 => Ok(BlendMode::Overlay),
        16 => Ok(BlendMode::Darken),
        17 => Ok(BlendMode::Lighten),
        18 => Ok(BlendMode::ColorDodge),
        19 => Ok(BlendMode::ColorBurn),
        20 => Ok(BlendMode::HardLight),
        21 => Ok(BlendMode::SoftLight),
        22 => Ok(BlendMode::Difference),
        23 => Ok(BlendMode::Exclusion),
        24 => Ok(BlendMode::Multiply),
        25 => Ok(BlendMode::Hue),
        26 => Ok(BlendMode::Saturation),
        27 => Ok(BlendMode::Color),
        28 => Ok(BlendMode::Luminosity),
        _ => Err(StreamError::new(line, format!("bad blend mode {value}"))),
    }
}

fn parse_join(value: u32, line: usize) -> Result<StrokeJoin, StreamError> {
    match value {
        0 => Ok(StrokeJoin::Miter),
        1 => Ok(StrokeJoin::Round),
        2 => Ok(StrokeJoin::Bevel),
        _ => Err(StreamError::new(line, format!("bad stroke join {value}"))),
    }
}

fn parse_cap(value: u32, line: usize) -> Result<StrokeCap, StreamError> {
    match value {
        0 => Ok(StrokeCap::Butt),
        1 => Ok(StrokeCap::Round),
        2 => Ok(StrokeCap::Square),
        _ => Err(StreamError::new(line, format!("bad stroke cap {value}"))),
    }
}

fn parse_wrap(value: u8, line: usize) -> Result<ImageWrap, StreamError> {
    match value {
        0 => Ok(ImageWrap::Clamp),
        1 => Ok(ImageWrap::Repeat),
        2 => Ok(ImageWrap::Mirror),
        _ => Err(StreamError::new(line, format!("bad image wrap {value}"))),
    }
}

fn parse_filter(value: u8, line: usize) -> Result<ImageFilter, StreamError> {
    match value {
        0 => Ok(ImageFilter::Bilinear),
        1 => Ok(ImageFilter::Nearest),
        _ => Err(StreamError::new(line, format!("bad image filter {value}"))),
    }
}

fn parse_buffer_type(value: u8, line: usize) -> Result<RenderBufferType, StreamError> {
    match value {
        0 => Ok(RenderBufferType::Index),
        1 => Ok(RenderBufferType::Vertex),
        _ => Err(StreamError::new(line, format!("bad buffer type {value}"))),
    }
}

fn parse_buffer_flags(value: u8, line: usize) -> Result<RenderBufferFlags, StreamError> {
    match value {
        0 => Ok(RenderBufferFlags::None),
        1 => Ok(RenderBufferFlags::MappedOnceAtInitialization),
        _ => Err(StreamError::new(line, format!("bad buffer flags {value}"))),
    }
}

fn parse_hex_u32(value: &str, line: usize) -> Result<u32, StreamError> {
    u32::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|_| StreamError::new(line, format!("bad color `{value}`")))
}

fn parse_hex(value: &str, line: usize) -> Result<Vec<u8>, StreamError> {
    if value.len() % 2 != 0 {
        return Err(StreamError::new(line, "hex data has odd length"));
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| StreamError::new(line, "invalid hex data"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{RecordingFactory, Vec2D};

    const STREAM: &str = "rive-golden-stream-v1\n\
frameSize width=64 height=32\n\
clearColor value=0x11223344\n\
makeLinearGradient id=7 start=(0,1) end=(2,3) stops=[{color=0xff000000,stop=0},{color=0xffffffff,stop=1}]\n\
decodeImage id=2 width=1 height=1 data=89504e47\n\
makeRenderBuffer id=3 type=1 flags=0 size=4\n\
bufferData id=3 type=1 size=4 data=01020304\n\
save\n\
transform matrix=[1,0,0,1,4,5]\n\
drawPath path={id=1,fillRule=0,path={verbs=[move,line,cubic,close],points=[(0,0),(1,1),(2,2),(3,3),(4,4)]}} paint={id=1,style=fill,color=0xff00ff00,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=7}\n\
restore\n\
frame\n";

    #[test]
    fn parses_complete_replay_facts() {
        let stream = RenderStream::parse(STREAM).unwrap();
        assert_eq!(stream.frame_size, Some((64, 32)));
        assert_eq!(stream.clear_color, Some(0x11223344));
        assert_eq!(stream.resources.len(), 3);
        assert_eq!(stream.frames.len(), 1);
        assert_eq!(stream.frames[0].commands.len(), 4);
        let Command::DrawPath { path, paint } = &stream.frames[0].commands[2] else {
            panic!("expected draw path");
        };
        assert_eq!(paint.shader, 7);
        assert_eq!(path.raw_path.points().last(), Some(&Vec2D::new(4.0, 4.0)));
    }

    #[test]
    fn replays_through_render_api() {
        let stream = RenderStream::parse(STREAM).unwrap();
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        stream.replay_frame(0, &mut factory, &mut renderer).unwrap();
        let replayed = factory.stream();
        assert!(replayed.contains("makeLinearGradient"));
        assert!(replayed.contains("drawPath"));
        assert!(replayed.contains("transform matrix=[1,0,0,1,4,5]"));
    }

    #[test]
    fn hard_difference_clip_round_trips_without_losing_half_pixel_edges() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let rect = Aabb::new(f32::from_bits(0x3effffff), -1.5, 12.125, 7.75);
        renderer.save();
        renderer.transform(Mat2D([1.25, 0.0, 0.0, 0.8, 0.25, 0.5]));
        assert!(renderer.clip_out_rect(rect));
        renderer.restore();
        let recorded = factory.stream();
        let stream = RenderStream::parse(&recorded).unwrap();
        assert_eq!(
            stream.frames[0].commands,
            vec![
                Command::Save,
                Command::Transform(Mat2D([1.25, 0.0, 0.0, 0.8, 0.25, 0.5])),
                Command::ClipOutRect(rect),
                Command::Restore,
            ]
        );
        let mut replay_factory = RecordingFactory::new();
        let mut replay_renderer = replay_factory.make_renderer();
        stream
            .replay_frame(0, &mut replay_factory, &mut replay_renderer)
            .unwrap();
        assert_eq!(recorded, replay_factory.stream());
        let before = factory.stream();
        assert!(!renderer.clip_out_rect(Aabb::new(f32::NAN, 0.0, 1.0, 1.0)));
        assert_eq!(before, factory.stream());
        // An unsupported backend must report the missing operation, not draw
        // subsequent paths as though no exclusion were present.
        let mut null = nuxie_render_api::NullFactory::new().make_renderer();
        assert_eq!(
            stream.replay_frame(0, &mut factory, &mut null),
            Err(ReplayError::UnsupportedOperation("clipOutRect"))
        );
    }

    #[test]
    fn rejects_malformed_or_nonfinite_difference_clip_rectangles() {
        for value in [
            "[0,1,2]",
            "[0,1,2,3,4]",
            "[NaN,0,1,2]",
            "[0,inf,1,2]",
            "0,1,2,3",
            "[0,1,no,3]",
        ] {
            let input = format!("rive-golden-stream-v1\nclipOutRect rect={value}\n");
            let error = RenderStream::parse(&input).unwrap_err();
            assert!(error.to_string().starts_with("render stream line 2:"));
        }
    }

    #[test]
    fn rejects_unknown_commands_with_line_number() {
        let error = RenderStream::parse("rive-golden-stream-v1\nexplode\n").unwrap_err();
        assert_eq!(
            error.to_string(),
            "render stream line 2: unsupported command `explode`"
        );
    }

    #[test]
    fn axis_clips_round_trip_and_require_backend_support() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        renderer.save();
        renderer.transform(Mat2D([1.,0.5,0.,1.,3.,4.]));
        assert!(renderer.clip_axis(true, f32::from_bits(0x3effffff), 7.25));
        assert!(renderer.clip_axis(false, -3.5, 100.));
        let local = Mat2D([0.8660254, 0.5, -0.5, 0.8660254, 100.25, 60.75]);
        assert!(renderer.clip_axis_transformed(true, 0.25, 100.75, local));
        renderer.restore();
        let recorded = factory.stream();
        let stream = RenderStream::parse(&recorded).unwrap();
        assert_eq!(stream.frames[0].commands[2], Command::ClipAxis {
            horizontal:true,min:f32::from_bits(0x3effffff),max:7.25,local:None
        });
        assert_eq!(stream.frames[0].commands[4], Command::ClipAxis {
            horizontal: true, min: 0.25, max: 100.75, local: Some(local)
        });
        let mut replay_factory = RecordingFactory::new();
        let mut replay_renderer = replay_factory.make_renderer();
        stream.replay_frame(0,&mut replay_factory,&mut replay_renderer).unwrap();
        assert_eq!(recorded,replay_factory.stream());
        let before = factory.stream();
        assert!(!renderer.clip_axis(true,f32::NAN,1.));
        assert!(!renderer.clip_axis_transformed(true, 0., 1., Mat2D([f32::NAN; 6])));
        assert_eq!(before,factory.stream());
        let mut null = nuxie_render_api::NullFactory::new().make_renderer();
        assert_eq!(stream.replay_frame(0,&mut factory,&mut null),Err(ReplayError::UnsupportedOperation("clipAxis")));
    }

    #[test]
    fn axis_clip_parser_rejects_invalid_axes_and_ranges() {
        for value in ["z range=[0,1]","x range=[0]","y range=[0,1,2]","x range=[NaN,1]","y range=[0,inf]","x range=0,1"] {
            let error=RenderStream::parse(&format!("rive-golden-stream-v1\nclipAxis axis={value}\n")).unwrap_err();
            assert!(error.to_string().starts_with("render stream line 2:"));
        }
    }
}

#[cfg(test)]
mod premultiplied_gradient_tests {
    use super::*;
    use nuxie_render_api::{Factory, NullFactory, PersistentFactory, RecordingFactory};

    #[test]
    fn explicit_interpolation_round_trips_and_rejects_unsupported_factory() {
        let mut source = PersistentFactory::new(RecordingFactory::new());
        assert!(source.make_premultiplied_linear_gradient(0.,0.,100.,0., &[0xffff0000,0], &[0.,1.]).is_some());
        let text = source.borrow().stream();
        assert!(text.contains("makePremultipliedLinearGradient"));
        let stream = RenderStream::parse(&format!("{text}frame\n")).unwrap();
        assert!(matches!(stream.resources[0], Resource::PremultipliedLinearGradient { .. }));
        let mut target = RecordingFactory::new();
        let mut renderer = target.make_renderer();
        stream.replay_frame(0,&mut target,&mut renderer).unwrap();
        assert_eq!(target.stream(),text);
        let mut unsupported = NullFactory::new();
        let mut renderer = unsupported.make_renderer();
        assert_eq!(stream.replay_frame(0,&mut unsupported,&mut renderer),
            Err(ReplayError::UnsupportedOperation("premultiplied linear gradients")));
    }

    #[test]
    fn invalid_recording_has_no_output_and_ordinary_mode_is_unchanged() {
        let mut factory = RecordingFactory::new();
        for (colors, stops) in [(vec![0],vec![0.]),(vec![0,1],vec![1.,0.]),(vec![0,1],vec![0.,f32::NAN])] {
            let before=factory.stream();
            assert!(factory.make_premultiplied_linear_gradient(0.,0.,100.,0.,&colors,&stops).is_none());
            assert_eq!(factory.stream(),before);
        }
        let _ = factory.make_linear_gradient(0.,0.,100.,0., &[0xffff0000,0], &[0.,1.]);
        let stream = RenderStream::parse(&format!("{}frame\n",factory.stream())).unwrap();
        assert!(matches!(stream.resources[0], Resource::LinearGradient { .. }));
        let mut null=NullFactory::new();let mut renderer=null.make_renderer();
        stream.replay_frame(0,&mut null,&mut renderer).unwrap();
    }
}

#[cfg(test)]
mod tiled_premultiplied_gradient_tests {
    use super::*;
    use nuxie_render_api::{NullFactory, PersistentFactory, RecordingFactory};

    #[test]
    fn tiled_gradient_round_trips_without_untiled_fallback() {
        let mut source = PersistentFactory::new(RecordingFactory::new());
        assert!(source.make_tiled_premultiplied_linear_gradient(
            -5., 4., 90., 40., [-12.5, 3., 120., 80.],
            &[0x00ff0000, 0xff0000ff, 0xffffffff], &[0., 0.5, 1.]).is_some());
        let text = source.borrow().stream();
        assert!(text.contains("makeTiledPremultipliedLinearGradient id=1 start=(-5,4) end=(90,40) tile=(-12.5,3,120,80)"));
        let stream = RenderStream::parse(&format!("{text}frame\n")).unwrap();
        assert!(matches!(stream.resources[0], Resource::TiledPremultipliedLinearGradient { tile: [-12.5, 3., 120., 80.], .. }));
        let mut target = RecordingFactory::new();
        let mut renderer = target.make_renderer();
        stream.replay_frame(0, &mut target, &mut renderer).unwrap();
        assert_eq!(target.stream(), text);
        let mut unsupported = NullFactory::new();
        let mut renderer = unsupported.make_renderer();
        assert_eq!(stream.replay_frame(0, &mut unsupported, &mut renderer),
            Err(ReplayError::UnsupportedOperation("tiled premultiplied linear gradients")));
    }

    #[test]
    fn invalid_tile_or_gradient_is_rejected_without_recording_or_consuming_ids() {
        let mut factory = RecordingFactory::new();
        let before = factory.stream();
        for tile in [[0.,0.,0.,20.], [0.,0.,20.,-1.], [f32::NAN,0.,20.,20.],
            [0.,f32::INFINITY,20.,20.], [0.,0.,f32::INFINITY,20.], [0.,0.,1e-40,20.],
            [0.,0.,20.,1e-40], [f32::MAX,0.,0.5,20.]] {
            assert!(factory.make_tiled_premultiplied_linear_gradient(
                0.,0.,20.,20.,tile,&[0,1],&[0.,1.]).is_none());
        }
        for (colors, stops) in [(vec![0],vec![0.]), (vec![0,1],vec![1.,0.]),
            (vec![0,1],vec![0.,f32::NAN]), (vec![0,1],vec![0.]),
            (vec![0,1],vec![-0.1,1.])] {
            assert!(factory.make_tiled_premultiplied_linear_gradient(
                0.,0.,20.,20.,[0.,0.,20.,20.],&colors,&stops).is_none());
        }
        assert!(factory.make_tiled_premultiplied_linear_gradient(
            0.,0.,f32::NAN,20.,[0.,0.,20.,20.],&[0,1],&[0.,1.]).is_none());
        assert_eq!(factory.stream(), before);
        assert!(factory.make_tiled_premultiplied_linear_gradient(
            0.,0.,20.,20.,[-1.,-1.,20.,20.],&[0,1],&[0.5,0.5]).is_some());
        assert!(factory.stream().contains("makeTiledPremultipliedLinearGradient id=1 "));
    }

    #[test]
    fn malformed_tiled_stream_is_rejected_at_parse_boundary() {
        let valid = "rive-golden-stream-v1\nmakeTiledPremultipliedLinearGradient id=1 start=(0,0) end=(20,20) tile=(0,0,20,20) stops=[{color=0xff000000,stop=0},{color=0xffffffff,stop=1}]\nframe\n";
        assert!(RenderStream::parse(valid).is_ok());
        for tile in ["(0,0,20)", "(0,0,20,20,1)", "[0,0,20,20]", "(0,0,0,20)",
            "(0,0,20,-1)", "(NaN,0,20,20)", "(0,0,inf,20)", "(0,0,1e-40,20)",
            "(0,0,20,1e-40)", "(3e38,0,0.5,20)"] {
            assert!(RenderStream::parse(&valid.replace("(0,0,20,20)", tile)).is_err(), "{tile}");
        }
        assert!(RenderStream::parse(&valid.replace("tile=(0,0,20,20) ", "")).is_err());
        assert!(RenderStream::parse(&valid.replace("end=(20,20)", "end=(NaN,20)")).is_err());
        assert!(RenderStream::parse(&valid.replace("stop=1", "stop=-1")).is_err());
    }
}
