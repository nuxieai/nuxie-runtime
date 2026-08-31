//! `include/utils/serialized_replay.hpp` + `utils/serialized_replay.cpp`.
use crate::serialize_ops::*;
use crate::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct SerializedReplayHooks<'a> {
    pub on_frame: Option<Box<dyn FnMut() + 'a>>,
    pub on_frame_size: Option<Box<dyn FnMut(u32, u32) + 'a>>,
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
    let mut images: HashMap<u64, Option<Box<dyn RenderImage>>> = HashMap::new();
    let mut buffers: HashMap<u64, Box<dyn RenderBuffer>> = HashMap::new();
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
            COLOR | STYLE | THICKNESS | JOIN | CAP | FEATHER | BLEND_MODE => {
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
                    _ => {
                        let Some(mode) = blend(reader.read_var_uint()) else {
                            return false;
                        };
                        paint.blend_mode(mode);
                    }
                }
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
                images.insert(id, factory.decode_image(&data).ok());
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
            SAVE => renderer.save(),
            RESTORE => renderer.restore(),
            TRANSFORM => renderer.transform(Mat2D(std::array::from_fn(|_| reader.read_float32()))),
            MODULATE_OPACITY => renderer.modulate_opacity(reader.read_float32()),
            DRAW_PATH => {
                let path = reader.read_var_uint();
                let paint = reader.read_var_uint();
                let (Some(path), Some(paint)) = (paths.get(&path), paints.get(&paint)) else {
                    return false;
                };
                renderer.draw_path(path.as_ref(), paint.as_ref());
            }
            CLIP_PATH => {
                let id = reader.read_var_uint();
                let Some(path) = paths.get(&id) else {
                    return false;
                };
                renderer.clip_path(path.as_ref());
            }
            DRAW_IMAGE | DRAW_IMAGE_MESH => {
                let id = reader.read_var_uint();
                let Some(mode) = blend(reader.read_var_uint()) else {
                    return false;
                };
                let opacity = reader.read_float32();
                let image = images.get(&id).and_then(|image| image.as_deref());
                if op == DRAW_IMAGE {
                    renderer.draw_image(image, ImageSampler::LINEAR_CLAMP, mode, opacity);
                } else {
                    let pos = reader.read_var_uint();
                    let uv = reader.read_var_uint();
                    let idx = reader.read_var_uint();
                    let pos = buffers.get(&pos).map(|value| value.as_ref());
                    let uv = buffers.get(&uv).map(|value| value.as_ref());
                    let idx = buffers.get(&idx).map(|value| value.as_ref());
                    let vertices = pos.map_or(0, |value| (value.size_in_bytes() / 8) as u32);
                    let indices = idx.map_or(0, |value| (value.size_in_bytes() / 2) as u32);
                    renderer.draw_image_mesh(
                        image,
                        ImageSampler::LINEAR_CLAMP,
                        pos,
                        uv,
                        idx,
                        vertices,
                        indices,
                        mode,
                        opacity,
                    );
                }
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
    true
}

fn blend(value: u64) -> Option<BlendMode> {
    Some(match value as u8 {
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
        _ => return None,
    })
}
