use crate::mechanical_port::source::{
    constraints::constraint::get_parent_world,
    generated::{
        constraints::rotation_constraint_base::{RotationConstraintBase, TransformSpace},
        core_registry::CoreCapabilities,
    },
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents},
    transform_component::TransformComponent,
};

#[derive(Default)]
pub struct RotationConstraint {
    pub base: RotationConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
}

impl RotationConstraint {
    pub fn requires_target(&self) -> bool {
        false
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let target_state = self.base.target().map(|target| {
            target
                .with(|target| {
                    let target = target
                        .as_transform_component()
                        .expect("validated targeted constraint target");
                    (
                        target.is_collapsed(),
                        *target.world_transform(),
                        get_parent_world(target),
                    )
                })
                .expect("TargetedConstraint retains a live target")
        });
        if target_state.is_some_and(|target| target.0) {
            return;
        }
        let transform_a = *component.world_transform();
        let mut transform_b;
        self.components_a = transform_a.decompose();
        if target_state.is_none() {
            transform_b = transform_a;
            self.components_b = self.components_a;
        } else {
            let (_, target_world, target_parent_world) = target_state.unwrap();
            transform_b = target_world;
            if self.base.source_space() == TransformSpace::Local {
                let Some(inverse) = target_parent_world.inverted() else {
                    return;
                };
                transform_b = inverse * transform_b;
            }
            self.components_b = transform_b.decompose();
            if !self.base.does_copy() {
                self.components_b.set_rotation(
                    if self.base.dest_space() == TransformSpace::Local {
                        0.0
                    } else {
                        self.components_a.rotation()
                    },
                );
            } else {
                self.components_b
                    .set_rotation(self.components_b.rotation() * self.base.copy_factor());
                if self.base.offset() {
                    self.components_b
                        .set_rotation(self.components_b.rotation() + component.rotation());
                }
            }
            if self.base.dest_space() == TransformSpace::Local {
                transform_b = Mat2D::compose(self.components_b);
                transform_b = get_parent_world(component) * transform_b;
                self.components_b = transform_b.decompose();
            }
        }
        let clamp_local = self.base.min_max_space() == TransformSpace::Local;
        if clamp_local {
            transform_b = Mat2D::compose(self.components_b);
            let Some(inverse) = get_parent_world(component).inverted() else {
                return;
            };
            self.components_b = (inverse * transform_b).decompose();
        }
        if self.base.max() && self.components_b.rotation() > self.base.max_value() {
            self.components_b.set_rotation(self.base.max_value());
        }
        if self.base.min() && self.components_b.rotation() < self.base.min_value() {
            self.components_b.set_rotation(self.base.min_value());
        }
        if clamp_local {
            transform_b = Mat2D::compose(self.components_b);
            transform_b = get_parent_world(component) * transform_b;
            self.components_b = transform_b.decompose();
        }
        let angle_a = self.components_a.rotation() % (math_types::PI * 2.0);
        let angle_b = self.components_b.rotation() % (math_types::PI * 2.0);
        let mut diff = angle_b - angle_a;
        if diff > math_types::PI {
            diff -= math_types::PI * 2.0;
        } else if diff < -math_types::PI {
            diff += math_types::PI * 2.0;
        }
        self.components_b
            .set_rotation(self.components_a.rotation() + diff * self.base.strength());
        self.components_b.set_x(self.components_a.x());
        self.components_b.set_y(self.components_a.y());
        self.components_b.set_scale_x(self.components_a.scale_x());
        self.components_b.set_scale_y(self.components_a.scale_y());
        self.components_b.set_skew(self.components_a.skew());
        *component.mutable_world_transform() = Mat2D::compose(self.components_b);
    }
}
