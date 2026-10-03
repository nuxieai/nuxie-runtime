//! `include/utils/serialized_replay.hpp` + `utils/serialized_replay.cpp`.
use crate::serialize_ops::*;
use crate::*;
use std::collections::HashMap;
use std::rc::Rc;

/// Owned renderer proxies preserve the upstream callback's renderer identity
/// without extending a borrowed host renderer's lifetime.
pub type CanvasContentBeginHook<'a> = dyn FnMut(u64, u32, u32, u32, &mut Option<Rc<dyn RenderImage>>) -> Option<Box<dyn Renderer + 'a>>
    + 'a;

#[derive(Default)]
pub struct SerializedReplayHooks<'a> {
    pub on_frame: Option<Box<dyn FnMut() + 'a>>,
    pub on_frame_size: Option<Box<dyn FnMut(u32, u32) + 'a>>,
    pub on_canvas_content_begin: Option<Box<CanvasContentBeginHook<'a>>>,
    pub on_canvas_content_end: Option<Box<dyn FnMut(u64) + 'a>>,
}

enum ReplayTarget<'a> {
    Screen,
    Content(Box<dyn Renderer + 'a>),
    Dropped,
}
impl ReplayTarget<'_> {
    fn renderer<'r>(
        &'r mut self,
        screen: &'r mut dyn Renderer,
        dropped: &'r mut dyn Renderer,
    ) -> &'r mut dyn Renderer {
        match self {
            Self::Screen => screen,
            Self::Content(renderer) => renderer.as_mut(),
            Self::Dropped => dropped,
        }
    }
}

