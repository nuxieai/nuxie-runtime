use crate::animation::{RuntimeLinearAnimation, RuntimeLinearAnimationHandle};
use crate::artboard::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, Mat2D};
use crate::objects::{InstanceObjectArena, ObjectHandle};
use crate::properties::property_key_for_name;
use nuxie_graph::ArtboardGraph;

pub(crate) const JOYSTICK_FLAG_INVERT_X: u64 = 1 << 0;
pub(crate) const JOYSTICK_FLAG_INVERT_Y: u64 = 1 << 1;

/// Occurrence-owned counterpart of pinned C++ `Joystick`.
///
/// The component/source fields are handles into one Artboard occurrence, not
/// serialized local ids. Animation definitions are immutable arena handles;
/// nested remaps use a private occurrence-local object handle because they are
/// Core objects rather than Components.
#[derive(Debug)]
pub(crate) struct RuntimeJoystick {
    component: ComponentHandle,
    world_transform: Mat2D,
    inverse_world_transform: Mat2D,
    x_animation: Option<RuntimeLinearAnimationHandle>,
    y_animation: Option<RuntimeLinearAnimationHandle>,
    handle_source: Option<ComponentHandle>,
    dependents: Vec<ObjectHandle>,
}

impl Clone for RuntimeJoystick {
    fn clone(&self) -> Self {
        // Core clone copies generated fields, then reruns Joystick lifecycle
        // against clone-owned Components. Component slot indices are stable
        // within the new occurrence, while derived matrices start cold
        // (`artboard.hpp:548-601`; `src/joystick.cpp:8-47`).
        Self {
            component: self.component,
            world_transform: Mat2D::IDENTITY,
            inverse_world_transform: Mat2D::IDENTITY,
            x_animation: self.x_animation,
            y_animation: self.y_animation,
            handle_source: self.handle_source,
            dependents: self.dependents.clone(),
        }
    }
}

impl RuntimeJoystick {
    #[cfg(test)]
    pub(crate) fn test_fixture(component: ComponentHandle, can_apply_before_update: bool) -> Self {
        Self {
            component,
            world_transform: Mat2D::IDENTITY,
            inverse_world_transform: Mat2D::IDENTITY,
            x_animation: None,
            y_animation: None,
            handle_source: (!can_apply_before_update).then_some(component),
            dependents: Vec::new(),
        }
    }

    pub(crate) fn component(&self) -> ComponentHandle {
        self.component
    }

    pub(crate) fn can_apply_before_update(&self) -> bool {
        self.handle_source.is_none()
    }

    pub(crate) fn x_animation_index(&self) -> Option<usize> {
        self.x_animation
            .and_then(RuntimeLinearAnimationHandle::definition_index)
    }

    pub(crate) fn y_animation_index(&self) -> Option<usize> {
        self.y_animation
            .and_then(RuntimeLinearAnimationHandle::definition_index)
    }

    pub(crate) fn dependent_count(&self) -> usize {
        self.dependents.len()
    }

    pub(crate) fn dependent_local(&self, index: usize) -> Option<usize> {
        self.dependents.get(index).map(|handle| handle.local_id())
    }
}

pub(crate) fn build_runtime_joysticks(
    graph: &ArtboardGraph,
    objects: &InstanceObjectArena,
    linear_animations: &[RuntimeLinearAnimation],
) -> Vec<RuntimeJoystick> {
    graph
        .joysticks
        .iter()
        .filter_map(|joystick| {
            let component = objects.component_handle(joystick.local_id)?;
            let handle_source = joystick
                .handle_source_local
                .and_then(|local| objects.component_handle(local))
                .filter(|handle| {
                    objects
                        .component(*handle)
                        .is_some_and(|component| component.capabilities.world_transform)
                });
            let animation_handle = |global_id: Option<u32>| {
                global_id
                    .and_then(|global_id| {
                        linear_animations
                            .iter()
                            .position(|animation| animation.global_id == global_id)
                    })
                    .map(RuntimeLinearAnimationHandle::new)
            };
            Some(RuntimeJoystick {
                component,
                world_transform: Mat2D::IDENTITY,
                inverse_world_transform: Mat2D::IDENTITY,
                x_animation: animation_handle(joystick.x_animation_global),
                y_animation: animation_handle(joystick.y_animation_global),
                handle_source,
                // C++ appends y-animation remaps before x-animation remaps and
                // preserves duplicates. The graph retains that exact order.
                dependents: joystick
                    .nested_remap_dependents
                    .iter()
                    .filter_map(|dependent| objects.object_handle(dependent.local_id))
                    .collect(),
            })
        })
        .collect()
}

