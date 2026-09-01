use crate::mechanical_port::source::{
    constraints::constraint::Constraint,
    core::CoreObject,
    generated::{
        constraints::distance_constraint_base::DistanceConstraintBase,
        core_registry::CoreCapabilities,
    },
    math::vec2d::Vec2D,
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
    pub fn constrain(&mut self, component: &mut dyn CoreObject) {
        let Some(target) = self.base.target() else {
            return;
        };
        let target_state = |target: &dyn CoreObject| {
            let target = target
                .as_transform_component()
                .expect("validated DistanceConstraint target");
            (target.is_collapsed(), target.world_translation())
        };
        let (target_collapsed, target_translation) =
            if component.core().handle().as_ref() == Some(&target) {
                target_state(component)
            } else {
                target
                    .with(|target| target_state(target))
                    .expect("DistanceConstraint retains a live target")
            };
        if target_collapsed {
            return;
        }
        let anchor = component.transform_component_local_anchor();
        let component = component
            .as_transform_component_mut()
            .expect("constraint TransformComponent");
        let world = component.mutable_world_transform();
        let anchor_world = Vec2D::new(
            world[0] * anchor.x + world[2] * anchor.y,
            world[1] * anchor.x + world[3] * anchor.y,
        );
        let our_translation = Vec2D::new(world[4], world[5]) + anchor_world;
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
        world[4] = position.x - anchor_world.x;
        world[5] = position.y - anchor_world.y;
    }

    pub fn distance_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn mode_value_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }
}
