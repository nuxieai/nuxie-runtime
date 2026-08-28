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
use std::collections::BTreeMap;
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
pub struct TextStylePaint {
    pub base: TextStylePaintBase,
    pub paints: ShapePaintContainer,
    opacity_paths: BTreeMap<Opacity, ShapePaintPath>,
    paint_pool: Vec<Box<dyn RenderPaint>>,
    path: ShapePaintPath,
    has_contents: bool,
}
impl TextStylePaint {
    pub fn new() -> Self {
        Self {
            base: TextStylePaintBase::default(),
            paints: ShapePaintContainer::default(),
            opacity_paths: BTreeMap::new(),
            paint_pool: Vec::new(),
            path: ShapePaintPath::default(),
            has_contents: false,
        }
    }
    pub fn rewind_path(&mut self) {
        self.path.rewind();
        self.has_contents = false;
        self.opacity_paths.clear();
    }
    pub fn add_path(&mut self, raw: &RawPath, opacity: f32) -> bool {
        let had = self.has_contents;
        self.has_contents = true;
        if opacity > 0.0 {
            self.path.add_path_clockwise(raw, None);
            self.opacity_paths
                .entry(Opacity(opacity))
                .or_default()
                .add_path_clockwise(raw, None);
        }
        !had
    }
    pub fn draw(&mut self, renderer: &mut dyn Renderer, world: &Mat2D) {
        for handle in self.paints.shape_paints().to_vec() {
            handle.with_mut(|object| {
                let Some(paint) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.shape_paint().should_draw() {
                    return;
                }
                let blend = self
                    .base
                    .parent_handle()
                    .and_then(|parent| {
                        parent
                            .with(|parent| parent.as_drawable().map(|parent| parent.blend_mode()))
                            .flatten()
                    })
                    .expect("TextStylePaint parent is Text");
                paint.shape_paint_mut().blend_mode(blend);
                if let Some(path) = self.opacity_paths.get_mut(&Opacity(1.0)) {
                    paint
                        .shape_paint_mut()
                        .draw(renderer, path, *world, true, None, true);
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
                for (opacity, path) in &mut self.opacity_paths {
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
                    paint
                        .shape_paint_mut()
                        .draw(renderer, path, *world, true, Some(pooled), true);
                }
            });
        }
    }
    pub fn foreground_color(&self) -> ColorInt {
        for paint in self.paints.shape_paints() {
            let color = paint
                .with_downcast::<Fill, _>(|fill| {
                    fill.base.paint().and_then(|mutator| {
                        mutator.with_downcast::<SolidColor, _>(|color| color.base.color_value())
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
        self.base
            .parent_handle()
            .and_then(|parent| {
                parent
                    .with(|parent| parent.as_text().map(|text| *text.shape_world_transform()))
                    .flatten()
            })
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
