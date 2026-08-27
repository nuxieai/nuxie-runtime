use crate::mechanical_port::source::{
    artboard::{Artboard, ArtboardInstance},
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    generated::nested_artboard_layout_base::{
        NestedArtboardLayoutBase, NestedArtboardLayoutBaseCallbacks,
    },
    layout::{
        layout_node_provider::{LayoutNodeProvider, LayoutNodeProviderState},
        style_overrider::{StyleOverrideProvider, StyleOverrider},
    },
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    status_code::StatusCode,
    transform_component::TransformComponent,
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};

pub struct NestedArtboardLayout {
    pub base: NestedArtboardLayoutBase,
    provider_state: LayoutNodeProviderState,
    style_overrider: StyleOverrider<NestedArtboardLayout>,
}

impl Default for NestedArtboardLayout {
    fn default() -> Self {
        Self {
            base: NestedArtboardLayoutBase::default(),
            provider_state: LayoutNodeProviderState::default(),
            style_overrider: StyleOverrider::detached(),
        }
    }
}

struct CloneCallbacks;
impl NestedArtboardLayoutBaseCallbacks for CloneCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

impl NestedArtboardLayout {
    fn attach_style_overrider(&mut self) {
        let this = self as *mut Self;
        self.style_overrider.attach(unsafe { &mut *this });
    }

    pub fn clone_layout(&self) -> Self {
        let mut nested = self.base.clone_into(&mut CloneCallbacks);
        nested.base.base.set_file(self.base.base.file());
        if let Some(referenced) = self.base.base.referenced_artboard() {
            nested
                .base
                .base
                .set_referenced_artboard(referenced.instance());
        }
        nested
    }

    pub fn mark_hosting_layout_dirty(&mut self, _instance: &mut ArtboardInstance) {
        if let Some(artboard) = self.base.base.artboard_mut() {
            let hosted = self
                .base
                .base
                .artboard_instance_mut()
                .map(|value| value as *mut _);
            artboard.mark_layout_dirty(hosted);
            artboard.mark_layout_style_dirty();
        }
    }

    pub fn mark_layout_node_dirty(&mut self, _force: bool) {
        self.update_width_override();
        self.update_height_override();
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        if !value.contains(ComponentDirt::WORLD_TRANSFORM) {
            return;
        }
        let Some(instance) = self.base.base.artboard_instance_mut() else {
            return;
        };
        let layout_position = Vec2D::new(instance.layout_x(), instance.layout_y());
        let mut world = *self.base.base.mutable_world_transform();
        if let Some(parent_artboard) = self
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_artboard_mut())
        {
            world = Mat2D::from_translation(parent_artboard.origin() + layout_position) * world;
        } else {
            world = Mat2D::from_translation(layout_position) * world;
        }
        *self.base.base.mutable_world_transform() =
            Mat2D::from_translation(-instance.origin()) * world;
    }

    pub fn update_constraints(&mut self) {
        let constraints = self.provider_state.layout_constraints().to_vec();
        for constraint in constraints {
            unsafe { &mut *constraint }.constrain_child(self);
        }
        self.base.base.update_constraints();
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.attach_style_overrider();
        self.update_width_override();
        self.update_height_override();
        StatusCode::Ok
    }

    pub fn sync_style_changes(&mut self) -> bool {
        self.base
            .base
            .referenced_artboard_mut()
            .is_some_and(Artboard::sync_style_changes)
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        #[cfg(feature = "rive_layout")]
        if let Some(instance) = self.base.base.artboard_instance_mut() {
            instance.update_layout_bounds(animate);
        }
        #[cfg(not(feature = "rive_layout"))]
        let _ = animate;
    }

    pub fn layout_bounds(&mut self) -> Aabb {
        #[cfg(feature = "rive_layout")]
        if let Some(instance) = self.base.base.artboard_instance_mut() {
            return instance.layout_bounds();
        }
        Aabb::default()
    }

    pub fn update_artboard(&mut self, value: &mut ViewModelInstanceArtboard) {
        #[cfg(feature = "rive_layout")]
        if let Some(layout) = self
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
        {
            layout.clear_layout_children();
        }
        self.base.base.update_artboard(value);
        self.update_width_override();
        self.update_height_override();
        #[cfg(feature = "rive_layout")]
        if let Some(layout) = self
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
        {
            layout.sync_layout_children();
        }
    }

    pub fn is_row(&mut self) -> bool {
        self.base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_layout_component_mut())
            .is_none_or(|layout| layout.main_axis_is_row())
    }

    fn update_width_override(&mut self) {
        let instance = self
            .base
            .base
            .artboard_instance_mut()
            .map(|value| value as *mut ArtboardInstance);
        if let Some(instance) = instance {
            self.attach_style_overrider();
            self.style_overrider
                .update_width_override(unsafe { &mut *instance });
        }
    }

    fn update_height_override(&mut self) {
        let instance = self
            .base
            .base
            .artboard_instance_mut()
            .map(|value| value as *mut ArtboardInstance);
        if let Some(instance) = instance {
            self.attach_style_overrider();
            self.style_overrider
                .update_height_override(unsafe { &mut *instance });
        }
    }
}

impl NestedArtboardLayoutBaseCallbacks for NestedArtboardLayout {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
    fn instance_width_changed(&mut self) {
        self.update_width_override();
    }
    fn instance_height_changed(&mut self) {
        self.update_height_override();
    }
    fn instance_width_units_value_changed(&mut self) {
        self.update_width_override();
    }
    fn instance_height_units_value_changed(&mut self) {
        self.update_height_override();
    }
    fn instance_width_scale_type_changed(&mut self) {
        self.update_width_override();
    }
    fn instance_height_scale_type_changed(&mut self) {
        self.update_height_override();
    }
}

impl StyleOverrideProvider for NestedArtboardLayout {
    fn is_row(&self) -> bool {
        true
    }
    fn instance_height_scale_type(&self) -> u32 {
        self.base.instance_height_scale_type()
    }
    fn instance_width_scale_type(&self) -> u32 {
        self.base.instance_width_scale_type()
    }
    fn instance_height_units_value(&self) -> u32 {
        self.base.instance_height_units_value()
    }
    fn instance_width_units_value(&self) -> u32 {
        self.base.instance_width_units_value()
    }
    fn instance_height(&self) -> f32 {
        self.base.instance_height()
    }
    fn instance_width(&self) -> f32 {
        self.base.instance_width()
    }
    fn mark_hosting_layout_dirty(&mut self, artboard: &mut ArtboardInstance) {
        NestedArtboardLayout::mark_hosting_layout_dirty(self, artboard);
    }
}

impl LayoutNodeProvider for NestedArtboardLayout {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider_state
    }
    fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        Some(self.base.base.transform_component_mut())
    }
    fn transform_component(&self) -> Option<&TransformComponent> {
        Some(self.base.base.transform_component())
    }
    fn layout_bounds(&self) -> Aabb {
        Aabb::default()
    }
    fn sync_style_changes(&mut self) -> bool {
        NestedArtboardLayout::sync_style_changes(self)
    }
    fn update_layout_bounds(&mut self, animate: bool) {
        NestedArtboardLayout::update_layout_bounds(self, animate);
    }
    fn mark_layout_node_dirty(&mut self, force: bool) {
        NestedArtboardLayout::mark_layout_node_dirty(self, force);
    }
    fn num_layout_nodes(&self) -> usize {
        1
    }
}
