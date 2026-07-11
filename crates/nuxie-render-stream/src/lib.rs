//! Typed parsing and renderer-neutral replay for `rive-golden-stream-v1`.

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageFilter, ImageSampler, ImageWrap, Mat2D, RawPath,
    RenderBufferFlags, RenderBufferType, RenderPaintStyle, Renderer, StrokeCap, StrokeJoin,
};
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
    Save,
    Restore,
    Transform(Mat2D),
    DrawPath {
        path: Path,
        paint: Paint,
    },
    ClipPath(Path),
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
        let mut shaders = HashMap::new();
        let mut images = HashMap::new();
        let mut buffers = HashMap::new();
        for resource in &self.resources {
            match resource {
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
                    images.insert(*id, factory.decode_image(data));
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

        let frame = self
            .frames
            .get(frame_index)
            .ok_or(ReplayError::MissingFrame(frame_index))?;
        for command in &frame.commands {
            match command {
                Command::Save => renderer.save(),
                Command::Restore => renderer.restore(),
                Command::Transform(matrix) => renderer.transform(*matrix),
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
                        let shader = shaders
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
                    resource_ref(&images, "image", *image)?,
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
                    resource_ref(&images, "image", *image)?,
                    *sampler,
                    buffer_ref(&buffers, *vertices)?,
                    buffer_ref(&buffers, *uvs)?,
                    buffer_ref(&buffers, *indices)?,
                    *vertex_count,
                    *index_count,
                    *blend_mode,
                    *opacity,
                ),
                Command::ModulateOpacity(opacity) => renderer.modulate_opacity(*opacity),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    MissingFrame(usize),
    MissingResource(&'static str, u64),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrame(index) => write!(f, "render stream has no frame {index}"),
            Self::MissingResource(kind, id) => write!(f, "missing {kind} resource {id}"),
        }
    }
}

impl Error for ReplayError {}

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
    fn rejects_unknown_commands_with_line_number() {
        let error = RenderStream::parse("rive-golden-stream-v1\nexplode\n").unwrap_err();
        assert_eq!(
            error.to_string(),
            "render stream line 2: unsupported command `explode`"
        );
    }
}
