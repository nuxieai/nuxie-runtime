pub use crate::mechanical_port::source::generated::shapes::paint::stroke_base::StrokeBase;
use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::paint::stroke_base::StrokeBase,
    renderer::{RenderPaint, RenderPaintStyle},
    shapes::{
        paint::{
            shape_paint::{ShapePaintPath, ShapePaintType},
            shape_paint_mutator::ShapePaintMutator,
            stroke_cap::StrokeCap,
            stroke_join::StrokeJoin,
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
    },
};
pub struct Stroke {
    pub base: StrokeBase,
}
impl Stroke {
    pub fn path_flags(&self) -> PathFlags {
        if self.base.transform_affects_stroke() {
            PathFlags::LOCAL
        } else {
            PathFlags::WORLD
        }
    }
    pub fn init_render_paint(&mut self, mutator: &mut dyn ShapePaintMutator) -> &mut RenderPaint {
        let paint = self.base.init_render_paint(mutator);
        paint.set_style(RenderPaintStyle::Stroke);
        paint.set_thickness(self.base.thickness());
        paint.set_cap(StrokeCap::from(self.base.cap()));
        paint.set_join(StrokeJoin::from(self.base.join()));
        paint
    }
    pub fn apply_to(&mut self, paint: &mut RenderPaint, opacity: f32) {
        paint.set_style(RenderPaintStyle::Stroke);
        paint.set_thickness(self.base.thickness());
        paint.set_cap(StrokeCap::from(self.base.cap()));
        paint.set_join(StrokeJoin::from(self.base.join()));
        paint.set_shader(None);
        self.base.paint_mutator_mut().apply_to(paint, opacity);
    }
    pub fn is_visible(&self) -> bool {
        self.base.super_is_visible() && self.base.thickness() > 0.0
    }
    pub fn thickness_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn cap_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn join_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        if has_dirt(value, ComponentDirt::PAINT) {
            let paint = self.base.render_paint_mut().unwrap();
            paint.set_thickness(self.base.thickness());
            paint.set_cap(StrokeCap::from(self.base.cap()));
            paint.set_join(StrokeJoin::from(self.base.join()));
        }
    }
    pub fn invalidate_rendering(&mut self) {
        self.base.render_paint_mut().unwrap().invalidate_stroke();
        self.base.super_invalidate_rendering();
    }
    pub fn pick_path<'a>(&self, shape: &'a mut ShapePaintContainer) -> &'a mut ShapePaintPath {
        if self.base.transform_affects_stroke() {
            shape.local_path_mut()
        } else {
            shape.world_path_mut()
        }
    }
    pub fn build_dependencies(&mut self) {
        if let Some(container) = ShapePaintContainer::from(self.base.parent_mut()) {
            container
                .path_builder_mut()
                .add_dependent(self.base.as_component_mut_ptr());
        }
    }
    pub fn paint_type(&self) -> ShapePaintType {
        ShapePaintType::Stroke
    }
}
