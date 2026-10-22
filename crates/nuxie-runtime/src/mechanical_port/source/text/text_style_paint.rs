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
use nuxie_render_api::{RenderPaint, Renderer};
#[derive(Clone, Copy, PartialEq)]
struct Opacity(f32);
impl Eq for Opacity {}
impl PartialOrd for Opacity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Opacity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// The glyph paths of each opacity, ordered by opacity as the source's
/// std::map orders them. A style holds a handful at most, so a sorted vector
/// keeps the map's order and lookups without its tree walks.
#[derive(Default)]
struct OpacityPaths(Vec<(Opacity, ShapePaintPath)>);

impl OpacityPaths {
    fn clear(&mut self) {
        self.0.clear();
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    /// The path of `opacity`, inserted empty in order when absent.
    fn entry_or_default(&mut self, opacity: Opacity) -> &mut ShapePaintPath {
        let index = match self.0.binary_search_by(|(key, _)| key.cmp(&opacity)) {
            Ok(index) => index,
            Err(index) => {
                self.0.insert(index, (opacity, ShapePaintPath::default()));
                index
            }
        };
        &mut self.0[index].1
    }

    fn get_mut(&mut self, opacity: Opacity) -> Option<&mut ShapePaintPath> {
        self.0
            .binary_search_by(|(key, _)| key.cmp(&opacity))
            .ok()
            .map(|index| &mut self.0[index].1)
    }

    fn all_opaque(&self) -> bool {
        self.0.iter().all(|(opacity, _)| opacity.0 == 1.0)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (Opacity, &mut ShapePaintPath)> {
        self.0.iter_mut().map(|(opacity, path)| (*opacity, path))
    }
}
pub struct TextStylePaint {
    pub base: TextStylePaintBase,
    pub paints: ShapePaintContainer,
    opacity_paths: OpacityPaths,
    paint_pool: Vec<Box<dyn RenderPaint>>,
    path: ShapePaintPath,
    has_contents: bool,
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
            opacity_paths: OpacityPaths::default(),
            paint_pool: Vec::new(),
            path: ShapePaintPath::default(),
            has_contents: false,
            background: None,
            decoration: None,
            css_text: None,
        }
    }
    pub fn rewind_path(&mut self) {
        self.path.rewind();
        self.has_contents = false;
        self.opacity_paths.clear();
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
        let had = self.has_contents;
        self.has_contents = true;
        if opacity > 0.0 {
            self.path.add_path_clockwise(raw, None);
            self.opacity_paths
                .entry_or_default(Opacity(opacity))
                .add_path_clockwise(raw, None);
        }
        !had
    }
    pub fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        world: &Mat2D,
        blend: nuxie_render_api::BlendMode,
        additive_amount: u8,
    ) {
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
        let only_full_opacity = self.opacity_paths.all_opaque();
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
                paint.cells.apply_blend_mode(blend, additive_amount);
                if let Some(path) = self.opacity_paths.get_mut(Opacity(1.0)) {
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
                if self.paint_pool.len() < self.opacity_paths.len() {
                    let factory = self
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten()
                        .expect("TextStylePaint requires its imported renderer factory");
                    while self.paint_pool.len() < self.opacity_paths.len() {
                        self.paint_pool
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
                paint.shape_paint_mut().blend_mode(blend, additive_amount);
                if let Some(path) = self.opacity_paths.get_mut(Opacity(1.0)) {
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
                if self.paint_pool.len() < self.opacity_paths.len() {
                    let factory = self
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten()
                        .expect("TextStylePaint requires its imported renderer factory");
                    while self.paint_pool.len() < self.opacity_paths.len() {
                        self.paint_pool
                            .push(factory.with_factory_mut(|factory| factory.make_render_paint()));
                    }
                }
                let mut index = 0;
                for (opacity, path) in self.opacity_paths.iter_mut() {
                    if opacity.0 == 1.0 {
                        continue;
                    }
                    let pooled = self.paint_pool[index].as_mut();
                    index += 1;
                    paint.apply_to(pooled, opacity.0);
                    let strength = paint
                        .shape_paint()
                        .feather()
                        .and_then(|feather| {
                            feather.with_downcast::<Feather, _>(|feather| feather.base.strength())
                        })
                        .unwrap_or(0.0);
                    pooled.feather(strength);
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
    fn parent_world_transform(parent: &CoreHandle) -> Mat2D {
        parent
            .text_shape_world_transform()
            .expect("TextStylePaint parent is Text")
    }
    pub fn path_builder(&self) -> CoreHandle {
        self.base.parent_handle().expect("TextStylePaint parent")
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
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
