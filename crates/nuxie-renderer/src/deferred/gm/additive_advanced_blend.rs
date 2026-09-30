//! Complete `tests/gm/additive_advanced_blend.cpp` scene at upstream b86b7ecb.
use super::additive_blend::*;

fn colored_circle(
    f: &mut dyn Factory,
    r: &mut dyn Renderer,
    cx: f32,
    cy: f32,
    color: u32,
    additive: f32,
) {
    let mut paint = f.make_render_paint();
    paint.color(color);
    paint.additiveness(additive);
    circle(f, r, cx, cy, 80., paint.as_ref());
}

fn scene(f: &mut dyn Factory, r: &mut dyn Renderer) {
    for (ox, additive) in [(0., 0.), (CELL, 1.)] {
        colored_circle(f, r, ox + 150., 110., 0xffff0000, additive);
        solid(
            f,
            r,
            Aabb::new(ox + 40., 40., ox + 190., 190.),
            0xff8080ff,
            BlendMode::Multiply,
            0.,
        );
        colored_circle(f, r, ox + 110., 185., 0xff00ff00, additive);
        solid(
            f,
            r,
            Aabb::new(ox + 110., 110., ox + 260., 260.),
            0xff404080,
            BlendMode::Screen,
            0.,
        );
        colored_circle(f, r, ox + 190., 185., 0xff0000ff, additive);
    }
    for (ox, additive) in [(0., 0.), (CELL, 1.)] {
        let oy = CELL;
        gradient(
            f,
            r,
            Aabb::new(ox + 40., oy + 40., ox + 260., oy + 260.),
            &[0xff00c0ff, 0xffff8000],
            false,
            0.,
            BlendMode::SrcOver,
            RenderPaintStyle::Fill,
            0.,
        );
        gradient(
            f,
            r,
            Aabb::new(ox + 70., oy + 70., ox + 170., oy + 230.),
            &[0xffff0000, 0x00ff0000],
            true,
            additive,
            BlendMode::SrcOver,
            RenderPaintStyle::Fill,
            0.,
        );
        gradient(
            f,
            r,
            Aabb::new(ox + 130., oy + 100., ox + 230., oy + 200.),
            &[0xffffffff, 0xff000000],
            false,
            0.,
            BlendMode::Difference,
            RenderPaintStyle::Fill,
            0.,
        );
        solid(
            f,
            r,
            Aabb::new(ox + 100., oy + 160., ox + 260., oy + 240.),
            0x8000ff00,
            BlendMode::SrcOver,
            additive,
        );
    }
    if let Ok(image) = f.decode_image(NOMOON) {
        for (ox, additive) in [(0., 0.), (CELL, 1.)] {
            let oy = CELL * 2.;
            draw_image_rect(
                r,
                image.as_ref(),
                Aabb::new(ox + 40., oy + 40., ox + 190., oy + 190.),
                0.,
                1.,
            );
            solid(
                f,
                r,
                Aabb::new(ox + 90., oy + 60., ox + 210., oy + 180.),
                0xffa0a0a0,
                BlendMode::Multiply,
                0.,
            );
            draw_image_rect(
                r,
                image.as_ref(),
                Aabb::new(ox + 110., oy + 110., ox + 260., oy + 260.),
                additive,
                0.7,
            );
            solid(
                f,
                r,
                Aabb::new(ox + 60., oy + 170., ox + 180., oy + 270.),
                0xff808040,
                BlendMode::HardLight,
                0.,
            );
        }
    }
    let oy = CELL * 3.;
    solid(
        f,
        r,
        Aabb::new(40., oy + 40., 140., oy + 260.),
        0xff80c0ff,
        BlendMode::Multiply,
        0.,
    );
    solid(
        f,
        r,
        Aabb::new(160., oy + 40., 260., oy + 260.),
        0xff80c0ff,
        BlendMode::Multiply,
        1.,
    );
    solid(
        f,
        r,
        Aabb::new(CELL + 40., oy + 40., CELL + 190., oy + 190.),
        0xff00ff00,
        BlendMode::SrcOver,
        0.,
    );
    let overlay = Aabb::new(CELL + 110., oy + 110., CELL + 260., oy + 260.);
    for i in 0..5 {
        if i == 3 {
            solid(
                f,
                r,
                Aabb::new(overlay.min_x, oy + 180., overlay.max_x, oy + 220.),
                0xff8080ff,
                BlendMode::Multiply,
                0.,
            );
        }
        let strip = Aabb::new(
            overlay.min_x + overlay.width() * i as f32 / 5.,
            overlay.min_y,
            overlay.min_x + overlay.width() * (i + 1) as f32 / 5.,
            overlay.max_y,
        );
        solid(f, r, strip, 0xc0ff0000, BlendMode::SrcOver, i as f32 / 4.);
    }
}

#[test]
fn additive_advanced_blend() {
    compare_scene("additive_advanced_blend", 1200, 0xff404040, scene);
}
