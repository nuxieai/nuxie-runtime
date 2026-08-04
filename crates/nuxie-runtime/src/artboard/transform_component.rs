use super::ArtboardInstance;
use crate::components::{AuthoredTransform, ComponentDirt, ComponentHandle, TransformProperty};

impl ArtboardInstance {
    pub fn set_transform_property(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        value: f32,
    ) -> bool {
        let Some(component) = self.component(local_id) else {
            return false;
        };
        let property_key = component.transform_property_key(property);
        let Some(property_key) = property_key else {
            return false;
        };
        self.set_transform_property_with_key(local_id, property, property_key, value)
    }

    pub(crate) fn set_transform_property_with_key(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
        value: f32,
    ) -> bool {
        let Some(component) = self.component(local_id) else {
            return false;
        };
        if !component.capabilities.transform {
            return false;
        }

        let Some(current) = self.transform_property_with_key(local_id, property, property_key)
        else {
            return false;
        };
        if current == value {
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
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);

        match property {
            TransformProperty::Opacity => {
                self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
            }
            TransformProperty::X
            | TransformProperty::Y
            | TransformProperty::Rotation
            | TransformProperty::ScaleX
            | TransformProperty::ScaleY => {
                let handle = self
                    .component_handle(local_id)
                    .expect("validated Transform has a Component handle");
                self.mark_transform_dirty_handle(handle);
            }
        }
        true
    }

    pub fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        let component = self
            .component(local_id)
            .filter(|component| component.capabilities.transform)?;
        let property_key = component.transform_property_key(property)?;
        self.transform_property_with_key(local_id, property, property_key)
    }

    pub(crate) fn transform_property_with_key(
        &self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
    ) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)?;
        Some(
            self.double_property(local_id, property_key)
                .unwrap_or_else(|| property.default_value()),
        )
    }

    pub(crate) fn authored_transform(&self, local_id: usize) -> AuthoredTransform {
        let component = self.component(local_id);
        let (x, y) = if component
            .and_then(|component| component.concrete.bone.as_ref())
            .is_some_and(|bone| !bone.is_root)
        {
            let parent_length = self
                .component_handle(local_id)
                .and_then(|handle| self.objects.component(handle))
                .and_then(|component| component.parent)
                .and_then(|parent| self.objects.component_local_id(parent))
                .and_then(|parent_local| self.bone_length(parent_local))
                .unwrap_or(0.0);
            (parent_length, 0.0)
        } else {
            (
                self.transform_property(local_id, TransformProperty::X)
                    .unwrap_or_else(|| TransformProperty::X.default_value()),
                self.transform_property(local_id, TransformProperty::Y)
                    .unwrap_or_else(|| TransformProperty::Y.default_value()),
            )
        };

        AuthoredTransform {
            x,
            y,
            rotation: self
                .transform_property(local_id, TransformProperty::Rotation)
                .unwrap_or_else(|| TransformProperty::Rotation.default_value()),
            scale_x: self
                .transform_property(local_id, TransformProperty::ScaleX)
                .unwrap_or_else(|| TransformProperty::ScaleX.default_value()),
            scale_y: self
                .transform_property(local_id, TransformProperty::ScaleY)
                .unwrap_or_else(|| TransformProperty::ScaleY.default_value()),
            opacity: self
                .transform_property(local_id, TransformProperty::Opacity)
                .unwrap_or_else(|| TransformProperty::Opacity.default_value()),
        }
    }

    /// Literal `TransformComponent::markTransformDirty`: recursive World dirt
    /// is gated by the transition that newly adds Transform dirt. Repeating a
    /// transform setter while Transform is already pending must not re-dirty a
    /// clean dependent subtree (`src/transform_component.cpp:54-61`).
    pub(in crate::artboard) fn mark_transform_dirty_handle(
        &mut self,
        handle: ComponentHandle,
    ) -> bool {
        if !self.add_component_dirt(handle, ComponentDirt::TRANSFORM, false) {
            return false;
        }
        self.add_component_dirt(handle, ComponentDirt::WORLD_TRANSFORM, true);
        true
    }
}
