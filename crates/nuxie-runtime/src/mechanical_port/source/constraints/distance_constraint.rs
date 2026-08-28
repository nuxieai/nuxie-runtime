use crate::mechanical_port::source::{
    constraints::constraint::Constraint,
    generated::{
        constraints::distance_constraint_base::DistanceConstraintBase,
        core_registry::CoreCapabilities,
    },
    math::vec2d::Vec2D,
    transform_component::TransformComponent,
};

#[repr(u32)]
enum Mode {
    Closer = 0,
    Further = 1,
    Exact = 2,
}

#[derive(Default)]
pub struct DistanceConstraint {
    pub base: DistanceConstraintBase,
}

impl DistanceConstraint {
    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some(target) = self.base.target() else {
            return;
        };
        let (target_collapsed, target_translation) = target
            .with(|target| {
                let target = target
                    .as_transform_component()
                    .expect("validated DistanceConstraint target");
                (target.is_collapsed(), target.world_translation())
            })
            .expect("DistanceConstraint retains a live target");
        if target_collapsed {
            return;
        }
        let our_translation = component.world_translation();
        let mut to_target = our_translation - target_translation;
        let current_distance = to_target.length();

        match self.base.mode_value() {
            x if x == Mode::Closer as u32 => {
                if current_distance < self.base.distance() {
                    return;
                }
            }
            x if x == Mode::Further as u32 => {
                if current_distance > self.base.distance() {
                    return;
                }
            }
            _ => {}
        }
        if current_distance < 0.001_f32 {
            return;
        }

        to_target *= self.base.distance() / current_distance;
        let mut position = target_translation + to_target;
        position = Vec2D::lerp(our_translation, position, self.base.strength());
        let world = component.mutable_world_transform();
        world[4] = position.x;
        world[5] = position.y;
    }

    pub fn distance_changed(&mut self) {
        self.mark_constraint_dirty();
    }

    pub fn mode_value_changed(&mut self) {
        self.mark_constraint_dirty();
    }
}
