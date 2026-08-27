use crate::mechanical_port::source::{
    constraints::constraint::get_parent_world,
    generated::constraints::translation_constraint_base::{TranslationConstraintBase, TransformSpace},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    transform_component::TransformComponent,
};

pub struct TranslationConstraint { pub base: TranslationConstraintBase }

impl TranslationConstraint {
    pub fn requires_target(&self) -> bool { false }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        if self.base.target().is_some_and(|target| target.is_collapsed()) { return; }
        let transform_a = component.mutable_world_transform();
        let translation_a = Vec2D::new(transform_a[4], transform_a[5]);
        let mut translation_b;
        if self.base.target().is_none() {
            translation_b = translation_a;
        } else {
            let target = self.base.target().unwrap();
            let mut transform_b = *target.world_transform();
            if self.base.source_space() == TransformSpace::Local {
                let Some(inverse) = get_parent_world(target).inverted() else { return; };
                transform_b = inverse * transform_b;
            }
            translation_b = transform_b.translation();
            if !self.base.does_copy() {
                translation_b.x = if self.base.dest_space() == TransformSpace::Local { 0.0 } else { translation_a.x };
            } else {
                translation_b.x *= self.base.copy_factor();
                if self.base.offset() { translation_b.x += component.x(); }
            }
            if !self.base.does_copy_y() {
                translation_b.y = if self.base.dest_space() == TransformSpace::Local { 0.0 } else { translation_a.y };
            } else {
                translation_b.y *= self.base.copy_factor_y();
                if self.base.offset() { translation_b.y += component.y(); }
            }
            if self.base.dest_space() == TransformSpace::Local {
                translation_b = *get_parent_world(component) * translation_b;
            }
        }
        let clamp_local = self.base.min_max_space() == TransformSpace::Local;
        if clamp_local {
            let Some(inverse) = get_parent_world(component).inverted() else { return; };
            translation_b = inverse * translation_b;
        }
        if self.base.max() && translation_b.x > self.base.max_value() { translation_b.x = self.base.max_value(); }
        if self.base.min() && translation_b.x < self.base.min_value() { translation_b.x = self.base.min_value(); }
        if self.base.max_y() && translation_b.y > self.base.max_value_y() { translation_b.y = self.base.max_value_y(); }
        if self.base.min_y() && translation_b.y < self.base.min_value_y() { translation_b.y = self.base.min_value_y(); }
        if clamp_local { translation_b = *get_parent_world(component) * translation_b; }
        let t = self.base.strength();
        let ti = 1.0 - t;
        transform_a[4] = translation_a.x * ti + translation_b.x * t;
        transform_a[5] = translation_a.y * ti + translation_b.y * t;
    }
}