pub(crate) fn joystick_x_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "x")
}

pub(crate) fn joystick_y_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "y")
}

pub(crate) fn joystick_flags_property_key() -> Option<u16> {
    property_key_for_name("Joystick", "joystickFlags")
}

fn joystick_property_key(name: &str) -> Option<u16> {
    property_key_for_name("Joystick", name)
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("Joystick") {
        return None;
    }
    let is_generated_callback = ["x", "y", "posX", "posY", "width", "height"]
        .into_iter()
        .any(|name| joystick_property_key(name) == Some(property_key));
    if !is_generated_callback {
        return None;
    }

    // Literal generated callback body: all six setters publish only root
    // Components dirt (`src/joystick.cpp:131-142`). In particular, do not use
    // the old prepared-frame epoch as a substitute for this owner callback.
    let _ = local_id;
    Some(artboard.mark_components_dirty())
}

impl ArtboardInstance {
    #[doc(hidden)]
    pub fn debug_joystick_flags(&self, local_id: usize) -> Option<u64> {
        let key = joystick_flags_property_key()?;
        self.uint_property(local_id, key)
    }

    pub(crate) fn update_runtime_joystick(
        &mut self,
        component: ComponentHandle,
        dirt: ComponentDirt,
    ) {
        let transform_dirt = ComponentDirt::WORLD_TRANSFORM | ComponentDirt::TRANSFORM;
        if dirt.0 & transform_dirt.0 == 0 {
            return;
        }
        let Some(joystick_index) = self
            .joysticks
            .iter()
            .position(|joystick| joystick.component() == component)
        else {
            return;
        };
        let Some(handle_source) = self.joysticks[joystick_index].handle_source else {
            return;
        };
        let local_id = self.component_at(component).local_id;
        let value = |artboard: &Self, name: &str, default: f32| {
            joystick_property_key(name)
                .and_then(|key| artboard.double_property(local_id, key))
                .unwrap_or(default)
        };
        let pos_x = value(self, "posX", 0.0);
        let pos_y = value(self, "posY", 0.0);
        let width = value(self, "width", 0.0);
        let height = value(self, "height", 0.0);
        let origin_x = value(self, "originX", 0.5);
        let origin_y = value(self, "originY", 0.5);

        let mut world = Mat2D([1.0, 0.0, 0.0, 1.0, pos_x, pos_y]);
        if let Some(parent) = self.component_parent_handle(component)
            && self.component_at(parent).capabilities.world_transform
        {
            world = self
                .component_at(parent)
                .transform
                .world_transform
                .multiply(world);
        }
        if self.joysticks[joystick_index].world_transform != world {
            self.joysticks[joystick_index].world_transform = world;
            self.joysticks[joystick_index].inverse_world_transform = world.invert_or_identity();
        }

        let source_world = self.component_at(handle_source).transform.world_transform;
        let local = self.joysticks[joystick_index]
            .inverse_world_transform
            .transform_point(source_world.0[4], source_world.0[5]);
        let (x, y) = joystick_factor_from(local, width, height, origin_x, origin_y);
        if let Some(key) = joystick_x_property_key() {
            self.set_double_property(local_id, key, x);
        }
        if let Some(key) = joystick_y_property_key() {
            self.set_double_property(local_id, key, y);
        }
    }

