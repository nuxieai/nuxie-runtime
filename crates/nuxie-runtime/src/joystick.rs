use nuxie_graph::ArtboardGraph;

use crate::animation::RuntimeLinearAnimation;
use crate::artboard::ArtboardInstance;
use crate::properties::{
    JOYSTICK_FLAG_INVERT_X, JOYSTICK_FLAG_INVERT_Y, joystick_flags_property_key,
    joystick_x_property_key, joystick_y_property_key,
};

/// Occurrence-owned retained fields for C++ `Joystick`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeJoystick {
    pub(crate) local_id: usize,
    pub(crate) can_apply_before_update: bool,
    pub(crate) x_animation_index: Option<usize>,
    pub(crate) y_animation_index: Option<usize>,
    pub(crate) nested_remap_dependents: Vec<usize>,
}

pub(crate) fn build_runtime_joysticks(
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
) -> Vec<RuntimeJoystick> {
    graph
        .joysticks
        .iter()
        .map(|joystick| RuntimeJoystick {
            local_id: joystick.local_id,
            can_apply_before_update: joystick.can_apply_before_update,
            x_animation_index: joystick.x_animation_global.and_then(|global_id| {
                linear_animations
                    .iter()
                    .position(|animation| animation.global_id == global_id)
            }),
            y_animation_index: joystick.y_animation_global.and_then(|global_id| {
                linear_animations
                    .iter()
                    .position(|animation| animation.global_id == global_id)
            }),
            nested_remap_dependents: joystick
                .nested_remap_dependents
                .iter()
                .map(|dependent| dependent.local_id)
                .collect(),
        })
        .collect()
}

impl ArtboardInstance {
    pub(crate) fn apply_joysticks(&mut self, can_apply_before_update: bool) -> bool {
        let mut changed = false;
        let joystick_count = self.joysticks.len();
        for joystick_index in 0..joystick_count {
            if self.joysticks[joystick_index].can_apply_before_update == can_apply_before_update {
                changed |= self.apply_joystick_at(joystick_index);
            }
        }
        changed
    }

    pub(crate) fn apply_joystick_at(&mut self, joystick_index: usize) -> bool {
        // Mirrors C++ Artboard::updatePass / Joystick::apply: iterate retained
        // joystick entries instead of cloning the joystick list per pass.
        let Some(joystick) = self.joysticks.get(joystick_index) else {
            return false;
        };
        let local_id = joystick.local_id;
        let x_animation_index = joystick.x_animation_index;
        let y_animation_index = joystick.y_animation_index;
        let nested_remap_dependents_len = joystick.nested_remap_dependents.len();

        let mut changed = false;
        if let Some(animation_index) = x_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, true) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        if let Some(animation_index) = y_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, false) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        for dependent_index in 0..nested_remap_dependents_len {
            let remap_local_id =
                self.joysticks[joystick_index].nested_remap_dependents[dependent_index];
            changed |= self.advance_nested_remap_animation(remap_local_id);
        }
        changed
    }

    pub(crate) fn joystick_axis_seconds(
        &self,
        local_id: usize,
        animation_index: usize,
        is_x_axis: bool,
    ) -> Option<f32> {
        let animation = self.linear_animation(animation_index)?;
        let axis_key = if is_x_axis {
            joystick_x_property_key()
        } else {
            joystick_y_property_key()
        }?;
        let flag = if is_x_axis {
            JOYSTICK_FLAG_INVERT_X
        } else {
            JOYSTICK_FLAG_INVERT_Y
        };
        let mut axis = self.double_property(local_id, axis_key).unwrap_or(0.0);
        let flags = joystick_flags_property_key()
            .and_then(|key| self.uint_property(local_id, key))
            .unwrap_or(0);
        if flags & flag != 0 {
            axis = -axis;
        }
        Some(((axis + 1.0) / 2.0) * animation.duration_seconds())
    }
}
