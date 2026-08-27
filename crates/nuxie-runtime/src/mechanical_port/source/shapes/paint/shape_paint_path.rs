use crate::mechanical_port::source::{
    component::Component,
    factory::Factory,
    math::{
        aabb::Aabb,
        mat2d::Mat2D,
        raw_path::{PathDirection, RawPath},
    },
    refcnt::Rcp,
    renderer::{FillRule, RenderPath},
};
pub struct ShapePaintPath {
    render_path_dirty: bool,
    render_path: Option<Rcp<RenderPath>>,
    raw_path: RawPath,
    is_local: bool,
    fill_rule: FillRule,
}
impl ShapePaintPath {
    pub fn new(is_local: bool) -> Self {
        Self {
            render_path_dirty: true,
            render_path: None,
            raw_path: RawPath::default(),
            is_local,
            fill_rule: FillRule::Clockwise,
        }
    }
    pub fn with_fill_rule(is_local: bool, fill_rule: FillRule) -> Self {
        Self {
            fill_rule,
            ..Self::new(is_local)
        }
    }
    pub fn raw_path(&self) -> &RawPath {
        &self.raw_path
    }
    pub fn mutable_raw_path(&mut self) -> &mut RawPath {
        &mut self.raw_path
    }
    pub fn is_local(&self) -> bool {
        self.is_local
    }
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }
    pub fn empty(&self) -> bool {
        self.raw_path.empty()
    }
    pub fn rewind(&mut self) {
        self.raw_path.rewind();
        self.render_path_dirty = true;
    }
    pub fn rewind_as(&mut self, is_local: bool, fill_rule: FillRule) {
        self.is_local = is_local;
        self.fill_rule = fill_rule;
        self.rewind();
    }
    pub fn rewind_local(&mut self, is_local: bool) {
        self.is_local = is_local;
        self.rewind();
    }
    pub fn add_path(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let iterator = self.raw_path.add_path(raw_path, transform);
        self.raw_path.prune_empty_segments(iterator);
        self.render_path_dirty = true;
    }
    pub fn add_shape_paint_path(&mut self, path: &ShapePaintPath, transform: Option<&Mat2D>) {
        self.add_path(path.raw_path(), transform);
    }
    pub fn add_path_backwards(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let iterator = self.raw_path.add_path_backwards(raw_path, transform);
        self.raw_path.prune_empty_segments(iterator);
        self.render_path_dirty = true;
    }
    pub fn add_shape_paint_path_backwards(
        &mut self,
        path: &ShapePaintPath,
        transform: Option<&Mat2D>,
    ) {
        self.add_path_backwards(path.raw_path(), transform);
    }
    pub fn add_path_clockwise(&mut self, raw_path: &RawPath, transform: Option<&Mat2D>) {
        let mut area = raw_path.compute_coarse_area();
        if let Some(transform) = transform {
            area *= transform.determinant();
        }
        if area < 0.0 {
            self.add_path_backwards(raw_path, transform);
        } else {
            self.add_path(raw_path, transform);
        }
    }
    pub fn add_rect(&mut self, aabb: Aabb, direction: PathDirection) {
        self.raw_path.add_rect(aabb, direction);
    }
    pub fn has_render_path(&self) -> bool {
        self.render_path.is_some() && !self.render_path_dirty
    }
    pub fn render_path_for_component(&mut self, component: &Component) -> &mut RenderPath {
        self.render_path_for_factory(component.artboard().factory_mut())
    }
    pub fn render_path_for_factory(&mut self, factory: &mut Factory) -> &mut RenderPath {
        if self.render_path.is_none() {
            let mut path = factory.make_empty_render_path();
            path.add_raw_path(&self.raw_path);
            path.set_fill_rule(self.fill_rule);
            self.render_path = Some(path);
            self.render_path_dirty = false;
        } else if self.render_path_dirty {
            let path = self.render_path.as_mut().unwrap();
            path.rewind();
            path.add_raw_path(&self.raw_path);
            self.render_path_dirty = false;
        }
        self.render_path.as_mut().unwrap()
    }
}
