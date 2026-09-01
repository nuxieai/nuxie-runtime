use crate::mechanical_port::source::{
    constraints::constraint::{Constraint, get_parent_world},
    core::CoreObject,
    generated::{
        constraints::transform_constraint_base::TransformConstraintBase,
        core_registry::CoreCapabilities,
    },
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents},
    transform_component::TransformComponent,
    transform_space::TransformSpace,
};

#[derive(Default)]
pub struct TransformConstraint {
    pub base: TransformConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
}

impl TransformConstraint {
    fn target_transform_for(&self, target: &dyn CoreObject) -> Mat2D {
        let bounds = target
            .transform_component_constraint_bounds()
            .expect("validated targeted constraint target");
        let target = target
            .as_transform_component()
            .expect("validated targeted constraint target");
        let local = Mat2D::from_translate(
            bounds.left() + bounds.width() * self.base.origin_x(),
            bounds.top() + bounds.height() * self.base.origin_y(),
        );
        *target.world_transform() * local
    }

    pub fn constrain(&mut self, component: &mut dyn CoreObject) {
        let Some(target) = self.base.target() else {
            return;
        };
        let read_target = |target: &dyn CoreObject| {
            let transform = target
                .as_transform_component()
                .expect("validated targeted constraint target");
            (
                transform.is_collapsed(),
                get_parent_world(transform),
                self.target_transform_for(target),
            )
        };
        let (target_collapsed, target_parent_world, mut transform_b) =
            if component.core().handle().as_ref() == Some(&target) {
                read_target(component)
            } else {
                target
                    .with(|target| read_target(target))
                    .expect("TargetedConstraint retains a live target")
            };
        if target_collapsed {
            return;
        }
        let transform = component
            .as_transform_component()
            .expect("constraint TransformComponent");
        let transform_a = *transform.world_transform();
        if self.base.source_space() == TransformSpace::Local {
            let mut inverse = Mat2D::default();
            if !target_parent_world.invert(&mut inverse) {
                return;
            }
            transform_b = inverse * transform_b;
        }
        if self.base.dest_space() == TransformSpace::Local {
            transform_b = get_parent_world(transform) * transform_b;
        }
        let strength = self.base.strength();
        Self::constrain_world(
            component
                .as_transform_component_mut()
                .expect("constraint TransformComponent"),
            transform_a,
            self.components_a,
            transform_b,
            self.components_b,
            strength,
        );
        Constraint::land_anchor(component, strength);
    }

    pub fn origin_x_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }
    pub fn origin_y_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn constrain_world(
        component: &mut TransformComponent,
        from: Mat2D,
        mut components_from: TransformComponents,
        to: Mat2D,
        mut components_to: TransformComponents,
        strength: f32,
    ) {
        components_from = from.decompose();
        components_to = to.decompose();
        let angle_a = components_from.rotation() % (math_types::PI * 2.0);
        let angle_b = components_to.rotation() % (math_types::PI * 2.0);
        let mut diff = angle_b - angle_a;
        if diff > math_types::PI {
            diff -= math_types::PI * 2.0;
        } else if diff < -math_types::PI {
            diff += math_types::PI * 2.0;
        }
        let t = strength;
        let ti = 1.0 - t;
        components_to.set_rotation(angle_a + diff * t);
        components_to.set_x(components_from.x() * ti + components_to.x() * t);
        components_to.set_y(components_from.y() * ti + components_to.y() * t);
        components_to.set_scale_x(components_from.scale_x() * ti + components_to.scale_x() * t);
        components_to.set_scale_y(components_from.scale_y() * ti + components_to.scale_y() * t);
        components_to.set_skew(components_from.skew() * ti + components_to.skew() * t);
        *component.mutable_world_transform() = Mat2D::compose(&components_to);
    }
}
