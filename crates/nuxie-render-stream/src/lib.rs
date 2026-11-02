//! Typed parsing and renderer-neutral replay for `rive-golden-stream-v1`.

use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageFilter, ImageSampler, ImageWrap,
    Mat2D, PathVerb, RawPath, RenderBufferFlags, RenderBufferType, RenderPaintStyle, Renderer,
    StrokeCap, StrokeJoin, Vec2D,
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
    ModulateColor {
        color: ColorInt,
        replace: bool,
    },
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
            if let Some(value) = line.strip_prefix("modulateColor ") {
                commands.push(Command::ModulateColor {
                    color: parse_hex_u32(field(value, "color", line_number)?, line_number)?,
                    replace: parse_field::<u32>(value, "replace", line_number)? != 0,
                });
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
                Command::ModulateColor { color, replace } => {
                    renderer.modulate_color(*color, *replace)
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    MissingFrame(usize),
    MissingResource(&'static str, u64),
    ImageDecode { id: u64, source: ImageDecodeError },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Self::MissingFrame(_) | Self::MissingResource(_, _) => None,
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

/// One command as [`frame_draws`] replays it. The recorded text stream gives
/// these through [`stream_ops`]; a harness whose commands aren't text (the
/// renderer's deferred one) fills the same type.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamOp {
    Save,
    Restore,
    Transform(Mat2D),
    ModulateOpacity(f32),
    ModulateColor {
        color: ColorInt,
        replace: bool,
    },
    ClipPath(RawPath),
    DrawPath {
        path: RawPath,
        color: ColorInt,
        shader: u64,
        /// The strongest alpha among the shader's stops, when the stream holds
        /// them. A stream cleared after the gradient was made doesn't.
        stops_alpha: Option<f32>,
    },
    DrawImage {
        opacity: f32,
    },
    /// A canvas drawn as one picture, under the matrix and opacity in scope,
    /// at its own matrix and opacity.
    Composite {
        canvas: u64,
        matrix: Mat2D,
        opacity: f32,
    },
}

/// What a canvas holds: the ops recorded while it was the target.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub id: u64,
    pub ops: Vec<StreamOp>,
}

/// A clip in scope at a draw: its path's verbs, and its points mapped by the
/// matrix at the clip.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnClip {
    pub verbs: Vec<PathVerb>,
    pub points: Vec<Vec2D>,
}

/// One draw as the renderer got it.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDraw {
    /// The whole matrix at the draw, through any canvas's composite.
    pub matrix: Mat2D,
    /// What the renderer's alpha comes to, through every render target. On a
    /// target, the draw's own alpha times that target's scope, held from 0 to
    /// 1. A canvas has its own target, whose scope starts at 1, and its picture
    /// is one more draw on the target under it, so each level is held by
    /// itself and passes on what it comes to.
    ///
    /// A draw's own alpha is the paint's for a plain color, the stops' for a
    /// gradient, whose paint alpha the renderer doesn't use, and the image's
    /// own opacity. Stops that differ read the strongest, as drawn, and stops
    /// the stream doesn't hold count as opaque. A scope is its `modulateOpacity`
    /// values, never below 0, times its `modulateColor` alpha.
    pub opacity: f32,
    /// The `modulateOpacity` values in scope, as recorded, on this draw's
    /// target and each target under it.
    pub modulations: Vec<f32>,
    /// The opacities of the composites this draw is inside, outermost first.
    pub composites: Vec<f32>,
    /// The paint's whole ARGB color, as recorded. None for an image.
    pub color: Option<ColorInt>,
    pub shader: u64,
    /// Every clip in scope, outermost first.
    pub clips: Vec<DrawnClip>,
    /// The drawn path's points, mapped by `matrix`. Empty for an image.
    pub points: Vec<Vec2D>,
    /// The canvas this draw is inside, innermost.
    pub canvas: Option<u64>,
}

impl FrameDraw {
    /// The paint's color without its alpha.
    pub fn rgb(&self) -> Option<u32> {
        self.color.map(|color| color & 0x00ff_ffff)
    }
    /// The lengths of the matrix's x and y columns.
    pub fn axis_scales(&self) -> (f32, f32) {
        let [xx, xy, yx, yy, _, _] = self.matrix.0;
        (xx.hypot(xy), yx.hypot(yy))
    }
}

