//! renderer/src/deferred_cmd.cpp at e949498e.
use super::{
    command_stream::{CommandReader, WirePod},
    render_commands::*,
    render_handle::*,
    render_replay::*,
};
use nuxie_render_api::*;
use std::{cell::RefCell, rc::Rc};

pub fn sniff_image_size(data: &[u8]) -> Option<(u32, u32)> {
    let n = data.len();
    let be32 = |i| u32::from_be_bytes(data[i..i + 4].try_into().unwrap());
    if n >= 24 && data[..4] == [0x89, b'P', b'N', b'G'] {
        return Some((be32(16), be32(20)));
    }
    if n >= 10 && data[..3] == *b"GIF" {
        return Some((
            u32::from(u16::from_le_bytes([data[6], data[7]])),
            u32::from(u16::from_le_bytes([data[8], data[9]])),
        ));
    }
    if n >= 30 && data[..4] == *b"RIFF" && data[8..12] == *b"WEBP" {
        if data[12..16] == *b"VP8 " {
            return Some((
                (u32::from(data[26]) | (u32::from(data[27]) << 8)) & 0x3fff,
                (u32::from(data[28]) | (u32::from(data[29]) << 8)) & 0x3fff,
            ));
        }
        if data[12..16] == *b"VP8L" {
            let bits = u32::from_le_bytes(data[21..25].try_into().unwrap());
            return Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1));
        }
        if data[12..16] == *b"VP8X" {
            return Some((
                (u32::from(data[24]) | (u32::from(data[25]) << 8) | (u32::from(data[26]) << 16))
                    + 1,
                (u32::from(data[27]) | (u32::from(data[28]) << 8) | (u32::from(data[29]) << 16))
                    + 1,
            ));
        }
    }
    if n >= 4 && data[..2] == [0xff, 0xd8] {
        let mut i = 2;
        while i + 9 < n {
            if data[i] != 0xff {
                i += 1;
                continue;
            }
            let marker = data[i + 1];
            if (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker) {
                return Some((
                    (u32::from(data[i + 7]) << 8) | u32::from(data[i + 8]),
                    (u32::from(data[i + 5]) << 8) | u32::from(data[i + 6]),
                ));
            }
            let segment = (usize::from(data[i + 2]) << 8) | usize::from(data[i + 3]);
            i += 2 + segment;
        }
    }
    None
}
fn fill_rule(value: u8) -> FillRule {
    match value {
        0 => FillRule::NonZero,
        1 => FillRule::EvenOdd,
        2 => FillRule::Clockwise,
        _ => panic!("invalid recorded fill rule"),
    }
}
fn style(value: u8) -> RenderPaintStyle {
    match value {
        0 => RenderPaintStyle::Stroke,
        1 => RenderPaintStyle::Fill,
        _ => panic!("invalid recorded paint style"),
    }
}
fn join(value: u8) -> StrokeJoin {
    match value {
        0 => StrokeJoin::Miter,
        1 => StrokeJoin::Round,
        2 => StrokeJoin::Bevel,
        _ => panic!("invalid recorded join"),
    }
}
fn cap(value: u8) -> StrokeCap {
    match value {
        0 => StrokeCap::Butt,
        1 => StrokeCap::Round,
        2 => StrokeCap::Square,
        _ => panic!("invalid recorded cap"),
    }
}
fn blend(value: u8) -> BlendMode {
    match value {
        3 => BlendMode::SrcOver,
        14 => BlendMode::Screen,
        15 => BlendMode::Overlay,
        16 => BlendMode::Darken,
        17 => BlendMode::Lighten,
        18 => BlendMode::ColorDodge,
        19 => BlendMode::ColorBurn,
        20 => BlendMode::HardLight,
        21 => BlendMode::SoftLight,
        22 => BlendMode::Difference,
        23 => BlendMode::Exclusion,
        24 => BlendMode::Multiply,
        25 => BlendMode::Hue,
        26 => BlendMode::Saturation,
        27 => BlendMode::Color,
        28 => BlendMode::Luminosity,
        _ => panic!("invalid recorded blend"),
    }
}
fn buffer_type(value: u8) -> RenderBufferType {
    match value {
        0 => RenderBufferType::Index,
        1 => RenderBufferType::Vertex,
        _ => panic!("invalid recorded buffer type"),
    }
}
fn buffer_flags(value: u16) -> RenderBufferFlags {
    match value {
        0 => RenderBufferFlags::None,
        1 => RenderBufferFlags::MappedOnceAtInitialization,
        _ => panic!("invalid recorded buffer flags"),
    }
}
fn sampler(x: u8, y: u8, filter: u8) -> ImageSampler {
    fn wrap(v: u8) -> ImageWrap {
        match v {
            0 => ImageWrap::Clamp,
            1 => ImageWrap::Repeat,
            2 => ImageWrap::Mirror,
            _ => panic!("invalid recorded image wrap"),
        }
    }
    ImageSampler {
        wrap_x: wrap(x),
        wrap_y: wrap(y),
        filter: match filter {
            0 => ImageFilter::Bilinear,
            1 => ImageFilter::Nearest,
            _ => panic!("invalid recorded image filter"),
        },
    }
}
fn resource_kind(value: u8) -> Option<ResourceKind> {
    match value {
        0 => Some(ResourceKind::Path),
        1 => Some(ResourceKind::Paint),
        2 => Some(ResourceKind::Shader),
        3 => Some(ResourceKind::Image),
        4 => Some(ResourceKind::Buffer),
        _ => None,
    }
}
fn filter_allows(filter: ReplayFilter, command: RenderCmd) -> bool {
    if filter == ReplayFilter::All {
        return true;
    }
    match command {
        RenderCmd::Save
        | RenderCmd::Restore
        | RenderCmd::Transform
        | RenderCmd::DrawPath
        | RenderCmd::ClipPath
        | RenderCmd::DrawImage
        | RenderCmd::DrawImageMesh
        | RenderCmd::ModulateOpacity
        | RenderCmd::CanvasContentBegin
        | RenderCmd::CanvasContentEnd => filter == ReplayFilter::Draws,
        RenderCmd::DestroyResource => filter == ReplayFilter::Destroys,
        _ => filter == ReplayFilter::Resources,
    }
}
fn scalars<T: WirePod>(bytes: &[u8]) -> Vec<T> {
    bytes.chunks_exact(T::SIZE).map(T::decode).collect()
}
fn with_renderer(
    screen: &mut Option<&mut dyn Renderer>,
    canvas: &Option<RendererOwner>,
    in_canvas: bool,
    f: impl FnOnce(&mut dyn Renderer),
) -> bool {
    if in_canvas {
        if let Some(canvas) = canvas {
            f(canvas.borrow_mut().as_mut());
            return true;
        }
    } else if let Some(screen) = screen.as_deref_mut() {
        f(screen);
        return true;
    }
    false
}
fn dropped(hooks: &mut ReplayHooks<'_>, kind: u8, a: u32, b: u32) {
    if let Some(stats) = hooks.stats.as_deref_mut() {
        stats.dropped_draws = stats.dropped_draws.wrapping_add(1);
        static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        if COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 120 == 0 {
            eprintln!(
                "rive replay: dropped draw opcode={kind} handles={a},{b} (unresolved at replay)"
            );
        }
    }
}

