//! Complete `tests/gm/additive_blend.cpp` scene at upstream b86b7ecb.
//! C++ captures apply the existing Metal FMA/coverage adaptations; pristine
//! captures and exact adaptation identities are retained beside each reference.
use crate::{
    native_metal::{NativeMetalContextOptions, NativeMetalFactory, ShaderCompilationMode},
    RenderMode,
};
pub(super) use nuxie_render_api::*;

pub(super) const CELL: f32 = 300.;
pub(super) const NOMOON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/renderer/image_paint/nomoon.png"
));

pub(super) fn rect(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    bounds: Aabb,
    paint: &dyn RenderPaint,
) {
    let mut raw = RawPath::new();
    raw.add_rect(bounds);
    r.draw_path(f.make_render_path(raw, FillRule::NonZero).as_ref(), paint);
}

pub(super) fn circle(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    cx: f32,
    cy: f32,
    radius: f32,
    paint: &dyn RenderPaint,
) {
    // gmutils.cpp::PathBuilder::Circle: scale first, then translate each
    // control point, retaining its exact cubic constant and starting point.
    const C: f32 = 0.5519150244935105707435627;
    let points = [
        (1., 0.),
        (1., C),
        (C, 1.),
        (0., 1.),
        (-C, 1.),
        (-1., C),
        (-1., 0.),
        (-1., -C),
        (-C, -1.),
        (0., -1.),
        (C, -1.),
        (1., -C),
        (1., 0.),
    ];
    let mut raw = RawPath::new();
    raw.move_to(cx + radius, cy);
    for i in (1..=12).step_by(3) {
        let a = points[i];
        let b = points[i + 1];
        let c = points[i + 2];
        raw.cubic_to(
            a.0 * radius + cx,
            a.1 * radius + cy,
            b.0 * radius + cx,
            b.1 * radius + cy,
            c.0 * radius + cx,
            c.1 * radius + cy,
        );
    }
    raw.close();
    r.draw_path(f.make_render_path(raw, FillRule::NonZero).as_ref(), paint);
}

fn buffer(f: &mut dyn Factory, kind: RenderBufferType, bytes: &[u8]) -> Box<dyn RenderBuffer> {
    let mut b = f.make_render_buffer(
        kind,
        RenderBufferFlags::MappedOnceAtInitialization,
        bytes.len(),
    );
    b.map_mut().copy_from_slice(bytes);
    b.unmap();
    b
}

fn draw_image_mesh_quad(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    image: &dyn RenderImage,
    dst: Aabb,
    uv: Aabb,
    additiveness: f32,
    opacity: f32,
) {
    let floats = |rect: Aabb| {
        [
            rect.min_x, rect.min_y, rect.max_x, rect.min_y, rect.min_x, rect.max_y, rect.max_x,
            rect.max_y,
        ]
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect::<Vec<_>>()
    };
    let pts = buffer(f, RenderBufferType::Vertex, &floats(dst));
    let uvs = buffer(f, RenderBufferType::Vertex, &floats(uv));
    let indices = buffer(
        f,
        RenderBufferType::Index,
        &[0u16, 1, 2, 1, 3, 2]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect::<Vec<_>>(),
    );
    r.draw_image_mesh_with_additiveness(
        Some(image),
        ImageSampler::default(),
        Some(pts.as_ref()),
        Some(uvs.as_ref()),
        Some(indices.as_ref()),
        4,
        6,
        BlendMode::SrcOver,
        opacity,
        additiveness,
    );
}

pub(super) fn draw_image_rect(
    r: &mut dyn Renderer,
    image: &dyn RenderImage,
    rect: Aabb,
    additiveness: f32,
    opacity: f32,
) {
    r.save();
    r.translate(rect.min_x, rect.min_y);
    r.scale(
        rect.width() / image.width() as f32,
        rect.height() / image.height() as f32,
    );
    r.draw_image_with_additiveness(
        Some(image),
        ImageSampler::default(),
        BlendMode::SrcOver,
        opacity,
        additiveness,
    );
    r.restore();
}

fn base_rect(ox: f32, oy: f32) -> Aabb {
    Aabb::new(ox + 40., oy + 40., ox + 190., oy + 190.)
}
fn overlay_rect(ox: f32, oy: f32) -> Aabb {
    Aabb::new(ox + 110., oy + 110., ox + 260., oy + 260.)
}

pub(super) fn gradient(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    bounds: Aabb,
    colors: &[u32; 2],
    vertical: bool,
    additive: f32,
    blend: BlendMode,
    style: RenderPaintStyle,
    thickness: f32,
) {
    let mut paint = f.make_render_paint();
    paint.style(style);
    if style == RenderPaintStyle::Stroke {
        paint.thickness(thickness);
    }
    let shader = if vertical {
        f.make_linear_gradient(0., bounds.min_y, 0., bounds.max_y, colors, &[0., 1.])
    } else {
        f.make_linear_gradient(bounds.min_x, 0., bounds.max_x, 0., colors, &[0., 1.])
    };
    paint.shader(Some(shader.as_ref()));
    paint.blend_mode(blend);
    paint.additiveness(additive);
    rect(f, r, bounds, paint.as_ref());
}

pub(super) fn solid(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    bounds: Aabb,
    color: u32,
    blend: BlendMode,
    additive: f32,
) {
    let mut paint = f.make_render_paint();
    paint.color(color);
    paint.blend_mode(blend);
    paint.additiveness(additive);
    rect(f, r, bounds, paint.as_ref());
}