/// The drawing commands of a recorded stream, each gradient paint with the
/// alpha of its stops. Everything else in it (other resources, frame marks) says
/// nothing about what was drawn.
pub fn stream_ops(recorded: &str) -> Result<Vec<StreamOp>, StreamError> {
    let mut ops = Vec::new();
    let mut stops_alpha = HashMap::new();
    for (zero_line, line) in recorded.lines().enumerate() {
        let number = zero_line + 1;
        if let Some(value) = line.strip_prefix("makeLinearGradient ") {
            if let Resource::LinearGradient { id, stops, .. } =
                parse_linear_gradient(value, number)?
            {
                stops_alpha.insert(id, strongest_alpha(&stops));
            }
        } else if let Some(value) = line.strip_prefix("makeRadialGradient ") {
            if let Resource::RadialGradient { id, stops, .. } =
                parse_radial_gradient(value, number)?
            {
                stops_alpha.insert(id, strongest_alpha(&stops));
            }
        } else if line == "save" {
            ops.push(StreamOp::Save);
        } else if line == "restore" {
            ops.push(StreamOp::Restore);
        } else if let Some(value) = line.strip_prefix("transform matrix=") {
            ops.push(StreamOp::Transform(Mat2D(parse_array6(value, number)?)));
        } else if let Some(value) = line.strip_prefix("modulateOpacity opacity=") {
            ops.push(StreamOp::ModulateOpacity(parse(value, number, "opacity")?));
        } else if let Some(value) = line.strip_prefix("modulateColor ") {
            ops.push(StreamOp::ModulateColor {
                color: parse_hex_u32(field(value, "color", number)?, number)?,
                replace: parse_field::<u32>(value, "replace", number)? != 0,
            });
        } else if let Some(value) = line.strip_prefix("clipPath path=") {
            ops.push(StreamOp::ClipPath(parse_path(value, number)?.raw_path));
        } else if let Some(value) = line.strip_prefix("drawPath path=") {
            let split = value
                .find(" paint=")
                .ok_or_else(|| StreamError::new(number, "drawPath has no paint"))?;
            let paint = parse_paint(&value[split + 7..], number)?;
            ops.push(StreamOp::DrawPath {
                path: parse_path(&value[..split], number)?.raw_path,
                color: paint.color,
                shader: paint.shader,
                stops_alpha: stops_alpha.get(&paint.shader).copied(),
            });
        } else if let Some(value) = line.strip_prefix("drawImage image=") {
            let Command::DrawImage { opacity, .. } = parse_draw_image(value, number)? else {
                unreachable!("parse_draw_image makes a DrawImage");
            };
            ops.push(StreamOp::DrawImage { opacity });
        } else if let Some(value) = line.strip_prefix("drawImageMesh image=") {
            let Command::DrawImageMesh { opacity, .. } = parse_draw_image_mesh(value, number)?
            else {
                unreachable!("parse_draw_image_mesh makes a DrawImageMesh");
            };
            ops.push(StreamOp::DrawImage { opacity });
        }
    }
    Ok(ops)
}

/// Every draw of a frame, in draw order, as the renderer got it: its whole
/// matrix, its opacity and every clip in scope. A canvas's draws are mapped by
/// its composite's matrix and what its composite comes to, so a draw inside a
/// picture has its final matrix and opacity.
pub fn frame_draws(ops: &[StreamOp], canvases: &[Canvas]) -> Vec<FrameDraw> {
    let mut draws = Vec::new();
    replay(
        ops,
        canvases,
        DrawScope {
            matrix: Mat2D::IDENTITY,
            opacity: 1.0,
            color_alpha: 1.0,
            pictured: 1.0,
            modulations: Vec::new(),
            composites: Vec::new(),
            clips: Vec::new(),
            canvas: None,
        },
        &mut draws,
    );
    draws
}

#[derive(Clone)]
struct DrawScope {
    matrix: Mat2D,
    /// The renderer's `modulatedOpacity` on this render target. A canvas's
    /// renderer starts at 1.
    opacity: f32,
    /// The alpha of the renderer's `modulatedColor` on this render target. A
    /// canvas's renderer starts at white.
    color_alpha: f32,
    /// What the pictures this target is inside come to: each composite as drawn
    /// on the target under it, all multiplied.
    pictured: f32,
    modulations: Vec<f32>,
    composites: Vec<f32>,
    clips: Vec<DrawnClip>,
    canvas: Option<u64>,
}