pub fn replay_render_commands(
    factory: &mut dyn Factory,
    mut renderer: Option<&mut dyn Renderer>,
    commands: &[u8],
    blobs: &[u8],
    table: &mut ResourceTable,
    hooks: &mut ReplayHooks<'_>,
) {
    let mut reader = CommandReader::new(commands, blobs);
    let mut current_canvas: Option<RendererOwner> = None;
    let mut in_canvas = false;
    let mut previous_type = 255;
    let mut previous_pos = 0;
    while let Some(kind) = reader.next_u8() {
        let Some(command) = RenderCmd::from_byte(kind) else {
            eprintln!(
                "rive replay ABORT: opcode {kind} at byte {} of {}, last good opcode {previous_type} at byte {previous_pos}",
                reader.position() - 1,
                commands.len()
            );
            debug_assert!(false, "unknown replay opcode");
            break;
        };
        previous_type = kind;
        previous_pos = reader.position() - 1;
        if !filter_allows(hooks.filter, command) {
            reader.skip(payload_size_of(command));
            continue;
        }
        match command {
            RenderCmd::MakePath => {
                let c: MakePathPod = reader.read();
                let raw = rebuild_raw_path(
                    reader.blob_at(c.blob_offset, c.verb_count),
                    reader.blob_at(c.points_offset, c.point_count.wrapping_mul(8)),
                );
                table.paths.set(
                    c.id,
                    Some(Rc::new(RefCell::new(
                        factory.make_render_path(raw, fill_rule(c.fill_rule as u8)),
                    ))),
                    c.generation,
                );
                table
                    .path_fill_rules
                    .resize(table.path_fill_rules.len().max(c.id as usize + 1), 0);
                table.path_fill_rules[c.id as usize] = c.fill_rule as u8;
            }
            RenderCmd::MakeEmptyPath => {
                let c: MakeIdPod = reader.read();
                table.paths.set(
                    c.id,
                    Some(Rc::new(RefCell::new(factory.make_empty_render_path()))),
                    c.generation,
                );
                table
                    .path_fill_rules
                    .resize(table.path_fill_rules.len().max(c.id as usize + 1), 0);
                table.path_fill_rules[c.id as usize] = 0;
            }
            RenderCmd::MakePaint => {
                let c: MakeIdPod = reader.read();
                table.paints.set(
                    c.id,
                    Some(Rc::new(RefCell::new(factory.make_render_paint()))),
                    c.generation,
                );
                table.paint_shadows.resize(
                    table.paint_shadows.len().max(c.id as usize + 1),
                    PaintShadow::default(),
                );
                table.paint_shadows[c.id as usize] = PaintShadow::default();
            }
            RenderCmd::MakeLinearGradient => {
                let c: LinearGradientPod = reader.read();
                let bytes = c.count.wrapping_mul(4);
                let colors = reader.blob_at(c.blob_offset, bytes);
                let stops = reader.blob_at(c.stops_offset, bytes);
                if colors.len() != bytes as usize || stops.len() != bytes as usize {
                    continue;
                }
                table.shaders.set(
                    c.id,
                    Some(Rc::from(factory.make_linear_gradient(
                        c.sx,
                        c.sy,
                        c.ex,
                        c.ey,
                        &scalars::<u32>(colors),
                        &scalars::<f32>(stops),
                    ))),
                    c.generation,
                );
            }
            RenderCmd::MakeRadialGradient => {
                let c: RadialGradientPod = reader.read();
                let bytes = c.count.wrapping_mul(4);
                let colors = reader.blob_at(c.blob_offset, bytes);
                let stops = reader.blob_at(c.stops_offset, bytes);
                if colors.len() != bytes as usize || stops.len() != bytes as usize {
                    continue;
                }
                table.shaders.set(
                    c.id,
                    Some(Rc::from(factory.make_radial_gradient(
                        c.cx,
                        c.cy,
                        c.radius,
                        &scalars::<u32>(colors),
                        &scalars::<f32>(stops),
                    ))),
                    c.generation,
                );
            }
            RenderCmd::DecodeImage => {
                let c: DecodeImagePod = reader.read();
                table.images.set(
                    c.id,
                    factory
                        .decode_image(reader.blob_at(c.blob_offset, c.byte_count))
                        .ok()
                        .map(Rc::from),
                    c.generation,
                );
            }
            RenderCmd::MakeBuffer => {
                let c: MakeBufferPod = reader.read();
                table.buffers.set(
                    c.id,
                    Some(Rc::new(RefCell::new(factory.make_render_buffer(
                        buffer_type(c.buffer_type),
                        buffer_flags(c.flags as u16),
                        c.size_in_bytes as usize,
                    )))),
                    c.generation,
                );
                table.buffer_shadows.resize(
                    table.buffer_shadows.len().max(c.id as usize + 1),
                    BufferShadow::default(),
                );
                table.buffer_shadows[c.id as usize] = BufferShadow {
                    buffer_type: c.buffer_type,
                    flags: c.flags as u16,
                    size: c.size_in_bytes,
                };
            }
            RenderCmd::BufferData => {
                let c: BufferDataPod = reader.read();
                let source = reader.blob_at(c.blob_offset, c.size);
                if let Some(buffer) = table.buffers.get(c.buffer) {
                    if source.len() == c.size as usize {
                        let mut buffer = buffer.borrow_mut();
                        let dest = buffer.map_mut();
                        if !dest.is_empty() {
                            dest[..source.len()].copy_from_slice(source);
                        }
                        buffer.unmap();
                    }
                }
            }
            RenderCmd::DestroyResource => {
                let c: DestroyResourcePod = reader.read();
                if let Some(kind) = resource_kind(c.kind) {
                    table.destroy(kind, c.id, c.generation);
                }
            }
            RenderCmd::ResourceNewVersion => {
                let c: ResourceVersionPod = reader.read();
                match resource_kind(c.kind) {
                    Some(ResourceKind::Paint) => {
                        let mut fresh = factory.make_render_paint();
                        if let Some(shadow) = table.paint_shadows.get(c.id as usize) {
                            fresh.style(style(shadow.style));
                            fresh.color(shadow.color);
                            fresh.thickness(shadow.thickness);
                            fresh.join(join(shadow.join));
                            fresh.cap(cap(shadow.cap));
                            fresh.feather(shadow.feather);
                            fresh.blend_mode(blend(shadow.blend_mode));
                            if shadow.shader != INVALID_RENDER_HANDLE {
                                fresh.shader(table.shaders.get(shadow.shader).as_deref());
                            }
                        }
                        table.paints.new_version(
                            c.id,
                            c.version,
                            Some(Rc::new(RefCell::new(fresh))),
                        );
                    }
                    Some(ResourceKind::Path) => {
                        let mut fresh = factory.make_empty_render_path();
                        if let Some(outgoing) = table.paths.get(c.id) {
                            fresh.add_render_path(outgoing.borrow().as_ref(), Mat2D::IDENTITY);
                        }
                        if let Some(&rule) = table.path_fill_rules.get(c.id as usize) {
                            fresh.fill_rule(fill_rule(rule));
                        }
                        table.paths.new_version(
                            c.id,
                            c.version,
                            Some(Rc::new(RefCell::new(fresh))),
                        );
                    }
                    Some(ResourceKind::Buffer) => {
                        let fresh = table.buffer_shadows.get(c.id as usize).map(|shadow| {
                            Rc::new(RefCell::new(factory.make_render_buffer(
                                buffer_type(shadow.buffer_type),
                                buffer_flags(shadow.flags),
                                shadow.size as usize,
                            )))
                        });
                        table.buffers.new_version(c.id, c.version, fresh);
                    }
                    _ => {}
                }
            }
            RenderCmd::PathRewind => {
                let c: ResIdPod = reader.read();
                if let Some(path) = table.paths.get(c.id) {
                    path.borrow_mut().rewind();
                }
            }
            RenderCmd::PathFillRule => {
                let c: PathFillRulePod = reader.read();
                if let Some(path) = table.paths.get(c.path) {
                    path.borrow_mut().fill_rule(fill_rule(c.fill_rule));
                    table.path_fill_rules[c.path as usize] = c.fill_rule;
                }
            }
            RenderCmd::PathAddRawPath => {
                let c: PathRawPod = reader.read();
                let raw = rebuild_raw_path(
                    reader.blob_at(c.blob_offset, c.verb_count),
                    reader.blob_at(c.points_offset, c.point_count.wrapping_mul(8)),
                );
                if let Some(path) = table.paths.get(c.path) {
                    path.borrow_mut().add_raw_path(&raw);
                }
            }
            RenderCmd::PathAddRenderPath => {
                let c: PathAddPathPod = reader.read();
                if let (Some(path), Some(source)) =
                    (table.paths.get(c.path), table.paths.get(c.src))
                {
                    let transform = Mat2D([c.xx, c.xy, c.yx, c.yy, c.tx, c.ty]);
                    if Rc::ptr_eq(&path, &source) {
                        path.borrow_mut().add_render_path_self(transform);
                    } else {
                        let mut destination = path.borrow_mut();
                        let source = source.borrow();
                        destination.add_render_path(source.as_ref(), transform);
                    }
                }
            }
            RenderCmd::PaintStyle
            | RenderCmd::PaintJoin
            | RenderCmd::PaintCap
            | RenderCmd::PaintBlendMode => {
                let c: PaintU8Pod = reader.read();
                if let Some(paint) = table.paints.get(c.paint) {
                    let mut paint = paint.borrow_mut();
                    let shadow = &mut table.paint_shadows[c.paint as usize];
                    match command {
                        RenderCmd::PaintStyle => {
                            paint.style(style(c.value));
                            shadow.style = c.value;
                        }
                        RenderCmd::PaintJoin => {
                            paint.join(join(c.value));
                            shadow.join = c.value;
                        }
                        RenderCmd::PaintCap => {
                            paint.cap(cap(c.value));
                            shadow.cap = c.value;
                        }
                        _ => {
                            paint.blend_mode(blend(c.value));
                            shadow.blend_mode = c.value;
                        }
                    }
                }
            }
            RenderCmd::PaintColor => {
                let c: PaintColorPod = reader.read();
                if let Some(paint) = table.paints.get(c.paint) {
                    paint.borrow_mut().color(c.color);
                    table.paint_shadows[c.paint as usize].color = c.color;
                }
            }
            RenderCmd::PaintThickness | RenderCmd::PaintFeather => {
                let c: PaintFloatPod = reader.read();
                if let Some(paint) = table.paints.get(c.paint) {
                    if command == RenderCmd::PaintThickness {
                        paint.borrow_mut().thickness(c.value);
                        table.paint_shadows[c.paint as usize].thickness = c.value;
                    } else {
                        paint.borrow_mut().feather(c.value);
                        table.paint_shadows[c.paint as usize].feather = c.value;
                    }
                }
            }
            RenderCmd::PaintShader => {
                let c: PaintShaderPod = reader.read();
                if let Some(paint) = table.paints.get(c.paint) {
                    paint
                        .borrow_mut()
                        .shader(table.shaders.get(c.shader).as_deref());
                    table.paint_shadows[c.paint as usize].shader = c.shader;
                }
            }
            RenderCmd::PaintInvalidateStroke => {
                let c: ResIdPod = reader.read();
                if let Some(paint) = table.paints.get(c.id) {
                    paint.borrow_mut().invalidate_stroke();
                }
            }
            RenderCmd::Save => {
                with_renderer(&mut renderer, &current_canvas, in_canvas, |r| r.save());
            }
            RenderCmd::Restore => {
                with_renderer(&mut renderer, &current_canvas, in_canvas, |r| r.restore());
            }
            RenderCmd::Transform => {
                let c: TransformPod = reader.read();
                with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    r.transform(Mat2D([c.xx, c.xy, c.yx, c.yy, c.tx, c.ty]))
                });
            }
            RenderCmd::DrawPath => {
                let c: DrawPathPod = reader.read();
                let path = table.paths.get_version(c.path, c.path_version);
                let paint = table.paints.get_version(c.paint, c.paint_version);
                let present = with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    if let (Some(path), Some(paint)) = (&path, &paint) {
                        r.draw_path(path.borrow().as_ref(), paint.borrow().as_ref());
                    }
                });
                if present && (path.is_none() || paint.is_none()) {
                    dropped(hooks, kind, c.path, c.paint);
                }
            }
            RenderCmd::ClipPath => {
                let c: ClipPathPod = reader.read();
                with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    if let Some(path) = table.paths.get_version(c.path, c.version) {
                        r.clip_path(path.borrow().as_ref());
                    }
                });
            }
            RenderCmd::DrawImage => {
                let c: DrawImagePod = reader.read();
                let image = if c.image & CANVAS_HANDLE_FLAG != 0 {
                    hooks
                        .canvas_image
                        .as_mut()
                        .and_then(|resolve| resolve(c.image & CANVAS_HANDLE_MASK))
                } else {
                    table.images.get(c.image)
                };
                let present = with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    if let Some(image) = &image {
                        r.draw_image(
                            Some(image.as_ref()),
                            sampler(c.wrap_x, c.wrap_y, c.filter),
                            blend(c.blend_mode),
                            c.opacity,
                        );
                    }
                });
                if present && image.is_none() {
                    dropped(hooks, kind, c.image, 0);
                }
            }
            RenderCmd::DrawImageMesh => {
                let c: DrawImageMeshPod = reader.read();
                let image = if c.image & CANVAS_HANDLE_FLAG != 0 {
                    hooks
                        .canvas_image
                        .as_mut()
                        .and_then(|resolve| resolve(c.image & CANVAS_HANDLE_MASK))
                } else {
                    table.images.get(c.image)
                };
                let vertices = table.buffers.get_version(c.vertices, c.vertex_version);
                let uv = table.buffers.get_version(c.uv_coords, c.uv_version);
                let indices = table.buffers.get_version(c.indices, c.index_version);
                let present = with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    if let (Some(image), Some(vertices), Some(uv), Some(indices)) =
                        (&image, &vertices, &uv, &indices)
                    {
                        r.draw_image_mesh(
                            Some(image.as_ref()),
                            sampler(c.wrap_x, c.wrap_y, c.filter),
                            Some(vertices.borrow().as_ref()),
                            Some(uv.borrow().as_ref()),
                            Some(indices.borrow().as_ref()),
                            c.vertex_count,
                            c.index_count,
                            blend(c.blend_mode),
                            c.opacity,
                        );
                    }
                });
                if present
                    && (image.is_none() || vertices.is_none() || uv.is_none() || indices.is_none())
                {
                    dropped(hooks, kind, c.image, 0);
                }
            }
            RenderCmd::ModulateOpacity => {
                let c: OpacityPod = reader.read();
                with_renderer(&mut renderer, &current_canvas, in_canvas, |r| {
                    r.modulate_opacity(c.opacity)
                });
            }
            RenderCmd::CanvasContentBegin => {
                let c: CanvasContentPod = reader.read();
                current_canvas = hooks
                    .begin_canvas_content
                    .as_mut()
                    .and_then(|begin| begin(c.canvas_id & CANVAS_HANDLE_MASK, c.clear_color));
                in_canvas = true;
            }
            RenderCmd::CanvasContentEnd => {
                let _: ResIdPod = reader.read();
                current_canvas = None;
                in_canvas = false;
            }
        }
    }
    if reader.overrun() {
        eprintln!(
            "rive replay ABORT: payload overrun at byte {} of {}",
            reader.position(),
            commands.len()
        );
        debug_assert!(false, "replay payload overrun");
    }
}