fn scene(f: &mut dyn Factory, r: &mut dyn Renderer) {
    for (oy, style, thickness) in [
        (0., RenderPaintStyle::Fill, 0.),
        (CELL, RenderPaintStyle::Stroke, 20.),
    ] {
        let mut paint = f.make_render_paint();
        paint.style(style);
        if style == RenderPaintStyle::Stroke {
            paint.thickness(thickness);
        }
        paint.color(0xff00ff00);
        rect(f, r, base_rect(0., oy), paint.as_ref());
        gradient(
            f,
            r,
            overlay_rect(0., oy),
            &[0xffff0000, 0x00ff0000],
            true,
            0.,
            BlendMode::SrcOver,
            style,
            thickness,
        );
    }
    for (oy, style, thickness) in [
        (0., RenderPaintStyle::Fill, 0.),
        (CELL, RenderPaintStyle::Stroke, 40.),
    ] {
        for (cx, cy, color) in [
            (CELL + 150., oy + 110., 0xffff0000),
            (CELL + 110., oy + 185., 0xff00ff00),
            (CELL + 190., oy + 185., 0xff0000ff),
        ] {
            let mut paint = f.make_render_paint();
            paint.style(style);
            if style == RenderPaintStyle::Stroke {
                paint.thickness(thickness);
            }
            paint.color(color);
            paint.additiveness(1.);
            circle(f, r, cx, cy, 80., paint.as_ref());
        }
    }
    for (ox, additive) in [(0., 0.), (CELL, 1.)] {
        gradient(
            f,
            r,
            base_rect(ox, CELL * 2.),
            &[0xff00ff00, 0xff0000ff],
            false,
            0.,
            BlendMode::SrcOver,
            RenderPaintStyle::Fill,
            0.,
        );
        gradient(
            f,
            r,
            overlay_rect(ox, CELL * 2.),
            &[0xffff0000, 0x00ffff00],
            true,
            additive,
            BlendMode::SrcOver,
            RenderPaintStyle::Fill,
            0.,
        );
    }
    if let Ok(image) = f.decode_image(NOMOON) {
        for (ox, additive) in [(0., 0.), (CELL, 1.)] {
            draw_image_mesh_quad(
                f,
                r,
                image.as_ref(),
                base_rect(ox, CELL * 3.),
                Aabb::new(0., 0., 1., 1.),
                0.,
                1.,
            );
            let dst = overlay_rect(ox, CELL * 3.);
            for i in 0..256 {
                let t0 = i as f32 / 256.;
                let t1 = (i + 1) as f32 / 256.;
                draw_image_mesh_quad(
                    f,
                    r,
                    image.as_ref(),
                    Aabb::new(
                        dst.min_x,
                        dst.min_y + t0 * dst.height(),
                        dst.max_x,
                        dst.min_y + t1 * dst.height(),
                    ),
                    Aabb::new(0., t0, 1., t1),
                    additive,
                    1. - (t0 + t1) * 0.5,
                );
            }
        }
        for (ox, additive) in [(0., 0.), (CELL, 1.)] {
            draw_image_rect(r, image.as_ref(), base_rect(ox, CELL * 4.), 0., 1.);
            draw_image_rect(
                r,
                image.as_ref(),
                overlay_rect(ox, CELL * 4.),
                additive,
                0.6,
            );
        }
    }
    for (ox, is_gradient) in [(0., false), (CELL, true)] {
        let oy = CELL * 5.;
        solid(f, r, base_rect(ox, oy), 0xff00ff00, BlendMode::SrcOver, 0.);
        let overlay = overlay_rect(ox, oy);
        for i in 0..5 {
            let strip = Aabb::new(
                overlay.min_x + overlay.width() * i as f32 / 5.,
                overlay.min_y,
                overlay.min_x + overlay.width() * (i + 1) as f32 / 5.,
                overlay.max_y,
            );
            if is_gradient {
                gradient(
                    f,
                    r,
                    strip,
                    &[0xffff0000, 0x80ff0000],
                    true,
                    i as f32 / 4.,
                    BlendMode::SrcOver,
                    RenderPaintStyle::Fill,
                    0.,
                );
            } else {
                solid(f, r, strip, 0xc0ff0000, BlendMode::SrcOver, i as f32 / 4.);
            }
        }
    }
}

pub(super) fn compare_scene(
    name: &str,
    height: u32,
    clear: u32,
    scene: fn(&mut dyn Factory, &mut dyn Renderer),
) {
    let mut f = NativeMetalFactory::new_with_mode_and_context_options(
        600,
        height,
        RenderMode::RasterOrdering,
        NativeMetalContextOptions {
            shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
            ..Default::default()
        },
    )
    .expect("native Metal GM factory");
    let mut r = f.begin_frame(clear).expect("GM frame");
    r.save();
    scene(&mut f, &mut r);
    r.restore();
    let pixels = r.finish().expect("GM readback");
    let actual = pixel_compare::RgbaImage::new(600, height, pixels).expect("GM dimensions");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/renderer/reference/metal/gm/{name}.png"
    ));
    let expected =
        pixel_compare::RgbaImage::read_png(&path).expect("pinned C++ additive GM capture");
    let report = pixel_compare::compare(&expected, &actual, pixel_compare::Tolerance::EXACT)
        .expect("matching dimensions");
    if !report.within_tolerance {
        let diagnostic = std::env::temp_dir().join(format!("nuxie-{name}-actual.png"));
        actual.write_png(&diagnostic).expect("GM diagnostic PNG");
        eprintln!("actual GM pixels: {}", diagnostic.display());
    }
    assert!(
        report.within_tolerance,
        "{name}: {} differing pixels, max channel delta {}",
        report.different_pixels, report.max_channel_delta
    );
}

#[test]
fn additive_blend() {
    compare_scene("additive_blend", 1800, 0xff000000, scene);
}