impl DrawScope {
    /// A draw of this alpha on this render target, as `RiveRenderer` takes it:
    /// its opacity times the target's, never below 0, then `color_modulate`
    /// with the target's color, held from 0 to 1. `min` keeps `color_modulate`'s
    /// NaN at 1. Only the alpha is taken, so its rounding to 8 bits is not.
    ///
    /// `drawImageMesh` and a frame without image paints take a `drawImage`
    /// this way. A frame with them holds a `drawImage`'s own opacity at 1 first
    /// (`RiveRenderPaint::image`), so an image above 1 under a scope below 1
    /// draws less there. The stream doesn't name the frame's mode, so that
    /// difference isn't read here.
    fn on_target(&self, alpha: f32) -> f32 {
        ((alpha * self.opacity).max(0.0) * self.color_alpha).min(1.0)
    }

    /// A draw of this alpha, through every picture it's inside.
    fn drawn(&self, alpha: f32) -> f32 {
        self.on_target(alpha) * self.pictured
    }
}

/// The alpha a stop's color carries. Stops that differ have no one alpha, so the
/// strongest stands for them: `color_modulate` holds each by itself, and the
/// strongest held is the strongest drawn.
fn strongest_alpha(stops: &[GradientStop]) -> f32 {
    stops
        .iter()
        .map(|stop| color_alpha(stop.color))
        .fold(0.0, f32::max)
}

fn color_alpha(color: ColorInt) -> f32 {
    ((color >> 24) & 0xff) as f32 / 255.0
}

fn replay(ops: &[StreamOp], canvases: &[Canvas], mut scope: DrawScope, draws: &mut Vec<FrameDraw>) {
    let mut saved = Vec::new();
    for op in ops {
        match op {
            StreamOp::Save => saved.push(scope.clone()),
            StreamOp::Restore => scope = saved.pop().expect("a save for each restore"),
            StreamOp::Transform(matrix) => scope.matrix = multiply(scope.matrix, *matrix),
            // As the renderer's `modulateOpacity`: never below 0.
            StreamOp::ModulateOpacity(opacity) => {
                scope.opacity = (scope.opacity * opacity).max(0.0);
                scope.modulations.push(*opacity);
            }
            // As the renderer's `modulateColor`.
            StreamOp::ModulateColor { color, replace } => {
                let alpha = color_alpha(*color);
                scope.color_alpha = if *replace {
                    alpha
                } else {
                    scope.color_alpha * alpha
                };
            }
            StreamOp::ClipPath(path) => scope.clips.push(DrawnClip {
                verbs: path.verbs().to_vec(),
                points: mapped(scope.matrix, path),
            }),
            StreamOp::DrawPath {
                path,
                color,
                shader,
                stops_alpha,
            } => draws.push(FrameDraw {
                matrix: scope.matrix,
                // Setting a shader drops the renderer paint's color, and its
                // alpha is its stops', each by `color_modulate`. An image rect
                // over a gradient takes white in place of the color.
                opacity: scope.drawn(if *shader == 0 {
                    color_alpha(*color)
                } else {
                    stops_alpha.unwrap_or(1.0)
                }),
                modulations: scope.modulations.clone(),
                composites: scope.composites.clone(),
                color: Some(*color),
                shader: *shader,
                clips: scope.clips.clone(),
                points: mapped(scope.matrix, path),
                canvas: scope.canvas,
            }),
            StreamOp::DrawImage { opacity } => draws.push(FrameDraw {
                matrix: scope.matrix,
                opacity: scope.drawn(*opacity),
                modulations: scope.modulations.clone(),
                composites: scope.composites.clone(),
                color: None,
                shader: 0,
                clips: scope.clips.clone(),
                points: Vec::new(),
                canvas: scope.canvas,
            }),
            StreamOp::Composite {
                canvas,
                matrix,
                opacity,
            } => {
                let held = canvases
                    .iter()
                    .find(|held| held.id == *canvas)
                    .unwrap_or_else(|| panic!("a composite of canvas {canvas}, which has no ops"));
                let mut inside = scope.clone();
                inside.matrix = multiply(scope.matrix, *matrix);
                inside.pictured = scope.drawn(*opacity);
                inside.opacity = 1.0;
                inside.color_alpha = 1.0;
                inside.composites.push(*opacity);
                inside.canvas = Some(*canvas);
                replay(&held.ops, canvases, inside, draws);
            }
        }
    }
}

fn multiply(a: Mat2D, b: Mat2D) -> Mat2D {
    let [a0, a1, a2, a3, a4, a5] = a.0;
    let [b0, b1, b2, b3, b4, b5] = b.0;
    Mat2D([
        a0 * b0 + a2 * b1,
        a1 * b0 + a3 * b1,
        a0 * b2 + a2 * b3,
        a1 * b2 + a3 * b3,
        a0 * b4 + a2 * b5 + a4,
        a1 * b4 + a3 * b5 + a5,
    ])
}

