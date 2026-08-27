use crate::mechanical_port::source::{
    constraints::constraint::get_parent_world,
    generated::constraints::transform_constraint_base::{TransformConstraintBase, TransformSpace},
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents},
    transform_component::TransformComponent,
};

pub struct TransformConstraint {
    pub base: TransformConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
}

impl TransformConstraint {
    pub fn target_transform(&self) -> Mat2D {
        let target = self.base.target().expect("targeted constraint has target");
        let bounds = target.constraint_bounds();
        let local = Mat2D::from_translate(
            bounds.left() + bounds.width() * self.base.origin_x(),
            bounds.top() + bounds.height() * self.base.origin_y(),
        );
        *target.world_transform() * local
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some(target) = self.base.target() else {
            return;
        };
        if target.is_collapsed() {
            return;
        }
        let transform_a = *component.world_transform();
        let mut transform_b = self.target_transform();
        if self.base.source_space() == TransformSpace::Local {
            let Some(inverse) = get_parent_world(target).inverted() else {
                return;
            };
            transform_b = inverse * transform_b;
        }
        if self.base.dest_space() == TransformSpace::Local {
            transform_b = *get_parent_world(component) * transform_b;
        }
        Self::constrain_world(
            component,
            transform_a,
            self.components_a,
            transform_b,
            self.components_b,
            self.base.strength(),
        );
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
        *component.mutable_world_transform() = Mat2D::compose(components_to);
    }
}
