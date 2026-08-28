use std::ops::{Deref, DerefMut};

use crate::mechanical_port::source::{
    component::ComponentDirt,
    constraints::constraint::get_parent_world,
    core_context::{CoreContext, StatusCode},
    generated::{
        constraints::follow_path_constraint_base::{
            FollowPathConstraintBase, FollowPathConstraintBaseCallbacks, TransformSpace,
        },
        core_registry::CoreCapabilities,
    },
    math::{
        mat2d::Mat2D, math_types, path_measure::PathMeasure, raw_path::RawPath,
        transform_components::TransformComponents, vec2d::Vec2D,
    },
    shapes::{path::Path, path_flags::PathFlags, shape::Shape},
    transform_component::TransformComponent,
};

#[derive(Default)]
pub struct FollowPathConstraint {
    pub base: FollowPathConstraintBase,
    raw_path: RawPath,
    path_measure: PathMeasure,
}

impl Deref for FollowPathConstraint {
    type Target = FollowPathConstraintBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for FollowPathConstraint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FollowPathConstraint {
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FollowPathConstraintBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut crate::mechanical_port::source::core::binary_reader::BinaryReader<'_>,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }

    pub fn distance_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn orient_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn target_transform(&self, distance_offset: f32) -> Mat2D {
        let target = self.base.target().expect("caller validates target");
        target
            .with(|target| {
                let transform = target
                    .as_transform_component()
                    .expect("validated FollowPathConstraint target");
                if target.as_shape().is_some() || target.as_path().is_some() {
                    let measured = self.path_measure.at_percentage(distance_offset);
                    let position = measured.pos;
                    let mut transform_b = *transform.world_transform();
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
                    if self.base.offset() {
                        let components = self
                            .base
                            .parent_handle()
                            .and_then(|parent| {
                                parent.with(|parent| {
                                    parent
                                        .as_transform_component()
                                        .map(|parent| *parent.transform())
                                })
                            })
                            .flatten();
                        if let Some(components) = components {
                            offset_position.x = components[4];
                            offset_position.y = components[5];
                        }
                    }
                    transform_b[4] = position.x + offset_position.x;
                    transform_b[5] = position.y + offset_position.y;
                    transform_b
                } else {
                    *transform.world_transform()
                }
            })
            .expect("FollowPathConstraint retains a live target")
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some(target) = self.base.target() else {
            return;
        };
        let target_collapsed = target
            .with(|target| {
                target
                    .as_transform_component()
                    .expect("validated FollowPathConstraint target")
                    .is_collapsed()
            })
            .expect("FollowPathConstraint retains a live target");
        if target_collapsed {
            return;
        }
        let mut transform_b = self.target_transform(self.base.distance());
        let target_parent_world = get_parent_world(component);
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
            let target_parent_world = self
                .base
                .target()
                .and_then(|target| {
                    target.with(|target| target.as_transform_component().map(get_parent_world))
                })
                .flatten()
                .expect("validated FollowPathConstraint target");
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
        let paths = target
            .with(|target| {
                if let Some(shape) = target.as_shape() {
                    shape.paths()
                } else if target.as_path().is_some() {
                    vec![self.base.target().expect("target remains assigned")]
                } else {
                    Vec::new()
                }
            })
            .expect("FollowPathConstraint retains a live target");
        if !paths.is_empty() {
            self.raw_path.rewind();
            for path in paths {
                path.with(|path| {
                    let path = path.as_path().expect("Shape paths remain Path-derived");
                    self.raw_path
                        .add_path(path.raw_path(), Some(path.path_transform()));
                })
                .expect("Shape retains live paths");
            }
            self.path_measure = PathMeasure::new(&self.raw_path);
        }
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        if let Some(target) = self.base.target() {
            target
                .with_mut(|target| {
                    if let Some(shape) = target.as_shape_mut() {
                        shape.add_flags(PathFlags::FOLLOW_PATH);
                    } else if let Some(path) = target.as_path_mut() {
                        path.add_flags(PathFlags::FOLLOW_PATH);
                    }
                })
                .expect("FollowPathConstraint retains a live target");
        }
        self.base.on_added_clean(context)
    }

    pub fn build_dependencies(&mut self) {
        let this = self
            .base
            .handle()
            .expect("arena-owned FollowPathConstraint");
        if let Some(target) = self.base.target() {
            target
                .with_mut(|target| {
                    if let Some(shape) = target.as_shape_mut() {
                        shape.path_composer_mut().add_dependent(this.clone());
                    } else if let Some(path) = target.as_path_mut() {
                        if let Some(shape) = path.shape_handle() {
                            shape
                                .with_mut(|shape| {
                                    shape
                                        .as_shape_mut()
                                        .expect("Path shape remains Shape-derived")
                                        .path_composer_mut()
                                        .add_dependent(this.clone());
                                })
                                .expect("Path retains a live Shape");
                        } else {
                            path.base.add_dependent(this.clone());
                        }
                    }
                })
                .expect("FollowPathConstraint retains a live target");
        }
        if let Some(parent) = self.base.parent_handle() {
            self.base.add_dependent(parent);
        }
    }
}