fn mapped(matrix: Mat2D, path: &RawPath) -> Vec<Vec2D> {
    path.points()
        .iter()
        .map(|point| matrix.transform_point(*point))
        .collect()
}

/// A rounded rectangle read from a clip's mapped points: the top edge's
/// direction is its turn, the four edge lines meet at its corners, and each
/// corner's two radii are the distances from that corner to the tangent points
/// beside it. It reads the path a CSS box's rounded rectangle makes: clockwise
/// from the top left corner's end, with a cubic at each rounded corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundedRect {
    pub center: Vec2D,
    pub width: f32,
    pub height: f32,
    pub turn: f32,
    /// Top left, top right, bottom right, bottom left.
    pub corners: [Vec2D; 4],
    /// Each corner's radius along the top edge's direction, then along the
    /// left edge's, in `corners`' order.
    pub radii: [(f32, f32); 4],
}

/// One straight side of a clip's path, and the tangents of the cubics beside
/// it, which give its direction when it has no length.
struct ClipEdge {
    from: Vec2D,
    to: Vec2D,
    leading: Option<Vec2D>,
    trailing: Option<Vec2D>,
}

impl ClipEdge {
    fn direction(&self) -> Option<Vec2D> {
        [Some(minus(self.to, self.from)), self.leading, self.trailing]
            .into_iter()
            .flatten()
            .find(|direction| direction.x.hypot(direction.y) > 1e-5)
    }
}

fn minus(a: Vec2D, b: Vec2D) -> Vec2D {
    Vec2D::new(a.x - b.x, a.y - b.y)
}

fn distance(a: Vec2D, b: Vec2D) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// Where the lines through `p` along `d` and through `q` along `e` meet.
fn meet(p: Vec2D, d: Vec2D, q: Vec2D, e: Vec2D) -> Option<Vec2D> {
    let cross = d.x * e.y - d.y * e.x;
    if cross.abs() < 1e-9 {
        return None;
    }
    let between = minus(q, p);
    let t = (between.x * e.y - between.y * e.x) / cross;
    Some(Vec2D::new(p.x + d.x * t, p.y + d.y * t))
}

impl RoundedRect {
    /// None when the clip isn't one contour with four straight sides.
    pub fn read(clip: &DrawnClip) -> Option<Self> {
        let mut points = clip.points.iter().copied();
        let mut edges: Vec<ClipEdge> = Vec::new();
        let (mut start, mut at, mut leading) = (None, None, None);
        for verb in &clip.verbs {
            match verb {
                PathVerb::Move => {
                    if start.is_some() {
                        return None;
                    }
                    start = points.next();
                    at = start;
                }
                PathVerb::Line => {
                    let to = points.next()?;
                    edges.push(ClipEdge {
                        from: at?,
                        to,
                        leading: leading.take(),
                        trailing: None,
                    });
                    at = Some(to);
                }
                PathVerb::Cubic => {
                    let (outer, inner, to) = (points.next()?, points.next()?, points.next()?);
                    if let Some(edge) = edges.last_mut() {
                        edge.trailing.get_or_insert(minus(outer, edge.to));
                    }
                    leading = Some(minus(to, inner));
                    at = Some(to);
                }
                PathVerb::Close => {
                    if distance(at?, start?) > 1e-5 {
                        edges.push(ClipEdge {
                            from: at?,
                            to: start?,
                            leading: leading.take(),
                            trailing: None,
                        });
                    }
                }
                PathVerb::Quad => return None,
            }
        }
        let [top, right, bottom, left] = edges.as_slice() else {
            return None;
        };
        let along = [
            top.direction()?,
            right.direction()?,
            bottom.direction()?,
            left.direction()?,
        ];
        let top_left = meet(left.to, along[3], top.from, along[0])?;
        let top_right = meet(top.to, along[0], right.from, along[1])?;
        let bottom_right = meet(right.to, along[1], bottom.from, along[2])?;
        let bottom_left = meet(bottom.to, along[2], left.from, along[3])?;
        let corners = [top_left, top_right, bottom_right, bottom_left];
        Some(Self {
            center: Vec2D::new(
                corners.iter().map(|corner| corner.x).sum::<f32>() / 4.0,
                corners.iter().map(|corner| corner.y).sum::<f32>() / 4.0,
            ),
            width: distance(top_left, top_right),
            height: distance(top_left, bottom_left),
            turn: along[0].y.atan2(along[0].x),
            corners,
            radii: [
                (distance(top_left, top.from), distance(top_left, left.to)),
                (distance(top_right, top.to), distance(top_right, right.from)),
                (
                    distance(bottom_right, bottom.from),
                    distance(bottom_right, right.to),
                ),
                (
                    distance(bottom_left, bottom.to),
                    distance(bottom_left, left.from),
                ),
            ],
        })
    }

