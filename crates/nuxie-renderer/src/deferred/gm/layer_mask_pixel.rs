//! All ten renderer/layer_mask_pixel_test.cpp cases at 8398db31.
//! Real Metal readback against a second vector render; no stored golden.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{snapshot_frame, DeferredReplayer},
    deferred_session::DeferredSession,
};
use nuxie_runtime::{
    source::{
        advance_flags::AdvanceFlags,
        artboard::Artboard,
        container_component::ContainerComponent,
        core::{CoreArena, CoreHandle},
        drawable_flag::DrawableFlag,
        generated::{
            component_base::ComponentBase,
            core_registry::CoreRegistry,
            drawable_base::DrawableBase,
            layer_mask_base::LayerMaskBase,
            layout_component_base::LayoutComponentBase,
            node_base::NodeBase,
            shapes::{
                paint::{feather_base::FeatherBase, solid_color_base::SolidColorBase},
                parametric_path_base::ParametricPathBase,
            },
        },
        layer_mask::LayerMask,
        node::Node,
        shapes::{
            paint::{feather::Feather, fill::Fill, solid_color::SolidColor},
            rectangle::Rectangle,
            shape::Shape,
        },
        status_code::StatusCode,
    },
    RuntimeFactoryHandle,
};
const ARTBOARD_SIZE: f32 = 200.0;
struct Diff {
    max_channel: u32,
    mean_channel: f64,
    significant_fraction: f64,
}
impl Diff {
    fn between(a: &[u8], b: &[u8]) -> Self {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len() % 4, 0);
        let (mut total, mut significant, mut max_channel) = (0u64, 0usize, 0u32);
        for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
            let mut worst = 0;
            for c in 0..4 {
                let delta = (i32::from(pa[c]) - i32::from(pb[c])).unsigned_abs();
                total += u64::from(delta);
                worst = worst.max(delta);
            }
            max_channel = max_channel.max(worst);
            if worst > 2 {
                significant += 1;
            }
        }
        let pixels = a.len() / 4;
        Self {
            max_channel,
            mean_channel: total as f64 / (pixels * 4) as f64,
            significant_fraction: significant as f64 / pixels as f64,
        }
    }
    fn report(&self, what: &str) {
        println!(
            "[layer-mask] {what}: max channel {}, mean {:.4}, {:.4}% of pixels off by more than 2",
            self.max_channel,
            self.mean_channel,
            self.significant_fraction * 100.0
        );
    }
}
fn sum_rgb(px: &[u8]) -> u64 {
    px.chunks_exact(4)
        .map(|p| u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2]))
        .sum()
}
fn non_background_pixels(px: &[u8]) -> usize {
    px.chunks_exact(4)
        .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
        .count()
}
#[derive(Clone, Copy)]
struct MaskSpec {
    present: bool,
    mode: LayerMaskMode,
    color: u32,
    covers_everything: bool,
    nested: bool,
    feathered: bool,
}
impl Default for MaskSpec {
    fn default() -> Self {
        Self {
            present: true,
            mode: LayerMaskMode::Alpha,
            color: 0xffffffff,
            covers_everything: false,
            nested: false,
            feathered: false,
        }
    }
}
fn uint(h: &CoreHandle, k: u16, v: u32) {
    assert!(CoreRegistry::set_uint_handle(h, k.into(), v));
}
fn number(h: &CoreHandle, k: u16, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, k.into(), v));
}
struct Scene {
    arena: CoreArena,
    artboard: CoreHandle,
    next: u32,
}
impl Scene {
    fn new(factory: RuntimeFactoryHandle) -> Self {
        let arena = CoreArena::default();
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard
            .with_downcast_mut::<Artboard, _>(|a| {
                a.set_core_arena(arena.clone());
                a.add_object(Some(artboard.clone()));
            })
            .unwrap();
        Self {
            arena,
            artboard,
            next: 1,
        }
    }
    fn add(&mut self, h: &CoreHandle, parent: u32) -> u32 {
        uint(h, ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        self.artboard
            .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(h.clone())))
            .unwrap();
        let id = self.next;
        self.next += 1;
        id
    }
    fn node(&mut self, parent: u32) -> (CoreHandle, u32) {
        let h = self.arena.insert(Node::default());
        let id = self.add(&h, parent);
        (h, id)
    }
    fn rect(&mut self, parent: u32, x: f32, y: f32, size: f32, color: u32, feather: Option<f32>) {
        let s = self.arena.insert(Shape::default());
        number(&s, NodeBase::X_PROPERTY_KEY, x);
        number(&s, NodeBase::Y_PROPERTY_KEY, y);
        let sid = self.add(&s, parent);
        let r = self.arena.insert(Rectangle::default());
        number(&r, ParametricPathBase::WIDTH_PROPERTY_KEY, size);
        number(&r, ParametricPathBase::HEIGHT_PROPERTY_KEY, size);
        self.add(&r, sid);
        let f = self.arena.insert(Fill::default());
        let fid = self.add(&f, sid);
        let c = self.arena.insert(SolidColor::default());
        assert!(CoreRegistry::set_color_handle(
            &c,
            SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
            color as i32
        ));
        self.add(&c, fid);
        if let Some(strength) = feather {
            let blur = self.arena.insert(Feather::default());
            number(&blur, FeatherBase::STRENGTH_PROPERTY_KEY, strength);
            self.add(&blur, fid);
        }
    }
    fn mask(&mut self, parent: u32, source: u32, resolution: f32, mode: LayerMaskMode) {
        let m = self.arena.insert(LayerMask::default());
        self.add(&m, parent);
        uint(&m, LayerMaskBase::SOURCE_ID_PROPERTY_KEY, source);
        number(&m, LayerMaskBase::RESOLUTION_PROPERTY_KEY, resolution);
        uint(&m, LayerMaskBase::MASK_MODE_VALUE_PROPERTY_KEY, mode as u32);
    }
}
fn hide(group: &CoreHandle) {
    ContainerComponent::for_all(group, |h| {
        if h.is_type_of(DrawableBase::TYPE_KEY) {
            let flags =
                CoreRegistry::get_uint_handle(&h, DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY.into())
                    .unwrap();
            uint(
                &h,
                DrawableBase::DRAWABLE_FLAGS_PROPERTY_KEY,
                flags | u32::from(DrawableFlag::HIDDEN.0),
            );
        }
        true
    });
}
fn render_once(
    spec: MaskSpec,
    resolution: f32,
    device_scale: f32,
    rotation: f32,
    disable_raster_ordering: bool,
) -> Vec<u8> {
    let size = (ARTBOARD_SIZE * device_scale).ceil() as u32 + 8;
    let mut host = GmHost::with_size(0xff000000, false, size, size);
    host.set_frame_mode(RenderCanvasFrameMode {
        disable_raster_ordering,
        ..Default::default()
    });
    // Match TestingWindow::beginFrame before recording: supportsLayerMask
    // must inspect this frame's actual interlock, not a previous/default frame.
    use crate::deferred::cmd::deferred_replayer::DeferredFrameSink;
    host.begin_screen_frame(0).expect("pixel screen frame");
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(host.factory.persistent_context());
    let mut scene = Scene::new(RuntimeFactoryHandle::from_factory(&mut session).unwrap());
    let (_, masked) = scene.node(0);
    scene.rect(masked, 30.0, 30.0, 80.0, 0x99ff2020, None);
    scene.rect(masked, 70.0, 70.0, 80.0, 0x9920ff20, None);
    if spec.feathered {
        scene.rect(masked, 150.0, 150.0, 40.0, 0xffffcc20, Some(12.0));
    }
    let mut extra_hidden = None;
    if spec.nested {
        let (_, inner) = scene.node(masked);
        scene.rect(inner, 150.0, 30.0, 40.0, 0xff2020ff, None);
        let (source, source_id) = scene.node(0);
        scene.rect(
            source_id,
            ARTBOARD_SIZE * 0.5,
            ARTBOARD_SIZE * 0.5,
            ARTBOARD_SIZE * 2.0,
            0xffffffff,
            None,
        );
        if spec.present {
            scene.mask(inner, source_id, 1.0, LayerMaskMode::Alpha);
        } else {
            extra_hidden = Some(source);
        }
    } else {
        scene.rect(masked, 150.0, 30.0, 40.0, 0xff2020ff, None);
    }
    let (source, source_id) = scene.node(0);
    if spec.covers_everything {
        scene.rect(
            source_id,
            ARTBOARD_SIZE * 0.5,
            ARTBOARD_SIZE * 0.5,
            ARTBOARD_SIZE * 2.0,
            spec.color,
            None,
        );
    } else {
        scene.rect(source_id, 20.0, 20.0, 120.0, spec.color, None);
    }
    if spec.present {
        scene.mask(masked, source_id, resolution, spec.mode);
    }
    assert_eq!(Artboard::initialize_handle(&scene.artboard), StatusCode::Ok);
    number(
        &scene.artboard,
        LayoutComponentBase::WIDTH_PROPERTY_KEY,
        ARTBOARD_SIZE,
    );
    number(
        &scene.artboard,
        LayoutComponentBase::HEIGHT_PROPERTY_KEY,
        ARTBOARD_SIZE,
    );
    if !spec.present {
        hide(&source);
        if let Some(extra) = extra_hidden {
            hide(&extra);
        }
    }
    Artboard::advance_handle(
        &scene.artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    let screen = session.borrow().screen_renderer(0);
    {
        let mut screen = screen.borrow_mut();
        screen.save();
        screen.transform(Mat2D([device_scale, 0.0, 0.0, device_scale, 0.0, 0.0]));
        if rotation != 0.0 {
            let mid = ARTBOARD_SIZE * 0.5;
            screen.translate(mid, mid);
            let m = nuxie_runtime::source::math::mat2d::Mat2D::from_rotation(rotation);
            screen.transform(Mat2D(*m.values()));
            screen.translate(-mid, -mid);
        }
        Artboard::draw_internal_handle(&scene.artboard, screen.as_mut());
        screen.restore();
    }
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    DeferredReplayer::default().replay_frame(&frame, &mut host);
    assert_eq!(
        host.canvas_frames(),
        if spec.present && !disable_raster_ordering {
            if spec.nested {
                4
            } else {
                2
            }
        } else {
            0
        }
    );
    host.finish()
}
fn render(spec: MaskSpec) -> Vec<u8> {
    render_once(spec, 1.0, 1.0, 0.0, false)
}
fn vectors() -> MaskSpec {
    MaskSpec {
        present: false,
        ..Default::default()
    }
}
#[test]
fn fully_covering_opaque_mask_is_identity() {
    for scale in [1.0, 2.0] {
        let a = render_once(
            MaskSpec {
                covers_everything: true,
                ..Default::default()
            },
            1.0,
            scale,
            0.0,
            false,
        );
        let b = render_once(vectors(), 1.0, scale, 0.0, false);
        assert!(non_background_pixels(&a) > 1000);
        assert!(non_background_pixels(&b) > 1000);
        let d = Diff::between(&a, &b);
        d.report(&format!("identity mask at {scale}x"));
        assert!(d.max_channel <= 2);
        assert_eq!(d.significant_fraction, 0.0);
    }
}
#[test]
fn alpha_and_inverted_partition_layer() {
    let a = render(Default::default());
    let inverted = render(MaskSpec {
        mode: LayerMaskMode::InvertedAlpha,
        ..Default::default()
    });
    let b = render(vectors());
    assert_eq!(a.len(), b.len());
    assert!(non_background_pixels(&a) > 100);
    assert!(non_background_pixels(&inverted) > 100);
    let (mut worst, mut significant) = (0, 0);
    for p in 0..a.len() / 4 {
        for c in 0..4 {
            let i = p * 4 + c;
            let background = if c == 3 { 255 } else { 0 };
            let sum = i32::from(a[i]) + i32::from(inverted[i]) - background;
            let delta = (sum - i32::from(b[i])).unsigned_abs();
            worst = worst.max(delta);
            if delta > 2 {
                significant += 1;
            }
        }
    }
    println!(
        "[layer-mask] alpha + invertedAlpha: worst {worst}, {significant} channels off by more than 2"
    );
    assert_eq!(significant, 0);
}
#[test]
fn opaque_white_luminance_equals_alpha() {
    let a = render(Default::default());
    let b = render(MaskSpec {
        mode: LayerMaskMode::Luminance,
        ..Default::default()
    });
    assert!(non_background_pixels(&a) > 100);
    let d = Diff::between(&a, &b);
    d.report("opaque white: luminance vs alpha");
    assert!(d.max_channel <= 2);
}
#[test]
fn half_transparent_white_luminance_equals_alpha() {
    let a = render(MaskSpec {
        color: 0x80ffffff,
        ..Default::default()
    });
    let b = render(MaskSpec {
        mode: LayerMaskMode::Luminance,
        color: 0x80ffffff,
        ..Default::default()
    });
    assert!(non_background_pixels(&a) > 100);
    let d = Diff::between(&a, &b);
    d.report("50% white: luminance vs alpha");
    assert!(d.max_channel <= 2);
}
#[test]
fn feather_survives_raster_edge() {
    let a = render(MaskSpec {
        covers_everything: true,
        feathered: true,
        ..Default::default()
    });
    let b = render(MaskSpec {
        feathered: true,
        ..vectors()
    });
    assert!(non_background_pixels(&a) > 1000);
    assert!(non_background_pixels(&b) > 1000);
    let d = Diff::between(&a, &b);
    d.report("feathered content under identity mask");
    assert!(d.max_channel <= 2);
}
#[test]
fn rotated_composite_matches_vectors() {
    let a = render_once(
        MaskSpec {
            covers_everything: true,
            ..Default::default()
        },
        1.0,
        1.0,
        0.37,
        false,
    );
    let b = render_once(vectors(), 1.0, 1.0, 0.37, false);
    assert!(non_background_pixels(&a) > 1000);
    assert!(non_background_pixels(&b) > 1000);
    let d = Diff::between(&a, &b);
    d.report("identity mask under rotation");
    assert!(d.max_channel <= 48);
    assert!(d.significant_fraction < 0.03);
}
#[test]
fn luminance_weights_rec601() {
    let reference = render(Default::default());
    assert!(non_background_pixels(&reference) > 100);
    let full = sum_rgb(&reference) as f64;
    assert!(full > 0.0);
    let mut measured = [0.0; 3];
    for (i, (name, color, expected)) in [
        ("red", 0xffff0000, 0.30),
        ("green", 0xff00ff00, 0.59),
        ("blue", 0xff0000ff, 0.11),
    ]
    .into_iter()
    .enumerate()
    {
        let lit = render(MaskSpec {
            mode: LayerMaskMode::Luminance,
            color,
            ..Default::default()
        });
        measured[i] = sum_rgb(&lit) as f64 / full;
        println!(
            "[layer-mask] luminance {name}: kept {:.4}, expected {expected}",
            measured[i]
        );
        assert!((measured[i] - expected).abs() <= 0.01);
    }
    assert!(measured[1] > measured[0]);
    assert!(measured[0] > measured[2]);
}
#[test]
fn nested_mask_composites() {
    let a = render(MaskSpec {
        covers_everything: true,
        nested: true,
        ..Default::default()
    });
    let b = render(MaskSpec {
        nested: true,
        ..vectors()
    });
    assert!(non_background_pixels(&a) > 1000);
    assert!(non_background_pixels(&b) > 1000);
    let d = Diff::between(&a, &b);
    d.report("nested identity masks");
    assert!(d.max_channel <= 2);
    assert_eq!(d.significant_fraction, 0.0);
}
#[test]
fn partial_mask_only_removes_content() {
    let a = render(Default::default());
    let b = render(vectors());
    assert_eq!(a.len(), b.len());
    let (mut removed, mut added) = (0, 0);
    for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let was = pb[..3].iter().any(|&v| v != 0);
        let is = pa[..3].iter().any(|&v| v != 0);
        if was && !is {
            removed += 1;
        }
        if !was && is {
            added += 1;
        }
        for c in 0..3 {
            assert!(i32::from(pa[c]) <= i32::from(pb[c]) + 2);
        }
    }
    println!("[layer-mask] partial mask: {removed} pixels removed, {added} appeared");
    assert!(removed > 100);
    assert_eq!(added, 0);
}
#[test]
fn unsupported_frame_draws_unmasked() {
    let supported = render(Default::default());
    let supported_vectors = render(vectors());
    assert_eq!(supported.len(), supported_vectors.len());
    assert!(non_background_pixels(&supported) > 100);
    let d = Diff::between(&supported, &supported_vectors);
    d.report("masking supported");
    assert!(d.significant_fraction > 0.0);
    let a = render_once(Default::default(), 1.0, 1.0, 0.0, true);
    let b = render_once(vectors(), 1.0, 1.0, 0.0, true);
    assert_eq!(a.len(), b.len());
    assert!(non_background_pixels(&a) > 100);
    let d = Diff::between(&a, &b);
    d.report("masking unsupported");
    assert!(d.max_channel <= 2);
    assert_eq!(d.significant_fraction, 0.0);
}
