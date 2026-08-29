use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::{feather_base::FeatherBase, fill_base::FillBase},
    math::{mat2d::Mat2D, path_types::PathDirection, vec2d::Vec2D},
    shapes::paint::shape_paint_path::ShapePaintPath,
    transform_space::TransformSpace,
};
impl std::ops::Deref for Feather {
    type Target = FeatherBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Feather {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Feather {
    pub const TYPE_KEY: u16 = FeatherBase::TYPE_KEY;
}

pub struct Feather {
    pub base: FeatherBase,
    inner_path: Rc<RefCell<ShapePaintPath>>,
    effect_path_dirty: bool,
    #[cfg(test)]
    pub render_count: i32,
}

impl Default for Feather {
    fn default() -> Self {
        Self {
            base: FeatherBase::default(),
            inner_path: Rc::new(RefCell::new(ShapePaintPath::new(true))),
            effect_path_dirty: true,
            #[cfg(test)]
            render_count: 0,
        }
    }
}

impl Feather {
    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        if !self.base.validate(context) {
            return false;
        }
        context
            .resolve(self.base.parent_id())
            .is_some_and(|object| {
                object
                    .with(|object| object.as_shape_paint().is_some())
                    .unwrap_or(false)
            })
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        let (Some(this), Some(parent)) = (self.base.handle(), self.base.parent_handle()) else {
            return StatusCode::MissingObject;
        };
        let installed = parent
            .with_mut(|parent| {
                parent
                    .as_shape_paint_mut()
                    .map(|paint| paint.set_feather(this))
            })
            .is_some();
        if !installed {
            return StatusCode::InvalidObject;
        }
        code
    }
    pub fn space(&self) -> TransformSpace {
        TransformSpace::from(self.base.space_value())
    }
    pub fn is_inner(&self) -> bool {
        self.base.inner()
            && self
                .base
                .parent_handle()
                .is_some_and(|parent| parent.is_type_of(FillBase::TYPE_KEY))
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PAINT) {
            if let Some(parent) = self.base.parent_handle() {
                let strength = self.base.strength();
                parent.with_mut(|parent| {
                    if let Some(paint) = parent.as_shape_paint_mut() {
                        paint.with_render_paint_mut(|paint| paint.feather(strength));
                    }
                });
            }
        }
        if has_dirt(value, ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH) && self.is_inner()
        {
            let paint = self.base.parent_handle().expect("Feather parent paint");
            let (container, kind) = paint
                .with(|paint| {
                    (
                        paint.component_parent_handle(),
                        paint
                            .as_shape_paint_behavior()
                            .expect("Feather ShapePaint parent")
                            .pick_path_kind(),
                    )
                })
                .expect("live Feather paint");
            if let Some(container) = container {
                let offset_in_artboard = self.space() == TransformSpace::World;
                let result = container
                    .with_mut(|container| {
                        let Some(transform) = container.shape_paint_world_transform() else {
                            return None;
                        };
                        Some(container.with_shape_paint_path_mut(kind, &mut |path| {
                            self.rebuild_inner_path(path, &transform, offset_in_artboard);
                        }))
                    })
                    .flatten();
                match result {
                    Some(true) => self.effect_path_dirty = true,
                    Some(false) => return,
                    None => {}
                }
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
            offset = Vec2D::transform_dir(offset, &shape_transform.invert_or_identity());
        }
        let mut inner_path = self.inner_path.borrow_mut();
        inner_path.rewind();
        inner_path.add_rect(bounds, PathDirection::Clockwise);
        let transform = Mat2D::from_translation(offset);
        inner_path.add_path_backwards(path.raw_path(), Some(&transform));
    }
    pub fn build_dependencies(&mut self) {
        let (Some(this), Some(paint)) = (self.base.handle(), self.base.parent_handle()) else {
            return;
        };
        let container = paint
            .with(|paint| paint.component_parent_handle())
            .flatten();
        if let Some(container) = container {
            container.with_mut(|container| {
                if let Some(shape) = container.as_shape_mut() {
                    shape.path_composer_mut().add_dependent(this);
                } else if let Some(component) = container.as_component_mut() {
                    component.add_dependent(this);
                }
            });
        }
    }
    pub fn strength_changed(&mut self) {
        let inner = self.base.inner();
        self.base.add_dirt(
            if inner {
                ComponentDirt::PAINT | ComponentDirt::WORLD_TRANSFORM
            } else {
                ComponentDirt::PAINT
            },
            false,
        );
    }
    pub fn offset_x_changed(&mut self) {
        self.strength_changed();
    }
    pub fn offset_y_changed(&mut self) {
        self.strength_changed();
    }
    pub fn inner_path(&self) -> Rc<RefCell<ShapePaintPath>> {
        self.inner_path.clone()
    }
    pub fn effect_path_dirty(&self) -> bool {
        self.effect_path_dirty
    }
    pub fn mark_effect_path_dirty(&mut self) {
        self.effect_path_dirty = true;
    }
}
