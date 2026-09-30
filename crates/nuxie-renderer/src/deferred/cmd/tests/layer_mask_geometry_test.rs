//! Remaining runtime/layer_mask_test.cpp cases (lines 505–1494), upstream 8398db31.
use super::layer_mask_test::{Fixture, boolean, bounds, build_mask_fixture, number, uint};
use nuxie_render_api::Mat2D;
use nuxie_runtime::source::{
    artboard::Artboard,
    core::CoreHandle,
    generated::{
        core_registry::CoreRegistry, layer_mask_base::LayerMaskBase,
        layout_component_base::LayoutComponentBase, node_base::NodeBase,
        shapes::paint::solid_color_base::SolidColorBase,
        transform_component_base::TransformComponentBase,
    },
    layer_mask::LayerMask,
    math::aabb::Aabb,
    offscreen_raster::{K_MAX_DIM, K_SHRINK_DRAWS},
    shapes::{paint::solid_color::SolidColor, shape::Shape},
};

fn mask<T>(h: &CoreHandle, f: impl FnOnce(&LayerMask) -> T) -> T {
    h.with_downcast::<LayerMask, _>(f).unwrap()
}
fn position(h: &CoreHandle, x: f32, y: f32) {
    number(h, NodeBase::X_PROPERTY_KEY, x);
    number(h, NodeBase::Y_PROPERTY_KEY, y);
}
fn scale(h: &CoreHandle, value: f32) {
    number(h, TransformComponentBase::SCALE_X_PROPERTY_KEY, value);
    number(h, TransformComponentBase::SCALE_Y_PROPERTY_KEY, value);
}
fn covers(outer: Aabb, inner: Aabb) {
    assert!(outer.left() <= inner.left());
    assert!(outer.top() <= inner.top());
    assert!(outer.right() >= inner.right());
    assert!(outer.bottom() >= inner.bottom());
}
fn custom(h: &CoreHandle, x: f32, y: f32, w: f32, height: f32) {
    boolean(h, LayerMaskBase::USE_CUSTOM_BOUNDS_PROPERTY_KEY, true);
    for (key, value) in [
        (LayerMaskBase::BOUNDS_X_PROPERTY_KEY, x),
        (LayerMaskBase::BOUNDS_Y_PROPERTY_KEY, y),
        (LayerMaskBase::BOUNDS_WIDTH_PROPERTY_KEY, w),
        (LayerMaskBase::BOUNDS_HEIGHT_PROPERTY_KEY, height),
    ] {
        number(h, key, value);
    }
}
fn color(shape: &CoreHandle) -> CoreHandle {
    let paint = shape
        .with_downcast::<Shape, _>(|s| {
            assert!(!s.paint_container.shape_paints().is_empty());
            s.paint_container.shape_paints()[0].clone()
        })
        .unwrap();
    let mutator = paint
        .with(|p| p.as_shape_paint().unwrap().paint())
        .unwrap()
        .unwrap();
    assert!(mutator.with_downcast::<SolidColor, _>(|_| ()).is_some());
    mutator
}
fn recolor(h: &CoreHandle, value: u32) {
    assert!(CoreRegistry::set_color_handle(
        h,
        SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
        value as i32
    ));
}