/// Replay through the Factory/Renderer seam. Failure retains all effects that
/// preceded it; a corrupt opcode is never retried through another renderer.
pub fn replay_serialized_commands(
    stream: &[u8],
    factory: &mut dyn Factory,
    renderer: &mut dyn Renderer,
    hooks: &mut SerializedReplayHooks<'_>,
) -> bool {
    let mut reader = Reader::new(stream);
    if reader.read_byte() != b'S'
        || reader.read_byte() != b'R'
        || reader.read_byte() != b'I'
        || reader.read_byte() != b'V'
    {
        return false;
    }
    if reader.read_var_uint() != 1 {
        return false;
    }
    let mut paths: HashMap<u64, Box<dyn RenderPath>> = HashMap::new();
    let mut paints: HashMap<u64, Box<dyn RenderPaint>> = HashMap::new();
    let mut shaders: HashMap<u64, Box<dyn RenderShader>> = HashMap::new();
    let mut images: HashMap<u64, Option<Rc<dyn RenderImage>>> = HashMap::new();
    let mut buffers: HashMap<u64, Box<dyn RenderBuffer>> = HashMap::new();
    let mut mesh_instances: HashMap<u64, ImageMeshInstancesHandle> = HashMap::new();
    let mut canvas_sizes = HashMap::new();
    let mut dropped_content = NullFactory::new().make_renderer();
    let mut active = ReplayTarget::Screen;
    let mut interrupted = Vec::new();
    while !reader.is_eof() && !reader.did_overflow() {
        let op = reader.read_var_uint() as u32 as u64;
        if reader.did_overflow() {
            return false;
        }
        match op {
            MAKE_RENDER_PATH => {
                let id = reader.read_var_uint();
                paths.insert(id, factory.make_empty_render_path());
            }
            MAKE_RENDER_PAINT => {
                let id = reader.read_var_uint();
                paints.insert(id, factory.make_render_paint());
            }
            REWIND | FILL_RULE | ADD_RAW_PATH => {
                let id = reader.read_var_uint();
                let Some(path) = paths.get_mut(&id) else {
                    return false;
                };
                match op {
                    REWIND => path.rewind(),
                    FILL_RULE => {
                        let rule = match reader.read_var_uint() as u8 {
                            0 => FillRule::NonZero,
                            1 => FillRule::EvenOdd,
                            2 => FillRule::Clockwise,
                            _ => return false,
                        };
                        path.fill_rule(rule);
                    }
                    _ => path.add_raw_path(&deserialize_raw_path(&mut reader)),
                }
            }
            COLOR | STYLE | THICKNESS | JOIN | CAP | FEATHER | BLEND_MODE | ADDITIVENESS => {
                let id = reader.read_var_uint();
                let Some(paint) = paints.get_mut(&id) else {
                    return false;
                };
                match op {
                    COLOR => paint.color(reader.read_var_uint() as u32),
                    STYLE => paint.style(if reader.read_var_uint() == 0 {
                        RenderPaintStyle::Stroke
                    } else {
                        RenderPaintStyle::Fill
                    }),
                    THICKNESS => paint.thickness(reader.read_float32()),
                    JOIN => paint.join(match reader.read_var_uint() as u32 {
                        0 => StrokeJoin::Miter,
                        1 => StrokeJoin::Round,
                        2 => StrokeJoin::Bevel,
                        _ => return false,
                    }),
                    CAP => paint.cap(match reader.read_var_uint() as u32 {
                        0 => StrokeCap::Butt,
                        1 => StrokeCap::Round,
                        2 => StrokeCap::Square,
                        _ => return false,
                    }),
                    FEATHER => paint.feather(reader.read_float32()),
                    ADDITIVENESS => paint.additiveness(reader.read_float32()),
                    _ => {
                        let Some(mode) = blend(reader.read_var_uint()) else {
                            return false;
                        };
                        paint.blend_mode(mode);
                    }
                }
            }
            STROKE_POSITION => {
                let id = reader.read_var_uint();
                let position = reader.read_var_uint();
                let Some(paint) = paints.get_mut(&id) else {
                    return false;
                };
                let position = match position {
                    0 => StrokePosition::Inside,
                    1 => StrokePosition::Center,
                    2 => StrokePosition::Outside,
                    _ => return false,
                };
                paint.stroke_position(position);
            }
            SHADER => {
                let id = reader.read_var_uint();
                let shader = reader.read_var_uint();
                let Some(paint) = paints.get_mut(&id) else {
                    return false;
                };
                // ID zero also spells nullptr: an existing shader zero wins.
                paint.shader(shaders.get(&shader).map(|shader| shader.as_ref()));
            }
            PAINT_MODULATED_IMAGE => {
                let id = reader.read_var_uint();
                let raw_image_id = reader.read_var_uint();
                let filter = match reader.read_var_uint() {
                    0 => ImageFilter::Bilinear,
                    1 => ImageFilter::Nearest,
                    _ => return false,
                };
                let mut read_wrap = || match reader.read_var_uint() {
                    0 => Some(ImageWrap::Clamp),
                    1 => Some(ImageWrap::Repeat),
                    2 => Some(ImageWrap::Mirror),
                    _ => None,
                };
                let Some(wrap_x) = read_wrap() else {
                    return false;
                };
                let Some(wrap_y) = read_wrap() else {
                    return false;
                };
                let matrix = Mat2D(std::array::from_fn(|_| reader.read_float32()));
                let Some(paint) = paints.get_mut(&id) else {
                    return false;
                };
                let image = raw_image_id
                    .checked_sub(1)
                    .and_then(|id| images.get(&id))
                    .and_then(|image| image.as_deref());
                paint.modulated_image(
                    image,
                    ImageSampler {
                        filter,
                        wrap_x,
                        wrap_y,
                    },
                    matrix,
                );
            }
            MAKE_LINEAR_GRADIENT | MAKE_RADIAL_GRADIENT => {
                let id = reader.read_var_uint();
                let count = reader.read_var_uint() as usize;
                let mut colors = Vec::with_capacity(count);
                let mut stops = Vec::with_capacity(count);
                for _ in 0..count {
                    colors.push(reader.read_var_uint() as u32);
                    stops.push(reader.read_float32());
                }
                let a = reader.read_float32();
                let b = reader.read_float32();
                let c = reader.read_float32();
                let shader = if op == MAKE_LINEAR_GRADIENT {
                    factory.make_linear_gradient(a, b, c, reader.read_float32(), &colors, &stops)
                } else {
                    factory.make_radial_gradient(a, b, c, &colors, &stops)
                };
                shaders.insert(id, shader);
            }
            DECODE_IMAGE => {
                let id = reader.read_var_uint();
                // BinaryDataReader's length-prefixed bytes preserve C++'s
                // empty span and sticky overflow when the payload truncates.
                let data = reader.read_string();
                images.insert(id, factory.decode_image(&data).ok().map(Rc::from));
            }
            MAKE_RENDER_BUFFER => {
                let id = reader.read_var_uint();
                let size = reader.read_var_uint() as usize;
                let kind = match reader.read_var_uint() as u8 {
                    0 => RenderBufferType::Index,
                    1 => RenderBufferType::Vertex,
                    _ => return false,
                };
                let flags = match reader.read_var_uint() as u8 {
                    0 => RenderBufferFlags::None,
                    1 => RenderBufferFlags::MappedOnceAtInitialization,
                    _ => return false,
                };
                buffers.insert(id, factory.make_render_buffer(kind, flags, size));
            }
            SET_VERTEX_BUFFER_DATA | SET_INDEX_BUFFER_DATA => {
                let id = reader.read_var_uint();
                let Some(buffer) = buffers.get_mut(&id) else {
                    return false;
                };
                let size = buffer.size_in_bytes();
                let mapped = buffer.map_mut();
                if op == SET_VERTEX_BUFFER_DATA {
                    for index in 0..size / 4 {
                        mapped[index * 4..index * 4 + 4]
                            .copy_from_slice(&reader.read_float32().to_ne_bytes());
                    }
                } else {
                    for index in 0..size / 2 {
                        mapped[index * 2..index * 2 + 2]
                            .copy_from_slice(&(reader.read_var_uint() as u16).to_ne_bytes());
                    }
                }
                buffer.unmap();
            }
            SAVE => active.renderer(renderer, &mut dropped_content).save(),
            RESTORE => active.renderer(renderer, &mut dropped_content).restore(),
            TRANSFORM => active
                .renderer(renderer, &mut dropped_content)
                .transform(Mat2D(std::array::from_fn(|_| reader.read_float32()))),
            MODULATE_OPACITY => active
                .renderer(renderer, &mut dropped_content)
                .modulate_opacity(reader.read_float32()),
            MODULATE_COLOR => {
                let color = reader.read_var_uint() as u32;
                let replace = reader.read_var_uint() != 0;
                active
                    .renderer(renderer, &mut dropped_content)
                    .modulate_color(color, replace);
            }
            DRAW_PATH => {
                let path = reader.read_var_uint();
                let paint = reader.read_var_uint();
                let (Some(path), Some(paint)) = (paths.get(&path), paints.get(&paint)) else {
                    return false;
                };
                active
                    .renderer(renderer, &mut dropped_content)
                    .draw_path(path.as_ref(), paint.as_ref());
            }
            CLIP_PATH => {
                let id = reader.read_var_uint();
                let Some(path) = paths.get(&id) else {
                    return false;
                };
                active
                    .renderer(renderer, &mut dropped_content)
                    .clip_path(path.as_ref());
            }
            DRAW_IMAGE | DRAW_IMAGE_MESH | DRAW_IMAGE_ADDITIVE | DRAW_IMAGE_MESH_ADDITIVE => {
                let id = reader.read_var_uint();
                let Some(mode) = blend(reader.read_var_uint()) else {
                    return false;
                };
                let opacity = reader.read_float32();
                let image = images.get(&id).and_then(|image| image.as_deref());
                if op == DRAW_IMAGE || op == DRAW_IMAGE_ADDITIVE {
                    let additiveness = if op == DRAW_IMAGE_ADDITIVE {
                        reader.read_float32()
                    } else {
                        0.0
                    };
                    if let Some(image) = image {
                        active
                            .renderer(renderer, &mut dropped_content)
                            .draw_image_with_additiveness(
                                Some(image),
                                ImageSampler::LINEAR_CLAMP,
                                mode,
                                opacity,
                                additiveness,
                            );
                    }
                } else {
                    let pos = reader.read_var_uint();
                    let uv = reader.read_var_uint();
                    let idx = reader.read_var_uint();
                    let additiveness = if op == DRAW_IMAGE_MESH_ADDITIVE {
                        reader.read_float32()
                    } else {
                        0.0
                    };
                    let pos = buffers.get(&pos).map(|value| value.as_ref());
                    let uv = buffers.get(&uv).map(|value| value.as_ref());
                    let idx = buffers.get(&idx).map(|value| value.as_ref());
                    if let (Some(image), Some(pos), Some(uv), Some(idx)) = (image, pos, uv, idx) {
                        let vertices = (pos.size_in_bytes() / 8) as u32;
                        let indices = (idx.size_in_bytes() / 2) as u32;
                        active
                            .renderer(renderer, &mut dropped_content)
                            .draw_image_mesh_with_additiveness(
                                Some(image),
                                ImageSampler::LINEAR_CLAMP,
                                Some(pos),
                                Some(uv),
                                Some(idx),
                                vertices,
                                indices,
                                mode,
                                opacity,
                                additiveness,
                            );
                    }
                }
            }
            MAKE_IMAGE_MESH_INSTANCES => {
                let id = reader.read_var_uint();
                let count = reader.read_var_uint() as usize;
                mesh_instances.insert(id, factory.make_image_mesh_instances(count));
            }
            SET_IMAGE_MESH_INSTANCES_DATA => {
                let id = reader.read_var_uint();
                let count = reader.read_var_uint() as usize;
                let mut instances = mesh_instances.get(&id).map(|value| value.borrow_mut());
                let dst = instances.as_mut().map(|value| value.edit(Some(count)));
                let mut dst = dst;
                for i in 0..count {
                    let instance = ImageMeshInstanceData {
                        transform: Mat2D(std::array::from_fn(|_| reader.read_float32())),
                        uv_translate: std::array::from_fn(|_| reader.read_float32()),
                        uv_scale: std::array::from_fn(|_| reader.read_float32()),
                        opacity: reader.read_float32(),
                        additiveness: reader.read_float32(),
                    };
                    if let Some(slot) = dst.as_mut().and_then(|dst| dst.get_mut(i)) {
                        *slot = instance;
                    }
                }
                if let Some(instances) = instances.as_mut() {
                    instances.end_edit();
                }
            }
            DRAW_IMAGE_MESH_INSTANCED => {
                let image = images
                    .get(&reader.read_var_uint())
                    .and_then(|value| value.as_deref());
                let pos = buffers.get(&reader.read_var_uint());
                let uv = buffers.get(&reader.read_var_uint());
                let idx = buffers.get(&reader.read_var_uint());
                let instances = mesh_instances.get(&reader.read_var_uint());
                if let (Some(image), Some(pos), Some(uv), Some(idx), Some(instances)) =
                    (image, pos, uv, idx, instances)
                {
                    active
                        .renderer(renderer, &mut dropped_content)
                        .draw_image_mesh_instanced(
                            Some(image),
                            ImageSampler::LINEAR_CLAMP,
                            Some(pos.as_ref()),
                            Some(uv.as_ref()),
                            Some(idx.as_ref()),
                            (pos.size_in_bytes() / 8) as u32,
                            (idx.size_in_bytes() / 2) as u32,
                            Some(instances),
                        );
                }
            }
            APPLY_LAYER_MASK => {
                let mask_id = reader.read_var_uint();
                let mode = reader.read_var_uint() as u8;
                if let Some(mask) = images.get(&mask_id).and_then(|image| image.as_deref()) {
                    active
                        .renderer(renderer, &mut dropped_content)
                        .apply_layer_mask(
                            Some(mask),
                            ImageSampler::LINEAR_CLAMP,
                            LayerMaskMode::from_value(mode as u32)
                                .expect("invalid serialized layer mask mode"),
                        );
                }
            }
            MAKE_RENDER_CANVAS => {
                let id = reader.read_var_uint();
                let width = reader.read_var_uint() as u32;
                let height = reader.read_var_uint() as u32;
                canvas_sizes.insert(id, (width, height));
            }
            CANVAS_CONTENT_BEGIN => {
                let id = reader.read_var_uint();
                let clear_color = reader.read_var_uint() as u32;
                let Some(&(width, height)) = canvas_sizes.get(&id) else {
                    return false;
                };
                interrupted.push((id, std::mem::replace(&mut active, ReplayTarget::Dropped)));
                let mut image = None;
                let content = hooks
                    .on_canvas_content_begin
                    .as_mut()
                    .and_then(|begin| begin(id, width, height, clear_color, &mut image));
                if let Some(content) = content {
                    images.insert(id, image);
                    active = ReplayTarget::Content(content);
                } else {
                    // A declined frame must erase any image from a prior frame,
                    // including an output image supplied by the declining host.
                    images.remove(&id);
                }
            }
            CANVAS_CONTENT_END => {
                let id = reader.read_var_uint();
                if interrupted.last().map(|(open, _)| *open) != Some(id) {
                    return false;
                }
                if let Some(end) = &mut hooks.on_canvas_content_end {
                    end(id);
                }
                active = interrupted.pop().expect("checked canvas bracket").1;
            }
            FRAME => {
                if let Some(callback) = &mut hooks.on_frame {
                    callback();
                }
            }
            FRAME_SIZE => {
                let width = reader.read_var_uint() as u32;
                let height = reader.read_var_uint() as u32;
                if let Some(callback) = &mut hooks.on_frame_size {
                    callback(width, height);
                }
            }
            _ => return false,
        }
        if reader.did_overflow() {
            return false;
        }
    }
    interrupted.is_empty()
}

fn blend(value: u64) -> Option<BlendMode> {
    Some(match value as u8 {
        3 => BlendMode::SrcOver,
        12 => BlendMode::Additive,
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
        _ => return None,
    })
}
