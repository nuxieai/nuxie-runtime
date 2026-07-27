use std::sync::OnceLock;

use super::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, Mat2D, RuntimeComponent};
use crate::properties::{cached_property_key_for_name, property_key_for_name};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformProperty {
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
}

impl TransformProperty {
    pub(crate) fn property_name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Rotation => "rotation",
            Self::ScaleX => "scaleX",
            Self::ScaleY => "scaleY",
            Self::Opacity => "opacity",
        }
    }

    pub(crate) fn default_value(self) -> f32 {
        match self {
            Self::X | Self::Y | Self::Rotation => 0.0,
            Self::ScaleX | Self::ScaleY | Self::Opacity => 1.0,
        }
    }

    pub(crate) fn property_key_for_type(self, type_name: &str) -> Option<u16> {
        match self {
            Self::X if type_name == "RootBone" => root_bone_x_property_key(),
            Self::Y if type_name == "RootBone" => root_bone_y_property_key(),
            Self::X => node_x_property_key(),
            Self::Y => node_y_property_key(),
            Self::Rotation => transform_component_rotation_property_key(),
            Self::ScaleX => transform_component_scale_x_property_key(),
            Self::ScaleY => transform_component_scale_y_property_key(),
            Self::Opacity if type_name == "Artboard" => artboard_opacity_property_key(),
            Self::Opacity => transform_component_opacity_property_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransformPropertyKeys {
    type_name: &'static str,
    x: Option<u16>,
    y: Option<u16>,
    rotation: Option<u16>,
    scale_x: Option<u16>,
    scale_y: Option<u16>,
    opacity: Option<u16>,
}

impl TransformPropertyKeys {
    pub(crate) fn for_type(type_name: &'static str) -> Self {
        Self {
            type_name,
            x: property_key_for_name(type_name, TransformProperty::X.property_name()),
            y: property_key_for_name(type_name, TransformProperty::Y.property_name()),
            rotation: property_key_for_name(type_name, TransformProperty::Rotation.property_name()),
            scale_x: property_key_for_name(type_name, TransformProperty::ScaleX.property_name()),
            scale_y: property_key_for_name(type_name, TransformProperty::ScaleY.property_name()),
            opacity: property_key_for_name(type_name, TransformProperty::Opacity.property_name()),
        }
    }

    fn is_for_type(self, type_name: &str) -> bool {
        self.type_name == type_name
    }

    pub(crate) fn key(self, property: TransformProperty) -> Option<u16> {
        match property {
            TransformProperty::X => self.x,
            TransformProperty::Y => self.y,
            TransformProperty::Rotation => self.rotation,
            TransformProperty::ScaleX => self.scale_x,
            TransformProperty::ScaleY => self.scale_y,
            TransformProperty::Opacity => self.opacity,
        }
    }
}

fn node_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "x")
}

fn node_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Node", "y")
}

fn root_bone_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "x")
}

fn root_bone_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "RootBone", "y")
}

fn transform_component_rotation_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "rotation")
}

fn transform_component_scale_x_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "scaleX")
}

fn transform_component_scale_y_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "scaleY")
}

fn transform_component_opacity_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "TransformComponent", "opacity")
}

fn artboard_opacity_property_key() -> Option<u16> {
    static KEY: OnceLock<Option<u16>> = OnceLock::new();
    cached_property_key_for_name(&KEY, "Artboard", "opacity")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AuthoredTransform {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformRuntimeState {
    pub local_transform: Mat2D,
    pub world_transform: Mat2D,
    pub render_opacity: f32,
}

impl Default for TransformRuntimeState {
    fn default() -> Self {
        Self {
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }
}

impl RuntimeComponent {
    pub(crate) fn transform_property_key(&self, property: TransformProperty) -> Option<u16> {
        if self.transform_property_keys.is_for_type(self.type_name) {
            self.transform_property_keys.key(property)
        } else {
            TransformPropertyKeys::for_type(self.type_name).key(property)
        }
    }

    pub(crate) fn update_transform(&mut self, authored: AuthoredTransform) {
        if !self.capabilities.transform {
            return;
        }

        let mut transform = Mat2D::from_rotation(authored.rotation);
        transform.0[4] = authored.x;
        transform.0[5] = authored.y;
        transform.scale_by_values(authored.scale_x, authored.scale_y);
        self.transform.local_transform = transform;
    }

    pub(crate) fn update_world_transform(&mut self, parent_world: Option<Mat2D>) {
        if self.type_name == "Artboard" || !self.capabilities.transform {
            return;
        }

        self.transform.world_transform = match parent_world {
            Some(parent_world) => parent_world.multiply(self.transform.local_transform),
            None => self.transform.local_transform,
        };
        if let Some(node) = self.concrete.node.as_ref() {
            node.mark_computed_local_dirty();
        }
    }

    pub(crate) fn update_render_opacity(&mut self, opacity: f32, parent_opacity: f32) {
        if !self.capabilities.transform {
            return;
        }

        self.transform.render_opacity = opacity * parent_opacity;
    }
}

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
        let legacy_scale_changed =
            self.mark_legacy_image_layout_scale_written(local_id, property_key);
        if !object_changed && !legacy_scale_changed {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);

        match property {
            TransformProperty::Opacity => {
                self.mark_world_transform_opacity_dirty(local_id);
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
    pub(super) fn mark_transform_dirty_handle(&mut self, handle: ComponentHandle) -> bool {
        if !self.add_component_dirt(handle, ComponentDirt::TRANSFORM, false) {
            return false;
        }
        self.add_component_dirt(handle, ComponentDirt::WORLD_TRANSFORM, true);
        true
    }
}
