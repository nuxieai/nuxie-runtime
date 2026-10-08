//! tests/unit_tests/runtime/layer_mask_test.cpp at 8398db31.
use super::super::{
    command_stream::CommandReader,
    deferred_session::DeferredSession,
    render_commands::{payload_size_of, RenderCmd},
};
use super::render_context_null::{observing_factory, ObservingFactory};
use nuxie_render_api::*;
use nuxie_runtime::{
    source::{
        advance_flags::AdvanceFlags,
        artboard::Artboard,
        core::{CoreArena, CoreHandle},
        drawable::{BoundsFidelity, Drawable},
        generated::{
            component_base::ComponentBase,
            core_registry::CoreRegistry,
            layer_mask_base::LayerMaskBase,
            layout_component_base::LayoutComponentBase,
            node_base::NodeBase,
            shapes::{
                paint::solid_color_base::SolidColorBase, parametric_path_base::ParametricPathBase,
            },
        },
        layer_mask::{LayerMask, LayerMaskOp},
        math::aabb::Aabb,
        node::Node,
        shapes::{
            paint::{fill::Fill, solid_color::SolidColor},
            rectangle::Rectangle,
            shape::Shape,
        },
        status_code::StatusCode,
    },
    RuntimeFactoryHandle,
};

pub(super) fn uint(h: &CoreHandle, k: u16, v: u32) {
    assert!(CoreRegistry::set_uint_handle(h, k.into(), v));
}
pub(super) fn number(h: &CoreHandle, k: u16, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, k.into(), v));
}
pub(super) fn boolean(h: &CoreHandle, k: u16, v: bool) {
    assert!(CoreRegistry::set_bool_handle(h, k.into(), v));
}
pub(super) struct Fixture {
    pub arena: CoreArena,
    pub artboard: CoreHandle,
    pub session: Option<PersistentFactory<DeferredSession>>,
    _context: Option<ObservingFactory>,
}
impl Fixture {
    pub fn new(canvas: bool) -> Self {
        let arena = CoreArena::default();
        let (session, context, factory) = if canvas {
            let (context, _, _) = observing_factory(1, 1);
            let mut session =
                PersistentFactory::new(DeferredSession::with_caps(Default::default()));
            session
                .borrow_mut()
                .bind_render_context(context.persistent_context());
            let factory = RuntimeFactoryHandle::from_factory(&mut session).unwrap();
            (Some(session), Some(context), factory)
        } else {
            let mut factory = PersistentFactory::new(NullFactory::default());
            (
                None,
                None,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            )
        };
        let artboard = arena.insert(Artboard::with_factory(factory));
        artboard.with_downcast_mut::<Artboard, _>(|a| {
            a.set_core_arena(arena.clone());
            a.add_object(Some(artboard.clone()));
        });
        Self {
            arena,
            artboard,
            session,
            _context: context,
        }
    }
    pub fn id(&self, h: &CoreHandle) -> u32 {
        self.artboard
            .with_downcast::<Artboard, _>(|a| a.id_of(h))
            .unwrap()
    }
    pub fn add(&self, h: &CoreHandle, parent: &CoreHandle) {
        uint(h, ComponentBase::PARENT_ID_PROPERTY_KEY, self.id(parent));
        self.artboard
            .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(h.clone())));
    }
    pub fn reparent(&self, h: &CoreHandle, parent: &CoreHandle) {
        uint(h, ComponentBase::PARENT_ID_PROPERTY_KEY, self.id(parent));
    }
    pub fn node(&self, parent: &CoreHandle) -> CoreHandle {
        let h = self.arena.insert(Node::default());
        self.add(&h, parent);
        h
    }
    pub fn shape(&self, parent: &CoreHandle, name: &str) -> CoreHandle {
        let shape = self.arena.insert(Shape::default());
        CoreRegistry::set_string_handle(
            &shape,
            ComponentBase::NAME_PROPERTY_KEY.into(),
            name.into(),
        );
        self.add(&shape, parent);
        let rect = self.arena.insert(Rectangle::default());
        number(&rect, ParametricPathBase::WIDTH_PROPERTY_KEY, 10.0);
        number(&rect, ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0);
        self.add(&rect, &shape);
        let fill = self.arena.insert(Fill::default());
        self.add(&fill, &shape);
        let color = self.arena.insert(SolidColor::default());
        CoreRegistry::set_color_handle(
            &color,
            SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
            0xffff0000u32 as i32,
        );
        self.add(&color, &fill);
        shape
    }
    pub fn mask(&self, parent: &CoreHandle, source: &CoreHandle) -> CoreHandle {
        let h = self.arena.insert(LayerMask::default());
        self.add(&h, parent);
        uint(&h, LayerMaskBase::SOURCE_ID_PROPERTY_KEY, self.id(source));
        h
    }
    pub fn initialize(&self) {
        assert_eq!(Artboard::initialize_handle(&self.artboard), StatusCode::Ok);
    }
    pub fn size(&self, w: f32, h: f32) {
        number(&self.artboard, LayoutComponentBase::WIDTH_PROPERTY_KEY, w);
        number(&self.artboard, LayoutComponentBase::HEIGHT_PROPERTY_KEY, h);
    }
    pub fn advance(&self) {
        Artboard::advance_handle(
            &self.artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }
    pub fn frame(&mut self) -> usize {
        self.advance();
        let session = self.session.as_ref().unwrap();
        let mut renderer = session.borrow().make_screen_renderer(0);
        Artboard::draw_internal_handle(&self.artboard, renderer.as_mut());
        drop(renderer);
        let buffer = session.borrow().command_buffer();
        let opens = {
            let b = buffer.lock().unwrap();
            count_canvas_opens(b.command_bytes(), b.blob_bytes())
        };
        session.borrow_mut().reset_frame();
        opens
    }
    pub fn list_shape(&self) -> String {
        let mut d = self
            .artboard
            .with_downcast::<Artboard, _>(Artboard::first_drawable)
            .flatten();
        let mut out = String::new();
        while let Some(current) = d {
            d = current.with(Drawable::prev_drawable).flatten();
            out.push(if current.is_mask_start() {
                '['
            } else if current.is_mask_end() {
                ']'
            } else if current.is_clip_start() {
                'c'
            } else if current.is_clip_end() {
                'C'
            } else if let Some(marker) = current.layer_mask_marker() {
                if marker.borrow().op == LayerMaskOp::SourceStart {
                    '{'
                } else {
                    '}'
                }
            } else {
                'S'
            });
        }
        out
    }
    pub fn draw_count(&self) -> usize {
        let mut r = CountingRenderer::default();
        Artboard::draw_handle(&self.artboard, &mut r);
        r.paths
    }
}
pub(super) fn count_canvas_opens(commands: &[u8], blobs: &[u8]) -> usize {
    let mut r = CommandReader::new(commands, blobs);
    let mut opens = 0;
    while let Some(byte) = r.next_u8() {
        let Some(cmd) = RenderCmd::from_byte(byte) else {
            break;
        };
        if cmd == RenderCmd::CanvasContentBegin {
            opens += 1;
        }
        r.skip(payload_size_of(cmd));
    }
    opens
}
#[derive(Default)]
struct CountingRenderer {
    paths: usize,
    saves: usize,
    restores: usize,
}
impl Renderer for CountingRenderer {
    fn save(&mut self) {
        self.saves += 1;
    }
    fn restore(&mut self) {
        self.restores += 1;
    }
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {
        self.paths += 1;
    }
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
pub(super) struct MaskFixture {
    pub masked: CoreHandle,
    pub content: CoreHandle,
    pub source_group: CoreHandle,
    pub source: CoreHandle,
    pub mask: CoreHandle,
}
pub(super) fn build_mask_fixture(f: &Fixture) -> MaskFixture {
    let masked = f.node(&f.artboard);
    let content = f.shape(&masked, "Content");
    let source_group = f.node(&f.artboard);
    let source = f.shape(&source_group, "Src");
    let mask = f.mask(&masked, &source_group);
    MaskFixture {
        masked,
        content,
        source_group,
        source,
        mask,
    }
}
pub(super) fn bounds(h: &CoreHandle) -> Aabb {
    let mut out = Aabb::default();
    assert_ne!(
        h.with_mut(|o| o.painted_world_bounds(&mut out)).unwrap(),
        BoundsFidelity::None
    );
    out
}

#[test]
fn a_mask_brackets_its_subtree_and_its_source() {
    let f = Fixture::new(false);
    let masked = f.node(&f.artboard);
    f.shape(&masked, "A");
    f.shape(&masked, "B");
    let source = f.node(&f.artboard);
    f.shape(&source, "Src");
    let mask = f.mask(&masked, &source);
    f.initialize();
    f.advance();
    mask.with_downcast::<LayerMask, _>(|m| {
        assert_eq!(m.source(), Some(source));
        assert!(!m.is_self_referential());
        assert_eq!(m.source_drawables().len(), 1);
    });
    assert_eq!(f.list_shape(), "{S}[SS]");
}
#[test]
fn the_mask_source_is_suppressed_in_the_normal_pass() {
    for visible in [true, false] {
        let f = Fixture::new(false);
        let m = build_mask_fixture(&f);
        f.initialize();
        f.advance();
        assert_eq!(f.draw_count(), 1);
        if visible {
            boolean(&m.mask, LayerMaskBase::SOURCE_DRAWS_PROPERTY_KEY, true);
        } else {
            boolean(&m.mask, LayerMaskBase::IS_VISIBLE_PROPERTY_KEY, false);
        }
        assert_eq!(f.draw_count(), 2);
    }
}
#[test]
fn a_mask_bracket_nests_inside_a_clip_bracket() {
    use nuxie_runtime::source::{
        generated::shapes::clipping_shape_base::ClippingShapeBase,
        shapes::clipping_shape::ClippingShape,
    };
    let f = Fixture::new(false);
    let clipped = f.node(&f.artboard);
    let masked = f.node(&clipped);
    f.shape(&masked, "A");
    let clip_source = f.shape(&f.artboard, "ClipSrc");
    let mask_source = f.shape(&f.artboard, "MaskSrc");
    let clip = f.arena.insert(ClippingShape::default());
    f.add(&clip, &clipped);
    uint(
        &clip,
        ClippingShapeBase::SOURCE_ID_PROPERTY_KEY,
        f.id(&clip_source),
    );
    f.mask(&masked, &mask_source);
    f.initialize();
    f.advance();
    let s = f.list_shape();
    assert!(s.find('c').unwrap() < s.find('[').unwrap());
    assert!(s.find('[').unwrap() < s.find(']').unwrap());
    assert!(s.find(']').unwrap() < s.find('C').unwrap());
}
#[test]
fn a_self_referential_mask_is_dropped_rather_than_recursing() {
    let f = Fixture::new(false);
    let masked = f.node(&f.artboard);
    let a = f.shape(&masked, "A");
    let mask = f.mask(&masked, &a);
    f.initialize();
    f.advance();
    assert!(mask
        .with_downcast::<LayerMask, _>(LayerMask::is_self_referential)
        .unwrap());
    assert_eq!(f.list_shape(), "S");
    assert_eq!(f.draw_count(), 1);
}
#[test]
fn an_unknown_mask_mode_falls_back_to_alpha() {
    use nuxie_runtime::source::layer_mask::MaskMode;
    let arena = CoreArena::default();
    let mask = arena.insert(LayerMask::default());
    uint(&mask, LayerMaskBase::MASK_MODE_VALUE_PROPERTY_KEY, 8);
    assert_eq!(
        mask.with_downcast::<LayerMask, _>(LayerMask::mask_mode),
        Some(MaskMode::Alpha)
    );
    uint(&mask, LayerMaskBase::MASK_MODE_VALUE_PROPERTY_KEY, 3);
    assert_eq!(
        mask.with_downcast::<LayerMask, _>(LayerMask::mask_mode),
        Some(MaskMode::InvertedLuminance)
    );
}

#[test]
fn a_mask_is_not_a_solo_option() {
    use nuxie_runtime::source::solo::Solo;
    let f = Fixture::new(false);
    let solo = f.arena.insert(Solo::default());
    f.add(&solo, &f.artboard);
    let source = f.shape(&f.artboard, "MaskSrc");
    f.mask(&solo, &source);
    f.shape(&solo, "First");
    let second = f.shape(&solo, "Second");
    f.initialize();
    f.advance();
    solo.with_downcast_mut::<Solo, _>(|s| s.update_by_index(1));
    assert_eq!(
        solo.with_downcast::<Solo, _>(Solo::active_component)
            .flatten(),
        Some(second)
    );
}
#[test]
fn a_mask_whose_range_a_draw_rule_splits_is_left_unbracketed() {
    use nuxie_runtime::source::{
        draw_rules::DrawRules,
        draw_target::DrawTarget,
        generated::{draw_rules_base::DrawRulesBase, draw_target_base::DrawTargetBase},
    };
    let f = Fixture::new(false);
    let masked = f.node(&f.artboard);
    f.shape(&masked, "A");
    let b = f.shape(&masked, "B");
    let outsider = f.shape(&f.artboard, "Outsider");
    let source = f.shape(&f.artboard, "MaskSrc");
    let rules = f.arena.insert(DrawRules::default());
    f.add(&rules, &outsider);
    let target = f.arena.insert(DrawTarget::default());
    f.add(&target, &rules);
    uint(&target, DrawTargetBase::DRAWABLE_ID_PROPERTY_KEY, f.id(&b));
    f.mask(&masked, &source);
    f.initialize();
    uint(
        &rules,
        DrawRulesBase::DRAW_TARGET_ID_PROPERTY_KEY,
        f.id(&target),
    );
    f.advance();
    let s = f.list_shape();
    assert_eq!(s.chars().filter(|c| *c == 'S').count(), 4);
    assert!(!s.contains('['));
    assert!(!s.contains('{'));
    assert_eq!(f.draw_count(), 4);
}
#[test]
fn a_mask_nested_inside_a_masked_subtree_brackets_inside_it() {
    let f = Fixture::new(false);
    let outer = f.node(&f.artboard);
    f.shape(&outer, "A");
    let inner = f.node(&outer);
    f.shape(&inner, "B");
    let outer_source = f.shape(&f.artboard, "OuterSrc");
    let inner_source = f.shape(&f.artboard, "InnerSrc");
    f.mask(&outer, &outer_source);
    f.mask(&inner, &inner_source);
    f.initialize();
    f.advance();
    let s = f.list_shape();
    for c in ['[', ']', '{', '}'] {
        assert_eq!(s.chars().filter(|x| *x == c).count(), 2);
    }
    let mut depth = 0;
    for c in s.chars() {
        if c == '[' {
            depth += 1;
        } else if c == ']' {
            depth -= 1;
            assert!(depth >= 0);
        }
    }
    assert_eq!(depth, 0);
    f.advance();
    assert_eq!(f.draw_count(), 2);
}
