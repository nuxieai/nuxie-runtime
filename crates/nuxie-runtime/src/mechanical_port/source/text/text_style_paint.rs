use super::text_style_geometry::{Opacity, TextStyleGeometry};
use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::text::text_style_paint_base::TextStylePaintBase,
    math::{mat2d::Mat2D, raw_path::RawPath},
    shapes::{
        paint::{
            color::ColorInt, feather::Feather, fill::Fill, shape_paint_path::ShapePaintPath,
            solid_color::SolidColor,
        },
        shape_paint_container::ShapePaintContainer,
    },
};
use nuxie_render_api::Renderer;
pub struct TextStylePaint {
    pub base: TextStylePaintBase,
    pub paints: ShapePaintContainer,
    geometry: TextStyleGeometry,
    is_shared: bool,
    background: Option<CoreHandle>,
    /// Nuxie addition: the style's `TextStyleDecoration`, when it has one.
    decoration: Option<CoreHandle>,
    css_text: Option<CoreHandle>,
}
impl TextStylePaint {
    pub fn new() -> Self {
        Self {
            base: TextStylePaintBase::default(),
            paints: ShapePaintContainer::default(),
            geometry: TextStyleGeometry::default(),
            is_shared: false,
            background: None,
            decoration: None,
            css_text: None,
        }
    }
    pub fn rewind_path(&mut self) {
        self.geometry.rewind();
    }
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
    pub fn set_shared(&mut self, value: bool) {
        self.is_shared = value;
    }
    pub fn background(&self) -> Option<CoreHandle> {
        self.background.clone()
    }
    pub fn set_background(&mut self, background: CoreHandle) {
        self.background = Some(background);
    }
    pub fn decoration(&self) -> Option<CoreHandle> {
        self.decoration.clone()
    }
    pub fn set_decoration(&mut self, decoration: CoreHandle) {
        self.decoration = Some(decoration);
    }
    pub fn css_text(&self) -> Option<CoreHandle> {
        self.css_text.clone()
    }
    pub fn set_css_text(&mut self, record: CoreHandle) {
        self.css_text = Some(record);
    }
    pub fn add_path(&mut self, raw: &RawPath, opacity: f32) -> bool {
        self.geometry.add_path(raw, opacity)
    }
    pub fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        world: &Mat2D,
        blend: nuxie_render_api::BlendMode,
        additive_amount: u8,
    ) {
        let mut geometry = std::mem::take(&mut self.geometry);
        self.draw_geometry(renderer, world, &mut geometry, 1.0, blend, additive_amount);
        self.geometry = geometry;
    }
    pub fn draw_geometry(
        &mut self,
        renderer: &mut dyn Renderer,
        world: &Mat2D,
        geometry: &mut TextStyleGeometry,
        opacity_modifier: f32,
        blend: nuxie_render_api::BlendMode,
        additive_amount: u8,
    ) {
        let owned = opacity_modifier == 1.0;
        let text_parent = self.base.parent_handle().is_some_and(|parent| {
            parent.is_type_of(crate::source::generated::text::text_base::TextBase::TYPE_KEY)
        });
        let base = &self.base;
        // Resolve the actual parent field at the source read site. The draw
        // argument is a distinct matrix and may not be used as a fallback.
        let container_transform = || {
            Self::parent_world_transform(
                &base.parent_handle().expect("TextStylePaint parent is Text"),
            )
        };
        // Every opacity drawn at full: no pooled paint needs the paint's own
        // applyTo, so a plain paint can draw from its cells.
        let only_full_opacity = owned && geometry.opacity_paths.all_opaque();
        // The paint list can't change while this style is borrowed.
        let paints = self.paints.cells().paints();
        for paint in paints.iter() {
            if only_full_opacity
                && paint.cells.is_plain()
                && paint.handle.is_occupied_settled()
                && let Some(mutator_visible) = paint.cells.settled_mutator_is_visible()
            {
                // ShapePaint::shouldDraw.
                if !(paint.cells.is_visible() && mutator_visible) {
                    continue;
                }
                if text_parent {
                    paint.cells.apply_blend_mode(blend, additive_amount);
                }
                if let Some(path) = geometry
                    .opacity_paths
                    .get_mut(Opacity(1.0))
                    .filter(|_| owned)
                {
                    let handle = &paint.handle;
                    // The glyph path's own fill rule: no paint fill rule.
                    paint
                        .cells
                        .draw_plain(renderer, path, *world, true, None, &|| {
                            // The factory the paint's own draw would read.
                            handle
                                .with(|object| {
                                    object
                                        .as_component()
                                        .and_then(|component| {
                                            component.with_artboard(|artboard| artboard.factory())
                                        })
                                        .flatten()
                                })
                                .flatten()
                                .expect("a drawing ShapePaint has its imported Artboard factory")
                        });
                }
                // The pool grows as the paint's own draw grows it.
                if geometry.paint_pool.len() < geometry.opacity_paths.len() {
                    let factory = self
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten()
                        .expect("TextStylePaint requires its imported renderer factory");
                    while geometry.paint_pool.len() < geometry.opacity_paths.len() {
                        geometry
                            .paint_pool
                            .push(factory.with_factory_mut(|factory| factory.make_render_paint()));
                    }
                }
                continue;
            }
            let handle = &paint.handle;
            handle.with_mut(|object| {
                let Some(paint) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.should_draw() {
                    return;
                }
                let fill_rule = paint.fill_rule();
                if owned && text_parent {
                    paint.shape_paint_mut().blend_mode(blend, additive_amount);
                }
                if let Some(path) = geometry
                    .opacity_paths
                    .get_mut(Opacity(1.0))
                    .filter(|_| owned)
                {
                    paint.shape_paint_mut().draw_with_active_container(
                        renderer,
                        path,
                        *world,
                        true,
                        None,
                        true,
                        fill_rule,
                        &container_transform,
                        None,
                    );
                }
                if geometry.paint_pool.len() < geometry.opacity_paths.len() {
                    let factory = self
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten()
                        .expect("TextStylePaint requires its imported renderer factory");
                    while geometry.paint_pool.len() < geometry.opacity_paths.len() {
                        geometry
                            .paint_pool
                            .push(factory.with_factory_mut(|factory| factory.make_render_paint()));
                    }
                }
                let mut index = 0;
                for (opacity, path) in geometry.opacity_paths.iter_mut() {
                    if owned && opacity.0 == 1.0 {
                        continue;
                    }
                    let pooled = geometry.paint_pool[index].as_mut();
                    index += 1;
                    paint.apply_to(pooled, opacity.0 * opacity_modifier);
                    let strength = paint
                        .shape_paint()
                        .feather()
                        .and_then(|feather| {
                            feather.with_downcast::<Feather, _>(|feather| feather.base.strength())
                        })
                        .unwrap_or(0.0);
                    pooled.feather(strength);
                    if !owned {
                        let shape = paint.shape_paint();
                        let mode = if shape.inherits_blend_mode() {
                            nuxie_render_api::BlendMode::SrcOver
                        } else {
                            shape.authored_blend_mode()
                        };
                        pooled.blend_mode(mode);
                        pooled.additiveness(
                            crate::source::shapes::paint::blend_mode::additiveness_for(
                                mode,
                                shape.base.additive_amount(),
                            ),
                        );
                    }
                    paint.shape_paint_mut().draw_with_active_container(
                        renderer,
                        path,
                        *world,
                        true,
                        Some(pooled),
                        true,
                        fill_rule,
                        &container_transform,
                        None,
                    );
                }
            });
        }
    }
    pub fn foreground_color(&self) -> ColorInt {
        for paint in self.paints.shape_paints() {
            let color = paint
                .with_downcast::<Fill, _>(|fill| {
                    fill.base.paint().and_then(|mutator| {
                        mutator
                            .with_downcast::<SolidColor, _>(|color| color.base.color_value() as u32)
                    })
                })
                .flatten();
            if let Some(color) = color {
                return color;
            }
        }
        0xff000000
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        Self::parent_world_transform(
            &self
                .base
                .parent_handle()
                .expect("TextStylePaint parent is Text"),
        )
    }
    pub(super) fn parent_world_transform(parent: &CoreHandle) -> Mat2D {
        if parent.is_type_of(crate::source::generated::text::text_base::TextBase::TYPE_KEY) {
            parent
                .text_shape_world_transform()
                .expect("TextStylePaint Text transform")
        } else {
            parent
                .with_downcast::<crate::source::artboard::Artboard, _>(
                    crate::source::artboard::Artboard::shape_world_transform,
                )
                .expect("global TextStylePaint parent Artboard")
        }
    }
    pub fn path_builder(&self) -> CoreHandle {
        self.base.parent_handle().expect("TextStylePaint parent")
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.geometry.path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.geometry.path
    }
    pub fn clone_value(&self) -> Box<Self> {
        Box::new(Self {
            base: TextStylePaintBase {
                base: *self.base.base.clone_value(),
            },
            ..Self::default()
        })
    }
}

impl Default for TextStylePaint {
    fn default() -> Self {
        Self::new()
    }
}
impl std::ops::Deref for TextStylePaint {
    type Target = TextStylePaintBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextStylePaint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
