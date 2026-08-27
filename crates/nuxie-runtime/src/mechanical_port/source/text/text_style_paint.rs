use crate::mechanical_port::source::{
    color::ColorInt,
    generated::text::text_style_paint_base::TextStylePaintBase,
    math::{mat2d::Mat2D, raw_path::RawPath},
    refcnt::RiveRc,
    renderer::{RenderPaint, Renderer},
    shapes::{shape_paint_container::ShapePaintContainer, shape_paint_path::ShapePaintPath},
};
use std::collections::BTreeMap;
pub struct TextStylePaint {
    pub base: TextStylePaintBase,
    pub paints: ShapePaintContainer,
    opacity_paths:
        BTreeMap<crate::mechanical_port::source::math::ordered_float::OrderedF32, ShapePaintPath>,
    paint_pool: Vec<RiveRc<RenderPaint>>,
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
            path: ShapePaintPath::clockwise(),
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
            self.path.add_path_clockwise(raw);
            self.opacity_paths
                .entry(opacity.into())
                .or_insert_with(ShapePaintPath::clockwise)
                .add_path_clockwise(raw);
        }
        !had
    }
    pub fn draw(&mut self, renderer: &mut Renderer, world: &Mat2D) {
        for paint in self.paints.shape_paints_mut() {
            if !paint.should_draw() {
                continue;
            }
            paint.set_blend_mode(self.base.parent_text().blend_mode());
            if let Some(path) = self.opacity_paths.get_mut(&1.0.into()) {
                paint.draw(renderer, path, world, true, None);
            }
            while self.paint_pool.len() < self.opacity_paths.len() {
                self.paint_pool
                    .push(self.base.artboard().factory().make_render_paint());
            }
            let mut index = 0;
            for (opacity, path) in &mut self.opacity_paths {
                if opacity.value() == 1.0 {
                    continue;
                }
                let pooled = &mut self.paint_pool[index];
                index += 1;
                paint.apply_to(pooled, opacity.value());
                pooled.set_feather(paint.feather().map_or(0.0, |f| f.strength()));
                paint.draw(renderer, path, world, true, Some(pooled));
            }
        }
    }
    pub fn foreground_color(&self) -> ColorInt {
        for paint in self.paints.shape_paints() {
            if let Some(color) = paint.solid_fill_color() {
                return color;
            }
        }
        0xff000000
    }
    pub fn shape_world_transform(&self) -> &Mat2D {
        self.base.parent_text().shape_world_transform()
    }
    pub fn path_builder(&self) -> crate::mechanical_port::source::core::CoreHandle {
        self.base.parent().expect("TextStylePaint parent")
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
    pub fn clone_value(&self) -> Box<Self> {
        let mut twin = self.base.clone_text_style_paint();
        if let Some(asset) = self.base.file_asset() {
            twin.base.set_asset(asset.clone());
        }
        twin
    }
}