    /// Direct `Joystick::apply` owner. Artboard retains the occurrence list
    /// and invokes this by index so applying an animation can mutably settle
    /// the same Artboard without cloning joystick state.
    pub(crate) fn apply_runtime_joystick_at(&mut self, joystick_index: usize) -> bool {
        let Some(joystick) = self.joysticks.get(joystick_index) else {
            return false;
        };
        let Some(local_id) = self.component_local_id(joystick.component()) else {
            return false;
        };
        let x_animation_index = joystick.x_animation_index();
        let y_animation_index = joystick.y_animation_index();
        let nested_remap_dependents_len = joystick.dependent_count();

        let mut changed = false;
        if let Some(animation_index) = x_animation_index
            && let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, true)
        {
            changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
        }
        if let Some(animation_index) = y_animation_index
            && let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, false)
        {
            changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
        }
        for dependent_index in 0..nested_remap_dependents_len {
            if let Some(remap_local_id) =
                self.joysticks[joystick_index].dependent_local(dependent_index)
            {
                changed |= self.advance_nested_remap_animation(remap_local_id);
            }
        }
        changed
    }

    fn joystick_axis_seconds(
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

    /// Direct `Joystick::controlSize` owner.
    pub(crate) fn control_runtime_joystick_size(
        &mut self,
        local_id: usize,
        width: f32,
        height: f32,
    ) -> bool {
        let value = |artboard: &Self, name: &str, default: f32| {
            joystick_property_key(name)
                .and_then(|key| artboard.double_property(local_id, key))
                .unwrap_or(default)
        };
        let origin_x = value(self, "originX", 0.5);
        let origin_y = value(self, "originY", 0.5);
        let writes = [
            ("width", width),
            ("height", height),
            ("posX", width * origin_x),
            ("posY", height * origin_y),
        ];
        writes.into_iter().fold(false, |changed, (name, value)| {
            joystick_property_key(name)
                .is_some_and(|key| self.set_double_property(local_id, key, value))
                | changed
        })
    }
}

/// Direct `Joystick::measureLayout` owner. Spell `std::min(maximum, authored)`
/// as its comparison rather than `f32::min`, which normalizes NaNs and does
/// not preserve the pinned first-argument signed-zero behavior.
pub(crate) fn measure_joystick_layout(
    artboard: &ArtboardInstance,
    component: ComponentHandle,
    maximum_width: f32,
    maximum_height: f32,
) -> Option<(f32, f32)> {
    let local_id = artboard.component_local_id(component)?;
    let authored = |name: &str| {
        joystick_property_key(name)
            .and_then(|key| artboard.double_property(local_id, key))
            .unwrap_or(0.0)
    };
    Some((
        cpp_minimum(maximum_width, authored("width")),
        cpp_minimum(maximum_height, authored("height")),
    ))
}

fn cpp_minimum(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}

fn joystick_factor_from(
    local: (f32, f32),
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
) -> (f32, f32) {
    let left = -width * origin_x;
    let top = -height * origin_y;
    let x = if width == 0.0 {
        0.0
    } else {
        (local.0 - left) * 2.0 / width - 1.0
    };
    // Preserve the intentionally asymmetric pinned AABB::factorFrom grouping:
    // only the y delta is guarded, so zero height proceeds through 0 / 0.
    let y = (if height == 0.0 { 0.0 } else { local.1 - top }) * 2.0 / height - 1.0;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::{cpp_minimum, joystick_factor_from};

    #[test]
    fn factor_from_preserves_pinned_zero_extent_asymmetry() {
        let (x, y) = joystick_factor_from((3.0, 4.0), 0.0, 0.0, 0.5, 0.5);
        assert_eq!(x, 0.0);
        assert!(y.is_nan());
    }

    #[test]
    fn measure_minimum_preserves_cpp_nan_and_signed_zero_order() {
        assert!(cpp_minimum(f32::NAN, 1.0).is_nan());
        assert_eq!(cpp_minimum(1.0, f32::NAN), 1.0);
        assert_eq!(cpp_minimum(0.0, -0.0).to_bits(), 0.0f32.to_bits());
    }
}