#[test]
fn a_small_mask_in_a_large_artboard_rasters_small_and_sharp() {
    let f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(2000.0, 2000.0);
    position(&m.masked, 100.0, 100.0);
    position(&m.source_group, 100.0, 100.0);
    let mut renderer = f.session.as_ref().unwrap().borrow().make_screen_renderer(0);
    renderer.save();
    renderer.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
    f.advance();
    Artboard::draw_internal_handle(&f.artboard, renderer.as_mut());
    renderer.restore();
    assert!(
        (mask(&m.mask, LayerMask::test_raster_scale) - 2.0).abs() <= 2.0 * f32::EPSILON * 100.0
    );
    assert_eq!(mask(&m.mask, LayerMask::test_width_px), 64);
    assert_eq!(mask(&m.mask, LayerMask::test_height_px), 64);
    let b = bounds(&m.content);
    assert!((b.width() - 10.0).abs() <= 10.0 * f32::EPSILON * 100.0);
    covers(mask(&m.mask, LayerMask::test_box), b);
}
#[test]
fn an_authored_rectangle_replaces_the_measured_box() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    custom(&m.mask, 60.0, 70.0, 40.0, 40.0);
    f.frame();
    let b = mask(&m.mask, LayerMask::test_box);
    assert!(b.left() <= 60.0);
    assert!(b.top() <= 70.0);
    assert!(b.right() >= 100.0);
    assert!(b.bottom() >= 110.0);
    assert!(b.left() > 50.0);
}
#[test]
fn a_degenerate_rectangle_falls_back_to_measuring() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    custom(&m.mask, 10.0, 10.0, 0.0, 0.0);
    f.frame();
    assert!(mask(&m.mask, LayerMask::test_width_px) > 0);
    assert!(!mask(&m.mask, LayerMask::test_box).is_empty_or_nan());
}
#[test]
fn a_clipping_artboard_clamps_the_mask_box_to_itself() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    f.frame();
    let b = bounds(&m.content);
    assert!(b.left() < 0.0);
    let raster = mask(&m.mask, LayerMask::test_box);
    assert!(raster.left() > b.left());
    assert!(raster.left() <= 0.0);
}
#[test]
fn an_unclipped_artboard_does_not_clamp_the_mask_box() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    boolean(&f.artboard, LayoutComponentBase::CLIP_PROPERTY_KEY, false);
    f.frame();
    let b = bounds(&m.content);
    assert!(b.left() < 0.0);
    let raster = mask(&m.mask, LayerMask::test_box);
    assert!(raster.left() <= b.left());
    assert!(raster.top() <= b.top());
}
#[test]
fn disjoint_content_and_source_skip_the_gpu_entirely() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    assert_eq!(f.frame(), 2);
    number(&m.source_group, NodeBase::X_PROPERTY_KEY, 80.0);
    assert_eq!(f.frame(), 0);
    number(&m.source_group, NodeBase::X_PROPERTY_KEY, 0.0);
    assert_eq!(f.frame(), 2);
    uint(&m.mask, LayerMaskBase::MASK_MODE_VALUE_PROPERTY_KEY, 1);
    number(&m.source_group, NodeBase::X_PROPERTY_KEY, 80.0);
    assert_eq!(f.frame(), 0);
}
#[test]
fn a_translating_mask_rerasters_but_never_reallocates() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(400.0, 400.0);
    position(&m.masked, 100.0, 100.0);
    position(&m.source_group, 100.0, 100.0);
    assert_eq!(f.frame(), 2);
    let allocations = mask(&m.mask, |m| m.test_allocations);
    assert_eq!(allocations, 1);
    let width = mask(&m.mask, LayerMask::test_width_px);
    let height = mask(&m.mask, LayerMask::test_height_px);
    let left = mask(&m.mask, LayerMask::test_box).left();
    for step in 1..=40 {
        let x = 100.0 + step as f32 * 0.37;
        number(&m.masked, NodeBase::X_PROPERTY_KEY, x);
        number(&m.source_group, NodeBase::X_PROPERTY_KEY, x);
        assert_eq!(f.frame(), 2);
    }
    assert_eq!(mask(&m.mask, |m| m.test_allocations), allocations);
    assert_eq!(mask(&m.mask, LayerMask::test_width_px), width);
    assert_eq!(mask(&m.mask, LayerMask::test_height_px), height);
    assert!(mask(&m.mask, LayerMask::test_box).left() > left);
}
#[test]
fn a_shrunk_mask_gives_its_texture_back_after_k_shrink_draws() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    let scaler = f.node(&f.artboard);
    scale(&scaler, 20.0);
    f.reparent(&m.masked, &scaler);
    f.reparent(&m.source_group, &scaler);
    f.initialize();
    f.size(1000.0, 1000.0);
    position(&scaler, 300.0, 300.0);
    f.frame();
    let big = mask(&m.mask, LayerMask::test_width_px);
    assert!(big > 64);
    let allocations = mask(&m.mask, |m| m.test_allocations);
    scale(&scaler, 2.0);
    f.frame();
    assert_eq!(mask(&m.mask, LayerMask::test_width_px), big);
    assert_eq!(mask(&m.mask, |m| m.test_allocations), allocations);
    let mut draws = 1;
    while mask(&m.mask, LayerMask::test_width_px) == big && draws < u32::from(K_SHRINK_DRAWS) * 4 {
        f.frame();
        draws += 1;
    }
    assert_eq!(draws, u32::from(K_SHRINK_DRAWS));
    assert!(mask(&m.mask, LayerMask::test_width_px) < big);
    assert_eq!(mask(&m.mask, |m| m.test_allocations), allocations + 1);
}
#[test]
fn a_mask_larger_than_k_max_dim_coarsens_rather_than_crops() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    let scaler = f.node(&f.artboard);
    scale(&scaler, 300.0);
    f.reparent(&m.masked, &scaler);
    f.reparent(&m.source_group, &scaler);
    f.initialize();
    f.size(4000.0, 4000.0);
    position(&scaler, 2000.0, 2000.0);
    f.frame();
    let b = bounds(&m.content);
    assert!(b.width() > 2048.0);
    assert!(mask(&m.mask, LayerMask::test_width_px) <= K_MAX_DIM);
    assert!(mask(&m.mask, LayerMask::test_height_px) <= K_MAX_DIM);
    assert!(mask(&m.mask, LayerMask::test_raster_scale) < 1.0);
    covers(mask(&m.mask, LayerMask::test_box), b);
}
#[test]
fn an_unrelated_change_leaves_a_masks_rasters_alone() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    let stranger = f.node(&f.artboard);
    f.shape(&stranger, "Stranger");
    f.initialize();
    f.size(200.0, 200.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
    number(&stranger, NodeBase::X_PROPERTY_KEY, 40.0);
    assert_eq!(f.frame(), 0);
    number(&stranger, NodeBase::X_PROPERTY_KEY, 80.0);
    assert_eq!(f.frame(), 0);
    number(&m.masked, NodeBase::X_PROPERTY_KEY, 3.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
}
#[test]
fn recolouring_masked_content_rebuilds_the_raster() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
    let c = color(&m.content);
    recolor(&c, 0xff00ff00);
    assert_eq!(f.frame(), 2);
    recolor(&c, 0xff0000ff);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
}
#[test]
fn moving_one_stacked_masks_source_rebuilds_both() {
    let mut f = Fixture::new(true);
    let masked = f.node(&f.artboard);
    f.shape(&masked, "Content");
    let a = f.node(&f.artboard);
    f.shape(&a, "SrcA");
    let b = f.node(&f.artboard);
    let moving = f.shape(&b, "SrcB");
    f.mask(&masked, &a);
    f.mask(&masked, &b);
    f.initialize();
    f.size(200.0, 200.0);
    let full = f.frame();
    assert!(full > 0);
    assert_eq!(f.frame(), 0);
    number(&moving, NodeBase::X_PROPERTY_KEY, 3.0);
    assert_eq!(f.frame(), full);
    assert_eq!(f.frame(), 0);
}
#[test]
fn moving_a_nested_masks_source_rebuilds_the_enclosing_mask() {
    let mut f = Fixture::new(true);
    let outer = f.node(&f.artboard);
    let inner = f.node(&outer);
    f.shape(&inner, "Content");
    let a = f.node(&f.artboard);
    f.shape(&a, "OuterSrc");
    let b = f.node(&f.artboard);
    let moving = f.shape(&b, "InnerSrc");
    f.mask(&outer, &a);
    f.mask(&inner, &b);
    f.initialize();
    f.size(200.0, 200.0);
    let full = f.frame();
    assert!(full > 0);
    assert_eq!(f.frame(), 0);
    number(&moving, NodeBase::X_PROPERTY_KEY, 3.0);
    assert_eq!(f.frame(), full);
    assert_eq!(f.frame(), 0);
}
#[test]
fn a_source_above_the_masked_node_is_rejected() {
    let mut f = Fixture::new(true);
    let outer = f.node(&f.artboard);
    let inner = f.node(&outer);
    f.shape(&inner, "Content");
    let m = f.mask(&inner, &outer);
    f.initialize();
    f.size(200.0, 200.0);
    assert!(mask(&m, LayerMask::is_self_referential));
    assert_eq!(f.frame(), 0);
}
#[test]
fn two_masks_pointing_at_each_other_terminate() {
    let mut f = Fixture::new(true);
    let a = f.node(&f.artboard);
    f.shape(&a, "A");
    let b = f.node(&f.artboard);
    f.shape(&b, "B");
    let ma = f.mask(&a, &b);
    let mb = f.mask(&b, &a);
    f.initialize();
    f.size(200.0, 200.0);
    assert!(!mask(&ma, LayerMask::is_self_referential));
    assert!(!mask(&mb, LayerMask::is_self_referential));
    assert!(f.frame() <= 4);
}
#[test]
fn recolouring_a_mask_source_rebuilds_the_raster() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    uint(&m.mask, LayerMaskBase::MASK_MODE_VALUE_PROPERTY_KEY, 2);
    f.initialize();
    f.size(200.0, 200.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
    let c = color(&m.source);
    recolor(&c, 0xff00ff00);
    assert_eq!(f.frame(), 2);
    recolor(&c, 0xff0000ff);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
}
#[test]
fn a_mask_rerasterizes_when_its_content_or_source_moves() {
    let mut f = Fixture::new(true);
    let m = build_mask_fixture(&f);
    f.initialize();
    f.size(200.0, 200.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
    number(&m.source_group, NodeBase::X_PROPERTY_KEY, 4.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
    number(&m.masked, NodeBase::X_PROPERTY_KEY, 3.0);
    assert_eq!(f.frame(), 2);
    assert_eq!(f.frame(), 0);
}
