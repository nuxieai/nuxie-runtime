use crate::mechanical_port::source::{
    generated::shapes::paint::fill_base::{FillBase, FillRule},
    renderer::{RenderPaint, RenderPaintStyle},
    shapes::{
        paint::{
            shape_paint::{ShapePaintPath, ShapePaintType},
            shape_paint_mutator::ShapePaintMutator,
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
    },
};
pub struct Fill {
    pub base: FillBase,
}
impl Fill {
    pub fn path_flags(&self) -> PathFlags {
        if FillRule::from(self.base.fill_rule()) == FillRule::Clockwise {
            PathFlags::LOCAL_CLOCKWISE
        } else {
            PathFlags::LOCAL
        }
    }
    pub fn init_render_paint(&mut self, mutator: &mut dyn ShapePaintMutator) -> &mut RenderPaint {
        let paint = self.base.init_render_paint(mutator);
        paint.set_style(RenderPaintStyle::Fill);
        paint
    }
    pub fn apply_to(&mut self, paint: &mut RenderPaint, opacity: f32) {
        paint.set_style(RenderPaintStyle::Fill);
        paint.set_shader(None);
        self.base.paint_mutator_mut().apply_to(paint, opacity);
    }
    pub fn pick_path<'a>(&self, shape: &'a mut ShapePaintContainer) -> &'a mut ShapePaintPath {
        if FillRule::from(self.base.fill_rule()) == FillRule::Clockwise {
            shape.local_clockwise_path_mut()
        } else {
            shape.local_path_mut()
        }
    }
    pub fn paint_type(&self) -> ShapePaintType {
        ShapePaintType::Fill
    }
    pub fn build_dependencies(&mut self) {
        if !self.base.effects().is_empty() {
            if let Some(container) = ShapePaintContainer::from(self.base.parent_mut()) {
                container
                    .path_builder_mut()
                    .add_dependent(self.base.as_component_mut_ptr());
            }
        }
    }
}
