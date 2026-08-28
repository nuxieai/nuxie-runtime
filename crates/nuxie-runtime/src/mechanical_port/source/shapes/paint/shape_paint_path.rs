use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    math::{
        aabb::Aabb,
        mat2d::Mat2D,
        raw_path::{PathDirection, RawPath},
    },
};
use nuxie_render_api::{FillRule, RawPath as RenderRawPath, RenderPath};
pub struct ShapePaintPath {
    render_path_dirty: bool,
    render_path: Option<Box<dyn RenderPath>>,
    raw_path: RawPath,
    is_local: bool,
    fill_rule: FillRule,
}
impl Default for ShapePaintPath {
    fn default() -> Self {
        Self::new(true)
    }
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
    fn render_raw_path(&self) -> RenderRawPath {
        let mut result = RenderRawPath::new();
        let mut point_index = 0;
        for verb in self.raw_path.verbs() {
            match verb {
                crate::mechanical_port::source::math::path_types::PathVerb::Move => {
                    let point = self.raw_path.points()[point_index];
                    point_index += 1;
                    result.move_to(point.x, point.y);
                }
                crate::mechanical_port::source::math::path_types::PathVerb::Line => {
                    let point = self.raw_path.points()[point_index];
                    point_index += 1;
                    result.line_to(point.x, point.y);
                }
                crate::mechanical_port::source::math::path_types::PathVerb::Quad => {
                    let control = self.raw_path.points()[point_index];
                    let point = self.raw_path.points()[point_index + 1];
                    point_index += 2;
                    result.quad_to(control.x, control.y, point.x, point.y);
                }
                crate::mechanical_port::source::math::path_types::PathVerb::Cubic => {
                    let out = self.raw_path.points()[point_index];
                    let incoming = self.raw_path.points()[point_index + 1];
                    let point = self.raw_path.points()[point_index + 2];
                    point_index += 3;
                    result.cubic_to(out.x, out.y, incoming.x, incoming.y, point.x, point.y);
                }
                crate::mechanical_port::source::math::path_types::PathVerb::Close => {
                    result.close();
                }
            }
        }
        result
    }

    pub fn render_path(&mut self, factory: &RuntimeFactoryHandle) -> &mut dyn RenderPath {
        let raw_path = self.render_raw_path();
        if self.render_path.is_none() {
            let mut path = factory.with_factory_mut(|factory| factory.make_empty_render_path());
            path.add_raw_path(&raw_path);
            path.fill_rule(self.fill_rule);
            self.render_path = Some(path);
            self.render_path_dirty = false;
        } else if self.render_path_dirty {
            let path = self.render_path.as_mut().unwrap();
            path.rewind();
            path.add_raw_path(&raw_path);
            self.render_path_dirty = false;
        }
        self.render_path.as_deref_mut().unwrap()
    }
}
