use super::super::ArtboardInstance;
use crate::animation::RuntimeKeyedCallback;
use crate::constraints::set_runtime_scroll_double_property;
use crate::properties::property_key_for_name;

impl ArtboardInstance {
    pub(crate) fn set_keyed_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_generated_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    /// C++ keyed animations retain a concrete Core pointer, so a known
    /// `SolidColor::colorValue` write does not rediscover its type or property
    /// on every frame. Keep the same observer and invalidation effects as the
    /// generic color setter while skipping branches that cannot apply to a
    /// SolidColor target (text, view-model, gradient, and layout topology).
    pub(crate) fn set_keyed_solid_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        data_bind_observed: bool,
        value: u32,
    ) -> bool {
        let Some(previous) = self.objects.replace_solid_color_value(local_id, value) else {
            return false;
        };
        // Generated C++ setters return before the property callback when the
        // stored value is unchanged (`solid_color_base.hpp:38-46`). Active
        // animations may apply the same keyed value every frame; do not
        // rebuild or reconfigure the retained ShapePaint owner in that case.
        if previous == value {
            return false;
        }
        if data_bind_observed {
            self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        }
        // `SolidColor::renderOpacityChanged()` mutates the retained
        // RenderPaint and calls only `Artboard::changed()`; it does not dirty
        // component/path preparation (`solid_color.cpp:23-54`).
        self.did_change.set(true);
        // Pinned C++ `SolidColor::colorValueChanged` immediately calls
        // `renderOpacityChanged` and mutates the ShapePaint-owned paint
        // (`solid_color.cpp:23-54`). It does not dirty or reconstruct the
        // ShapePaint owner.
        self.settle_runtime_solid_color_callback(local_id, value);
        if let Some(revision) = self.solid_color_paint_revisions.get_mut(local_id) {
            *revision = revision.wrapping_add(1);
        }
        self.mark_prepared_changed_for_solid_color_visibility(Some(previous), value);
        true
    }

    pub(crate) fn set_keyed_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedNumber")
            && property_key_for_name("NestedNumber", "nestedValue") == Some(property_key)
        {
            return self.set_nested_number_value(local_id, value);
        }
        if let Some(changed) =
            set_runtime_scroll_double_property(self, local_id, property_key, value)
        {
            if !changed {
                return false;
            }
            let _ = self
                .objects
                .set_generated_double_property(local_id, property_key, value);
            return self.after_double_property_set(local_id, property_key, value);
        }
        if self.runtime_images.has_public_scale(local_id, property_key)
            && self.double_property(local_id, property_key) == Some(value)
        {
            return false;
        }
        let object_changed =
            self.objects
                .set_generated_double_property(local_id, property_key, value);
        let image_scale_changed = self
            .runtime_images
            .mark_public_scale_written(local_id, property_key);
        if !object_changed && !image_scale_changed {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    pub(crate) fn apply_keyed_callback(&mut self, callback: RuntimeKeyedCallback) -> bool {
        let _seconds_delay = callback.seconds_delay;
        match self
            .slot(callback.target_local_id)
            .and_then(|slot| slot.type_name)
        {
            Some("CustomPropertyTrigger")
                if property_key_for_name("CustomPropertyTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("CustomPropertyTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    + 1;
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            Some("ViewModelInstanceTrigger")
                if property_key_for_name("ViewModelInstanceTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("ViewModelInstanceTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    .wrapping_add(1);
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            _ => false,
        }
    }
}
