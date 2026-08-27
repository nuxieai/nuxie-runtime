use crate::mechanical_port::source::{
    constraints::constraint::get_parent_world,
    generated::constraints::scale_constraint_base::{ScaleConstraintBase, TransformSpace},
    math::{mat2d::Mat2D, transform_components::TransformComponents},
    transform_component::TransformComponent,
};

pub struct ScaleConstraint {
    pub base: ScaleConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
}

impl ScaleConstraint {
    pub fn requires_target(&self) -> bool {
        false
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        if self
            .base
            .target()
            .is_some_and(|target| target.is_collapsed())
        {
            return;
        }
        let transform_a = *component.world_transform();
        let mut transform_b;
        self.components_a = transform_a.decompose();
        if self.base.target().is_none() {
            transform_b = transform_a;
            self.components_b = self.components_a;
        } else {
            let target = self.base.target().unwrap();
            transform_b = *target.world_transform();
            if self.base.source_space() == TransformSpace::Local {
                let Some(inverse) = get_parent_world(target).inverted() else {
                    return;
                };
                transform_b = inverse * transform_b;
            }
            self.components_b = transform_b.decompose();
            if !self.base.does_copy() {
                self.components_b
                    .set_scale_x(if self.base.dest_space() == TransformSpace::Local {
                        1.0
                    } else {
                        self.components_a.scale_x()
                    });
            } else {
                self.components_b
                    .set_scale_x(self.components_b.scale_x() * self.base.copy_factor());
                if self.base.offset() {
                    self.components_b
                        .set_scale_x(self.components_b.scale_x() * component.scale_x());
                }
            }
            if !self.base.does_copy_y() {
                self.components_b
                    .set_scale_y(if self.base.dest_space() == TransformSpace::Local {
                        1.0
                    } else {
                        self.components_a.scale_y()
                    });
            } else {
                self.components_b
                    .set_scale_y(self.components_b.scale_y() * self.base.copy_factor_y());
                if self.base.offset() {
                    self.components_b
                        .set_scale_y(self.components_b.scale_y() * component.scale_y());
                }
            }
            if self.base.dest_space() == TransformSpace::Local {
                transform_b = Mat2D::compose(self.components_b);
                transform_b = *get_parent_world(component) * transform_b;
                self.components_b = transform_b.decompose();
            }
        }
        let clamp_local = self.base.min_max_space() == TransformSpace::Local;
        if clamp_local {
            transform_b = Mat2D::compose(self.components_b);
            let Some(inverse) = get_parent_world(component).inverted() else {
                return;
            };
            transform_b = inverse * transform_b;
            self.components_b = transform_b.decompose();
        }
        if self.base.max() && self.components_b.scale_x() > self.base.max_value() {
            self.components_b.set_scale_x(self.base.max_value());
        }
        if self.base.min() && self.components_b.scale_x() < self.base.min_value() {
            self.components_b.set_scale_x(self.base.min_value());
        }
        if self.base.max_y() && self.components_b.scale_y() > self.base.max_value_y() {
            self.components_b.set_scale_y(self.base.max_value_y());
        }
        if self.base.min_y() && self.components_b.scale_y() < self.base.min_value_y() {
            self.components_b.set_scale_y(self.base.min_value_y());
        }
        if clamp_local {
            transform_b = Mat2D::compose(self.components_b);
            transform_b = *get_parent_world(component) * transform_b;
            self.components_b = transform_b.decompose();
        }
        let t = self.base.strength();
        let ti = 1.0 - t;
        self.components_b.set_rotation(self.components_a.rotation());
        self.components_b.set_x(self.components_a.x());
        self.components_b.set_y(self.components_a.y());
        self.components_b
            .set_scale_x(self.components_a.scale_x() * ti + self.components_b.scale_x() * t);
        self.components_b
            .set_scale_y(self.components_a.scale_y() * ti + self.components_b.scale_y() * t);
        self.components_b.set_skew(self.components_a.skew());
        *component.mutable_world_transform() = Mat2D::compose(self.components_b);
    }
}
