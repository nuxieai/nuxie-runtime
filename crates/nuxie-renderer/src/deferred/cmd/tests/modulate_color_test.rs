//! All three modulate_color_test.cpp cases at upstream df0cc777.
use super::render_context_null::{FlushStats, NullBackend};
use super::*;
use crate::exact_source_adapter::ExactSourceBackend;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_paint_hpp::RiveRenderPaint;
use crate::mechanical_port::source::{
    include::rive::renderer_hpp::RendererContract,
    renderer::{
        include::rive::renderer::rive_renderer_hpp::RiveRenderer,
        src::{
            draw_cpp::{color_modulate, color_modulate_opacity},
            gradient_hpp::Gradient,
        },
    },
};

#[test]
fn modulate_color_save_restore() {
    let mut backend = NullBackend::new(
        100,
        100,
        Rc::new(RefCell::new(FlushStats::default())),
        Rc::new(Cell::new(0)),
    );
    backend
        .begin_frame(0, crate::RenderMode::RasterOrdering)
        .unwrap();
    let context = unsafe { backend.context_mut().get_unchecked_mut() };
    let mut rect = RawPath::default();
    rect.add_rect(Aabb::new(0.0, 0.0, 100.0, 100.0));
    let path = context.makeRenderPath(&mut rect, FillRule::NonZero);
    let paint = context.makeRenderPaint();
    // The null context's source factory allocates RiveRenderPaint owners.
    unsafe {
        (&mut *paint.get().cast::<RiveRenderPaint>()).color(0xffff_ffff);
    }
    let mut renderer = unsafe { RiveRenderer::new_from_context(context) };
    assert_eq!(renderer.currentModulatedColor(), 0xffff_ffff);
    renderer.save();
    renderer.modulateColor(0xff00_0000, false);
    renderer.modulateColor(0xff80_8080, false);
    assert_eq!(renderer.currentModulatedColor(), 0xff00_0000);
    unsafe {
        renderer.drawPath(path.get(), paint.get());
    }
    renderer.save();
    renderer.modulateColor(0xff80_4020, true);
    assert_eq!(renderer.currentModulatedColor(), 0xff80_4020);
    renderer.modulateColor(0x80ff_8000, false);
    assert_eq!(renderer.currentModulatedColor(), 0x8080_2000);
    unsafe {
        renderer.drawPath(path.get(), paint.get());
    }
    renderer.restore();
    assert_eq!(renderer.currentModulatedColor(), 0xff00_0000);
    renderer.restore();
    assert_eq!(renderer.currentModulatedColor(), 0xffff_ffff);
    backend.flush();
}

#[test]
fn color_modulate_is_exact_at_identity() {
    for color in [0x0000_0000, 0x8012_3456, 0xfffe_fdfc] {
        for opacity in [1.0, 0.5, 0.3] {
            assert_eq!(
                color_modulate(color, 0xffff_ffff, opacity),
                color_modulate_opacity(color, opacity)
            );
        }
    }
    assert_eq!(color_modulate(0xff80_40ff, 0xff00_0000, 1.0), 0xff00_0000);
}

#[test]
fn gradient_modulated_by_color() {
    let colors = [0xffff_0000, 0xff00_ff00];
    let stops = [0.0, 1.0];
    let gradient =
        unsafe { Gradient::MakeLinear(0.0, 0.0, 100.0, 0.0, colors.as_ptr(), stops.as_ptr(), 2) };
    assert!(!gradient.get().is_null());
    let owner = unsafe { &*gradient.get() };
    let black = owner.getModulated(1.0, 0xff00_0000);
    assert_ne!(black.get(), gradient.get());
    assert_eq!(
        unsafe { &*black.get() }.colors_slice(),
        &[0xff00_0000, 0xff00_0000]
    );
    assert_eq!(owner.getModulated(1.0, 0xff00_0000).get(), black.get());
    assert_ne!(owner.getModulated(1.0, 0xff80_8080).get(), black.get());
}