    /// What differs from `other`, comparing the center, width, height, corners
    /// and every corner's two radii to 1e-3 of this box's width, and the turn
    /// to 1e-3 rad. None when nothing does.
    pub fn difference(&self, other: &Self) -> Option<String> {
        let tolerance = self.width.abs() * 1e-3;
        let near = |a: f32, b: f32| (a - b).abs() <= tolerance;
        let turn = (self.turn - other.turn + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let same = turn.abs() <= 1e-3
            && near(self.center.x, other.center.x)
            && near(self.center.y, other.center.y)
            && near(self.width, other.width)
            && near(self.height, other.height)
            && self
                .corners
                .iter()
                .zip(&other.corners)
                .all(|(a, b)| near(a.x, b.x) && near(a.y, b.y))
            && self
                .radii
                .iter()
                .zip(&other.radii)
                .all(|(a, b)| near(a.0, b.0) && near(a.1, b.1));
        (!same).then(|| format!("{self:?} differs from {other:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::RecordingFactory;

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
    fn a_canvas_is_read_through_its_composite() {
        let square = RawPath::from_verbs_and_points(
            vec![PathVerb::Move, PathVerb::Line, PathVerb::Close],
            vec![Vec2D::new(1.0, 1.0), Vec2D::new(2.0, 1.0)],
        );
        let canvas = Canvas {
            id: 4,
            ops: vec![
                StreamOp::Transform(Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 0.0])),
                StreamOp::DrawPath {
                    path: square.clone(),
                    color: 0x80ff_0000,
                    shader: 0,
                    stops_alpha: None,
                },
            ],
        };
        let draws = frame_draws(
            &[
                StreamOp::Save,
                StreamOp::Transform(Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 0.0])),
                StreamOp::Composite {
                    canvas: 4,
                    matrix: Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 5.0]),
                    opacity: 0.5,
                },
                StreamOp::Restore,
            ],
            &[canvas],
        );
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].matrix, Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 5.0]));
        assert_eq!(draws[0].points[0], Vec2D::new(12.0, 7.0));
        assert_eq!(draws[0].canvas, Some(4));
        assert_eq!(draws[0].composites, vec![0.5]);
        assert!((draws[0].opacity - 0.5 * 128.0 / 255.0).abs() < 1e-6);
    }

    /// An overshooting animation takes an opacity past either end. Each kind of
    /// draw reads what the renderer's alpha comes to, and the scope is only
    /// held at 0, so a scope above 1 still counts before the draw is clamped.
    #[test]
    fn a_draws_opacity_is_clamped_as_the_renderer_clamps_it() {
        const SAMPLER: &str = "{wrapX=0,wrapY=0,filter=0,key=0}";
        let image = |opacity: f32| {
            format!("drawImage image=2 sampler={SAMPLER} blendMode=3 opacity={opacity}")
        };
        let mesh = |opacity: f32| {
            format!(
                "drawImageMesh image=2 sampler={SAMPLER} vertices=3 uvs=4 indices=5 \
                 vertexCount=3 indexCount=3 blendMode=3 opacity={opacity}"
            )
        };
        let square = RawPath::from_verbs_and_points(
            vec![PathVerb::Move, PathVerb::Line, PathVerb::Close],
            vec![Vec2D::new(1.0, 1.0), Vec2D::new(2.0, 1.0)],
        );
        let opacity_of = |ops: Vec<StreamOp>| {
            let draws = frame_draws(&ops, &[]);
            assert_eq!(draws.len(), 1);
            draws[0].opacity
        };
        let recorded = |line: String| {
            let mut ops = vec![StreamOp::ModulateOpacity(0.5)];
            ops.extend(stream_ops(&line).unwrap());
            opacity_of(ops)
        };

        assert_eq!(recorded(image(-0.2)), 0.0);
        assert_eq!(recorded(mesh(-0.2)), 0.0);
        // A frame without image paints and every mesh draw multiply first. A
        // frame with them holds a `drawImage` at 1 first, so reads 0.5 here; the
        // stream doesn't name the frame's mode. See `replay`.
        assert_eq!(recorded(image(1.5)), 0.75);
        assert_eq!(recorded(mesh(1.5)), 0.75);
        assert_eq!(recorded(image(4.0)), 1.0);
        assert_eq!(recorded(mesh(4.0)), 1.0);
        assert_eq!(
            opacity_of(vec![
                StreamOp::ModulateOpacity(2.0),
                StreamOp::DrawImage { opacity: 0.25 },
            ]),
            0.5,
        );
        assert_eq!(
            opacity_of(vec![
                StreamOp::ModulateOpacity(2.0),
                StreamOp::DrawPath {
                    path: square,
                    color: 0xff00_0000,
                    shader: 0,
                    stops_alpha: None,
                },
            ]),
            1.0,
        );
    }

    fn fill(color: ColorInt, shader: u64) -> StreamOp {
        StreamOp::DrawPath {
            path: RawPath::from_verbs_and_points(
                vec![PathVerb::Move, PathVerb::Line, PathVerb::Close],
                vec![Vec2D::new(1.0, 1.0), Vec2D::new(2.0, 1.0)],
            ),
            color,
            shader,
            stops_alpha: None,
        }
    }
    fn image(opacity: f32) -> StreamOp {
        StreamOp::DrawImage { opacity }
    }
    fn composite(canvas: u64, opacity: f32) -> StreamOp {
        StreamOp::Composite {
            canvas,
            matrix: Mat2D::IDENTITY,
            opacity,
        }
    }
    fn canvas(id: u64, ops: Vec<StreamOp>) -> Canvas {
        Canvas { id, ops }
    }
    #[track_caller]
    fn reads(ops: &[StreamOp], canvases: &[Canvas], expected: &[f32]) {
        let opacities: Vec<f32> = frame_draws(ops, canvases)
            .iter()
            .map(|draw| draw.opacity)
            .collect();
        assert_eq!(opacities.len(), expected.len(), "{opacities:?}");
        for (read, expected) in opacities.iter().zip(expected) {
            assert!((read - expected).abs() < 1e-5, "{opacities:?}, {expected}");
        }
    }
    /// The commands of a recorded stream, as text a test writes by hand.
    fn lines(recorded: &[&str]) -> Vec<StreamOp> {
        stream_ops(&recorded.join("\n")).unwrap()
    }
    const HALF: f32 = 128.0 / 255.0;
    const QUARTER: f32 = 64.0 / 255.0;
    const COLOR_HALF: &str = "modulateColor color=0x80ffffff replace=0";
    const COLOR_QUARTER: &str = "modulateColor color=0x40ffffff replace=0";
    const COLOR_QUARTER_REPLACED: &str = "modulateColor color=0x40ffffff replace=1";
    const WHITE: ColorInt = 0xffff_ffff;

    /// A canvas has its own renderer, which starts at a scope of 1 and holds
    /// each draw from 0 to 1 by that scope alone. The picture is one more draw
    /// on the renderer under it, at its own opacity and held the same way.
    #[test]
    fn a_canvas_holds_its_draws_by_its_own_scope() {
        // Inside: 1.5 held at 1. The picture: 0.5 under a scope of 1.
        reads(&[composite(1, 0.5)], &[canvas(1, vec![image(1.5)])], &[0.5]);
        reads(
            &[composite(1, 0.5)],
            &[canvas(
                1,
                vec![StreamOp::ModulateOpacity(1.5), fill(WHITE, 0)],
            )],
            &[0.5],
        );
        // The picture: 2 times 0.75 held at 1. So each draw reads its own.
        let over = [StreamOp::ModulateOpacity(2.0), composite(1, 0.75)];
        reads(&over, &[canvas(1, vec![image(0.25)])], &[0.25]);
        reads(&over, &[canvas(1, vec![fill(0x8000_0000, 0)])], &[HALF]);
        // A negative composite draws at 0, whatever is inside.
        reads(
            &[composite(1, -0.5)],
            &[canvas(1, vec![image(0.5), fill(WHITE, 0)])],
            &[0.0, 0.0],
        );
        // The canvas's scope is its own: the draw after the picture has none of
        // it, and the canvas has none of the scope before it except through the
        // picture.
        reads(
            &[
                StreamOp::ModulateOpacity(0.5),
                composite(1, 1.0),
                image(1.0),
            ],
            &[canvas(1, vec![StreamOp::ModulateOpacity(0.5), image(1.0)])],
            &[0.25, 0.5],
        );
    }

    #[test]
    fn a_canvas_inside_a_canvas_is_held_at_each_level() {
        let inner = |ops| {
            vec![
                canvas(
                    1,
                    vec![
                        StreamOp::ModulateOpacity(3.0),
                        composite(2, 0.5),
                        image(0.1),
                    ],
                ),
                canvas(2, ops),
            ]
        };
        // The first picture: 0.5. The second, on the first's renderer: 3 times
        // 0.5 held at 1, so 0.5 through. Inside it, 1.5 times the draw's own.
        // Then the first canvas's own draw: 3 times 0.1, through 0.5.
        reads(
            &[composite(1, 0.5)],
            &inner(vec![StreamOp::ModulateOpacity(1.5), image(0.5)]),
            &[0.375, 0.15],
        );
        reads(
            &[composite(1, 0.5)],
            &inner(vec![StreamOp::ModulateOpacity(1.5), fill(0x8000_0000, 0)]),
            &[1.5 * HALF * 0.5, 0.15],
        );
        // A scope of 2 over the first picture at 0.75 holds it at 1. The second
        // is at 0.5, and 1.5 inside it holds at 1.
        reads(
            &[StreamOp::ModulateOpacity(2.0), composite(1, 0.75)],
            &[
                canvas(1, vec![composite(2, 0.5)]),
                canvas(2, vec![image(1.5), fill(WHITE, 0)]),
            ],
            &[0.5, 0.5],
        );
    }

    /// `color_modulate` multiplies the scope's color alpha in with the scope's
    /// opacity and the draw's own, and holds the whole from 0 to 1.
    /// `modulateColor` multiplies into that alpha, or with `replace` sets it.
    #[test]
    fn modulate_color_alpha_joins_the_scope() {
        let solid = "drawPath path={id=1,fillRule=0,path={verbs=[move,line,close],points=[(1,1),(2,1)]}} \
                     paint={id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}";
        reads(&lines(&[COLOR_HALF, solid]), &[], &[HALF]);
        let halfway =
            "drawImage image=2 sampler={wrapX=0,wrapY=0,filter=0,key=0} blendMode=3 opacity=0.5";
        reads(&lines(&[COLOR_HALF, halfway]), &[], &[0.5 * HALF]);
        reads(
            &lines(&["modulateOpacity opacity=0.5", COLOR_HALF, solid]),
            &[],
            &[0.5 * HALF],
        );
        // A scope above 1: the whole is held, not the opacity alone.
        reads(
            &lines(&["modulateOpacity opacity=4", COLOR_HALF, solid]),
            &[],
            &[1.0],
        );
        reads(
            &lines(&[COLOR_HALF, COLOR_QUARTER, solid]),
            &[],
            &[HALF * QUARTER],
        );
        reads(
            &lines(&[COLOR_HALF, COLOR_QUARTER_REPLACED, solid]),
            &[],
            &[QUARTER],
        );
        reads(&lines(&["save", COLOR_HALF, "restore", solid]), &[], &[1.0]);

        // A canvas's renderer starts at a color of white. The color in scope
        // over a picture counts on the picture, as on any draw.
        let outer = |recorded: &[&str]| {
            let mut ops = lines(recorded);
            ops.push(composite(1, 0.5));
            ops
        };
        reads(
            &outer(&[COLOR_HALF]),
            &[canvas(1, lines(&[COLOR_HALF, halfway]))],
            &[0.5 * HALF * 0.5 * HALF],
        );
        // Inside: 4 times its color's half, held at 1. The picture is at 0.5.
        reads(
            &outer(&[]),
            &[canvas(
                1,
                lines(&["modulateOpacity opacity=4", COLOR_HALF, solid]),
            )],
            &[0.5],
        );
        // The color over the picture doesn't reach inside on its own: replacing
        // it inside changes only what's inside.
        reads(
            &outer(&[COLOR_HALF]),
            &[canvas(1, lines(&[COLOR_QUARTER_REPLACED, solid]))],
            &[QUARTER * 0.5 * HALF],
        );
    }

    /// The opacity of one recorded fill under a scope of `scope`, its paint at
    /// `paint_color`. A shader's stops are given, or there is no shader. With
    /// `over_image` the paint also holds an image, the paint a frame without
    /// image paints draws as an image rect.
    fn recorded_fill(
        paint_color: ColorInt,
        stops: Option<&[ColorInt]>,
        radial: bool,
        over_image: bool,
        scope: f32,
    ) -> Vec<FrameDraw> {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut paint = factory.make_render_paint();
        paint.color(paint_color);
        let shader = stops.map(|colors| {
            let offsets: Vec<f32> = (0..colors.len())
                .map(|index| index as f32 / (colors.len() - 1) as f32)
                .collect();
            if radial {
                factory.make_radial_gradient(0.0, 0.0, 1.0, colors, &offsets)
            } else {
                factory.make_linear_gradient(0.0, 0.0, 1.0, 0.0, colors, &offsets)
            }
        });
        paint.shader(shader.as_deref());
        if over_image {
            let picture = factory.decode_image(&[0x89, 0x50, 0x4e, 0x47]).unwrap();
            paint.modulated_image(
                Some(picture.as_ref()),
                ImageSampler::LINEAR_CLAMP,
                Mat2D::IDENTITY,
            );
        }
        let square = RawPath::from_verbs_and_points(
            vec![PathVerb::Move, PathVerb::Line, PathVerb::Close],
            vec![Vec2D::new(1.0, 1.0), Vec2D::new(2.0, 1.0)],
        );
        let path = factory.make_render_path(square, FillRule::NonZero);
        renderer.modulate_opacity(scope);
        renderer.draw_path(path.as_ref(), paint.as_ref());
        frame_draws(&stream_ops(&factory.stream()).unwrap(), &[])
    }

    /// `shader` sets the renderer's paint to a gradient and drops its color
    /// (`RiveRenderPaint::shader_source`), and `getModulated` holds each stop
    /// by `color_modulate`. So a gradient's alpha is its stops', and the paint
    /// alpha the recording keeps beside the shader isn't used. Without an image
    /// paint, an image rect takes white in place of the color, over the stops
    /// (`RiveRenderer::drawPath`), so it reads the same.
    #[test]
    fn a_gradient_reads_its_stops_and_not_its_paints_alpha() {
        const OPAQUE: [ColorInt; 2] = [0xff00_0000, 0xffff_ffff];
        let read = |paint, stops: Option<&[ColorInt]>, radial, over_image, scope| {
            let draws = recorded_fill(paint, stops, radial, over_image, scope);
            assert_eq!(draws.len(), 1);
            assert_eq!(draws[0].color, Some(paint), "the paint as recorded");
            draws[0].opacity
        };
        let near = |read: f32, expected: f32| {
            assert!((read - expected).abs() < 1e-5, "{read}, {expected}");
        };

        for radial in [false, true] {
            for over_image in [false, true] {
                near(
                    read(0x4000_0000, Some(&OPAQUE), radial, over_image, 0.5),
                    0.5,
                );
                // Stops of 0x80 and a scope of 0.5 multiply.
                near(
                    read(
                        0x4000_0000,
                        Some(&[0x8000_0000, 0x8000_0000]),
                        radial,
                        over_image,
                        0.5,
                    ),
                    0.5 * HALF,
                );
            }
        }
        // Stops that differ read the strongest, as drawn.
        near(
            read(WHITE, Some(&[0x4000_0000, 0x8000_0000]), false, false, 0.5),
            0.5 * HALF,
        );
        near(
            read(
                0x0000_0000,
                Some(&[0x8000_0000, 0x8000_0000]),
                false,
                false,
                4.0,
            ),
            1.0,
        );

        // A plain paint uses its alpha, also as an image rect over no gradient.
        near(read(0x4000_0000, None, false, false, 0.5), 0.5 * QUARTER);
        near(read(0x4000_0000, None, false, true, 0.5), 0.5 * QUARTER);

        // A stream cleared after the gradient was made has no stops to read, so
        // only the paint's unused alpha is known. It's counted as opaque.
        let solid_shader = "drawPath path={id=1,fillRule=0,path={verbs=[move,line,close],points=[(1,1),(2,1)]}} \
                            paint={id=1,style=fill,color=0x40000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=7}";
        reads(
            &lines(&["modulateOpacity opacity=0.5", solid_shader]),
            &[],
            &[0.5],
        );
    }

    #[test]
    fn stream_ops_reads_only_what_was_drawn() {
        let ops = stream_ops(STREAM).unwrap();
        assert_eq!(ops.len(), 4);
        let draws = frame_draws(&ops, &[]);
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].matrix, Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0]));
        assert_eq!(draws[0].rgb(), Some(0x00ff00));
        assert_eq!(draws[0].shader, 7);
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
