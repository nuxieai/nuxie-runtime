use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::feather_base::FeatherBase,
    math::{mat2d::Mat2D, raw_path::PathDirection, vec2d::Vec2D},
    shapes::{
        paint::{fill::Fill, shape_paint::ShapePaint, shape_paint_path::ShapePaintPath},
        shape::Shape,
        shape_paint_container::ShapePaintContainer,
    },
    transform_space::TransformSpace,
};
pub struct Feather {
    pub base: FeatherBase,
    inner_path: ShapePaintPath,
    effect_path_dirty: bool,
    #[cfg(test)]
    pub render_count: i32,
}
impl Feather {
    pub fn validate(&self, context: &CoreContext) -> bool {
        if !self.base.validate(context) {
            return false;
        }
        let object = context
            .resolve(self.base.parent_id())
            .expect("base validates parent");
        object.is::<ShapePaint>()
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        let this = self as *mut _;
        self.base
            .parent_mut()
            .as_mut::<ShapePaint>()
            .unwrap()
            .set_feather(unsafe { &mut *this });
        code
    }
    pub fn space(&self) -> TransformSpace {
        TransformSpace::from(self.base.space_value())
    }
    pub fn is_inner(&self) -> bool {
        self.base.inner() && self.base.parent().is_some_and(|parent| parent.is::<Fill>())
    }
    pub fn update(&mut self, value: ComponentDirt) {
        let paint = self.base.parent_mut().as_mut::<ShapePaint>().unwrap();
        if has_dirt(value, ComponentDirt::PAINT) {
            paint
                .render_paint_mut()
                .unwrap()
                .set_feather(self.base.strength());
        }
        if has_dirt(value, ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH) && self.is_inner()
        {
            if let Some(shape) = ShapePaintContainer::from(paint.base.parent_mut()) {
                let transform = shape.shape_world_transform();
                let Some(path) = paint.pick_path(shape) else {
                    return;
                };
                self.rebuild_inner_path(path, &transform, self.space() == TransformSpace::World);
                self.effect_path_dirty = true;
            }
            #[cfg(test)]
            {
                self.render_count += 1;
            }
        }
    }
    pub fn rebuild_inner_path(
        &mut self,
        path: &ShapePaintPath,
        shape_transform: &Mat2D,
        offset_in_artboard: bool,
    ) {
        self.effect_path_dirty = false;
        let bounds = path.raw_path().bounds().pad(self.base.strength() * 1.5);
        let mut offset = Vec2D::new(self.base.offset_x(), self.base.offset_y());
        if offset_in_artboard {
            offset = Vec2D::transform_dir(offset, shape_transform.invert_or_identity());
        }
        self.inner_path.rewind();
        self.inner_path.add_rect(bounds, PathDirection::Cw);
        let transform = Mat2D::from_translation(offset);
        self.inner_path
            .add_path_backwards(path.raw_path(), Some(&transform));
    }
    pub fn build_dependencies(&mut self) {
        let Some(shape) = self
            .base
            .parent_mut()
            .as_mut::<ShapePaint>()
            .unwrap()
            .base
            .parent_mut()
        else {
            return;
        };
        if let Some(shape) = shape.as_mut::<Shape>() {
            shape
                .path_composer_mut()
                .add_dependent(self.base.as_component_mut_ptr());
        } else {
            shape.add_dependent(self.base.as_component_mut_ptr());
        }
    }
    pub fn strength_changed(&mut self) {
        self.base.add_dirt(if self.base.inner() {
            ComponentDirt::PAINT | ComponentDirt::WORLD_TRANSFORM
        } else {
            ComponentDirt::PAINT
        });
    }
    pub fn offset_x_changed(&mut self) {
        self.strength_changed();
    }
    pub fn offset_y_changed(&mut self) {
        self.strength_changed();
    }
    pub fn inner_path_mut(&mut self) -> &mut ShapePaintPath {
        &mut self.inner_path
    }
    pub fn effect_path_dirty(&self) -> bool {
        self.effect_path_dirty
    }
    pub fn mark_effect_path_dirty(&mut self) {
        self.effect_path_dirty = true;
    }
}