// The merged record is constructed from a real PathDraw, matching gpu_namespace_test.cpp.
fn with_path_data(
    paint: &RiveRenderPaint,
    check: impl FnOnce(
        &crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PathData,
    ),
) {
    use crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::PathData;
    use crate::mechanical_port::source::renderer::src::draw_cpp::make_path_draw_from_source;
    let mut backend = NullBackend::new(
        64,
        64,
        Rc::new(RefCell::new(FlushStats::default())),
        Rc::new(Cell::new(0)),
    );
    backend
        .begin_frame(0, crate::RenderMode::RasterOrdering)
        .unwrap();
    let context = unsafe { backend.context_mut().get_unchecked_mut() };
    let mut raw = RawPath::default();
    raw.add_rect(Aabb::new(8.0, 8.0, 56.0, 56.0));
    let path = context.makeRenderPath(&mut raw, FillRule::NonZero);
    let mut owner = unsafe {
        make_path_draw_from_source(
            context,
            Mat2D::IDENTITY,
            None,
            static_rcp_cast(path),
            FillRule::NonZero,
            paint,
            1.0,
            0xffffffff,
            None,
        )
    }
    .unwrap();
    owner.draw.setClipID(2);
    let data = PathData::new(
        &owner.draw,
        0,
        context.frameInterlockMode(),
        context.platformFeatures(),
    );
    check(&data);
    unsafe {
        owner.draw.releaseRefs();
    }
    drop(owner);
    backend.flush();
}

#[test]
fn layer_mask_mode_stays_out_of_the_clip_id() {
    use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
    use crate::mechanical_port::source::renderer::src::gpu_cpp::packed_layer_mask_mode;
    use crate::mechanical_port::source::renderer::src::shaders::constants_glsl::{
        LAYER_MASK_MODE_INVERTED_LUMINANCE, PAINT_LAYER_MASK_MODE_MASK, PAINT_LAYER_MASK_MODE_SHIFT,
    };
    let mut paint = RiveRenderPaint::new();
    paint.layerMask(
        rcp::new(),
        nuxie_render_api::LayerMaskMode::InvertedLuminance,
    );
    with_path_data(&paint, |data| {
        let valid = data.m_params;
        assert_eq!(valid >> 16, 2);
        assert_eq!(
            (valid & PAINT_LAYER_MASK_MODE_MASK) >> PAINT_LAYER_MASK_MODE_SHIFT,
            LAYER_MASK_MODE_INVERTED_LUMINANCE as u32
        );
        // A Rust enum cannot carry C++ static_cast<LayerMaskMode>(8); exercise
        // the same production packing operation without creating an invalid enum.
        let out_of_range = (valid & !PAINT_LAYER_MASK_MODE_MASK) | packed_layer_mask_mode(8);
        assert_eq!(out_of_range >> 16, 2);
    });
}

#[test]
fn gradient_packs_unquantized_additiveness_complement() {
    use crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast;
    for (blend, additive, expected) in [
        (BlendMode::SrcOver, 0.0, 1.0),
        (BlendMode::SrcOver, 0.5, 0.5),
        (BlendMode::SrcOver, 1.0, 0.0),
        (BlendMode::Multiply, 1.0, 0.0),
        (BlendMode::SrcOver, 0.12345, 1.0 - 0.12345),
    ] {
        let mut paint = RiveRenderPaint::new();
        let colors = [0xffff0000, 0xff00ff00];
        let stops = [0.0, 1.0];
        let gradient = unsafe {
            Gradient::MakeLinear(0.0, 0.0, 100.0, 0.0, colors.as_ptr(), stops.as_ptr(), 2)
        };
        unsafe {
            paint.shader_source(static_rcp_cast(gradient));
        }
        paint.blendMode(blend);
        paint.additiveness(additive);
        with_path_data(&paint, |data| {
            assert_eq!(
                unsafe { data.value.m_gradientAdditivenessComplement },
                expected
            )
        });
    }
}
