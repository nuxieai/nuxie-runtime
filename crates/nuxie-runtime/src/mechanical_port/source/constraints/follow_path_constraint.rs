use crate::mechanical_port::source::{
    component::ComponentDirt,
    constraints::constraint::get_parent_world,
    core_context::{CoreContext, StatusCode},
    generated::constraints::follow_path_constraint_base::{
        FollowPathConstraintBase, TransformSpace,
    },
    math::{
        mat2d::Mat2D, math_types, path_measure::PathMeasure, raw_path::RawPath,
        transform_components::TransformComponents, vec2d::Vec2D,
    },
    shapes::{path::Path, path_flags::PathFlags, shape::Shape},
    transform_component::TransformComponent,
};

pub struct FollowPathConstraint {
    pub base: FollowPathConstraintBase,
    raw_path: RawPath,
    path_measure: PathMeasure,
}

impl FollowPathConstraint {
    pub fn distance_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn orient_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn target_transform(&self, distance_offset: f32) -> Mat2D {
        let target = self.base.target().expect("caller validates target");
        if target.is::<Shape>() || target.is::<Path>() {
            let measured = self.path_measure.at_percentage(distance_offset);
            let position = measured.pos;
            let mut transform_b = *target.world_transform();
            if self.base.orient() {
                let components_b = transform_b.decompose();
                let tangent_rotation = measured.tan.y.atan2(measured.tan.x);
                let angle_b = components_b.rotation() % (math_types::PI * 2.0);
                let mut diff = tangent_rotation - angle_b;
                if diff > math_types::PI {
                    diff -= math_types::PI * 2.0;
                } else if diff < -math_types::PI {
                    diff += math_types::PI * 2.0;
                }
                transform_b = Mat2D::from_rotation(angle_b + diff * self.base.strength());
            }
            let mut offset_position = Vec2D::default();
            if self.base.offset() && self.base.parent().is::<TransformComponent>() {
                let components = self
                    .base
                    .parent()
                    .as_ref::<TransformComponent>()
                    .expect("type checked above")
                    .transform();
                offset_position.x = components[4];
                offset_position.y = components[5];
            }
            transform_b[4] = position.x + offset_position.x;
            transform_b[5] = position.y + offset_position.y;
            transform_b
        } else {
            *target.world_transform()
        }
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some(target) = self.base.target() else {
            return;
        };
        if target.is_collapsed() {
            return;
        }
        let mut transform_b = self.target_transform(self.base.distance());
        let target_parent_world = *get_parent_world(component);
        let components = self.constrain_helper(
            component.world_transform(),
            &mut transform_b,
            &target_parent_world,
        );
        *component.mutable_world_transform() = Mat2D::compose(components);
    }

    pub fn constrain_helper(
        &self,
        component_transform: &Mat2D,
        transform_b: &mut Mat2D,
        component_parent_world: &Mat2D,
    ) -> TransformComponents {
        let transform_a = component_transform;
        if self.base.source_space() == TransformSpace::Local {
            let target_parent_world = get_parent_world(self.base.target().unwrap());
            let Some(inverse) = target_parent_world.inverted() else {
                return TransformComponents::default();
            };
            *transform_b = inverse * *transform_b;
        }
        if self.base.dest_space() == TransformSpace::Local {
            *transform_b = *component_parent_world * *transform_b;
        }
        let components_a = transform_a.decompose();
        let mut components_b = transform_b.decompose();
        let t = self.base.strength();
        let ti = 1.0 - t;
        if !self.base.orient() {
            components_b.set_rotation(components_a.rotation() % (math_types::PI * 2.0));
        }
        components_b.set_x(components_a.x() * ti + components_b.x() * t);
        components_b.set_y(components_a.y() * ti + components_b.y() * t);
        components_b.set_scale_x(components_a.scale_x());
        components_b.set_scale_y(components_a.scale_y());
        components_b.set_skew(components_a.skew());
        components_b
    }

    pub fn update(&mut self, _value: ComponentDirt) {
        let target = self.base.target().expect("added constraint has target");
        let mut paths: Vec<&Path> = Vec::new();
        if let Some(shape) = target.as_ref::<Shape>() {
            paths.extend(shape.paths());
        } else if let Some(path) = target.as_ref::<Path>() {
            paths.push(path);
        }
        if !paths.is_empty() {
            self.raw_path.rewind();
            for path in paths {
                self.raw_path
                    .add_path(path.raw_path(), Some(path.path_transform()));
            }
            self.path_measure = PathMeasure::new(&self.raw_path);
        }
    }

    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        if let Some(target) = self.base.target_mut() {
            if let Some(shape) = target.as_mut::<Shape>() {
                shape.add_flags(PathFlags::FOLLOW_PATH);
            } else if let Some(path) = target.as_mut::<Path>() {
                path.add_flags(PathFlags::FOLLOW_PATH);
            }
        }
        self.base.on_added_clean(context)
    }

    pub fn build_dependencies(&mut self) {
        let this = self.base.as_component_mut_ptr();
        if let Some(shape) = self.base.target_mut().and_then(|t| t.as_mut::<Shape>()) {
            shape.path_composer_mut().add_dependent(this);
        } else if let Some(path) = self.base.target_mut().and_then(|t| t.as_mut::<Path>()) {
            if let Some(shape) = path.shape_mut() {
                shape.path_composer_mut().add_dependent(this);
            } else {
                path.add_dependent(this);
            }
        }
        self.base.add_dependent(self.base.parent_mut());
    }
}
