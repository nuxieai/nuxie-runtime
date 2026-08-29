use crate::mechanical_port::source::{
    artboard::{Artboard, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle},
    artboard_host::ArtboardHost,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    data_bind::data_context::RuntimeDataContextHandle,
    file::RuntimeFileWeakHandle,
    generated::nested_artboard_layout_base::{
        NestedArtboardLayoutBase, NestedArtboardLayoutBaseCallbacks,
    },
    layout::{
        layout_enums::{LayoutDirection, LayoutStyleInterpolation},
        layout_node_provider::{LayoutNodeProvider, LayoutNodeProviderState},
        style_overrider::{StyleOverrideProvider, StyleOverrider},
    },
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    status_code::StatusCode,
};

pub struct NestedArtboardLayout {
    pub base: NestedArtboardLayoutBase,
    provider_state: LayoutNodeProviderState,
}

impl Default for NestedArtboardLayout {
    fn default() -> Self {
        Self {
            base: NestedArtboardLayoutBase::default(),
            provider_state: LayoutNodeProviderState::default(),
        }
    }
}

impl NestedArtboardLayout {
    pub fn layout_node(
        &self,
        _index: i32,
    ) -> Option<crate::mechanical_port::source::layout::layout_node_provider::LayoutNodeKey> {
        let artboard = self.base.base.artboard_instance_handle(0)?;
        artboard.with_artboard_mut(|artboard| {
            artboard.take_layout_data();
            artboard.layout_node_key(0)
        })
    }

    pub fn clone_layout(&self) -> Self {
        let mut nested = NestedArtboardLayoutBase::clone_into(self);
        nested.base.base.set_file(self.base.base.file());
        // Upstream instances the current reference, not necessarily its
        // original authored definition.
        let referenced = match self.base.base.artboard_instance_handle(0) {
            Some(instance) => Some(instance.core_handle()),
            None => self.base.base.source_artboard(),
        };
        if let Some(referenced) = referenced {
            if let Some(instance) = Artboard::nested_instance_from_handle(&referenced) {
                nested.base.base.referenced_artboard_instance(instance);
            }
        }
        nested
    }

    fn mark_hosting_layout_dirty_instance(&mut self, instance: &RuntimeArtboardInstanceHandle) {
        if let Some(artboard) = self.base.base.parent_artboard_handle() {
            Artboard::mark_layout_dirty_occurrence(&artboard, instance.core_handle(), None);
            crate::mechanical_port::source::layout_component::LayoutComponent::mark_layout_style_dirty_occurrence(&artboard);
        }
    }

    pub fn mark_layout_node_dirty(&mut self, _force: bool) {
        self.update_width_override();
        self.update_height_override();
    }

    pub(crate) fn update_after_nested_artboard_super(&mut self, value: ComponentDirt) {
        if !value.contains(ComponentDirt::WORLD_TRANSFORM) {
            return;
        }
        let Some(instance) = self.base.base.artboard_instance_handle(0) else {
            return;
        };
        let layout_position =
            instance.with_artboard(|instance| Vec2D::new(instance.layout_x(), instance.layout_y()));
        let mut world = *self.base.base.mutable_world_transform();
        let parent_origin = self
            .base
            .base
            .parent_handle()
            .and_then(|parent| parent.with_downcast::<Artboard, _>(Artboard::origin));
        if let Some(origin) = parent_origin {
            world = Mat2D::from_translation(origin + layout_position) * world;
        } else {
            world = Mat2D::from_translation(layout_position) * world;
        }
        let origin = instance.with_artboard(|instance| instance.origin());
        *self.base.base.mutable_world_transform() = Mat2D::from_translation(-origin) * world;
    }

    pub(crate) fn layout_constraint_handles(&self) -> Vec<CoreHandle> {
        self.provider_state.layout_constraints().to_vec()
    }

    pub(crate) fn on_added_clean_after_animation_initialization(
        &mut self,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let code = self
            .base
            .base
            .on_added_clean_after_animation_initialization(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.update_width_override();
        self.update_height_override();
        StatusCode::Ok
    }

    pub fn sync_style_changes(&mut self) -> bool {
        self.base
            .base
            .artboard_instance_handle(0)
            .is_some_and(|instance| instance.sync_style_changes())
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        if let Some(instance) = self.base.base.artboard_instance_handle(0) {
            instance.with_artboard_mut(|instance| instance.update_layout_bounds(animate));
        }
    }

    pub fn cascade_layout_style(
        &mut self,
        inherited_interpolation: LayoutStyleInterpolation,
        inherited_interpolator: Option<CoreHandle>,
        inherited_interpolation_time: f32,
        direction: LayoutDirection,
    ) -> bool {
        if let Some(instance) = self.base.base.artboard_instance_handle(0) {
            instance.with_artboard_mut(|instance| {
                instance.cascade_layout_style(
                    inherited_interpolation,
                    inherited_interpolator,
                    inherited_interpolation_time,
                    direction,
                )
            });
        }
        false
    }

    pub fn layout_bounds(&self) -> Aabb {
        self.base
            .base
            .artboard_instance_handle(0)
            .map_or_else(Aabb::default, |instance| {
                instance.with_artboard(|instance| instance.layout_bounds())
            })
    }

    pub fn is_layout_provider(&self) -> bool {
        true
    }

    pub fn update_artboard(&mut self, value: Option<CoreHandle>) {
        if let Some(parent) = self.base.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(layout) = parent.as_layout_component_mut() {
                    layout.clear_layout_children();
                }
            });
        }
        self.base.base.update_artboard(value);
        self.update_width_override();
        self.update_height_override();
        if let Some(parent) = self.base.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(layout) = parent.as_layout_component_mut() {
                    layout.sync_layout_children();
                }
            });
        }
    }

    pub(crate) fn update_artboard_occurrence(owner: &CoreHandle, value: Option<CoreHandle>) {
        let parent = owner
            .with(|owner| {
                owner
                    .as_component()
                    .expect("NestedArtboardLayout component")
                    .parent_handle()
            })
            .expect("live NestedArtboardLayout");
        if let Some(parent) = parent.as_ref() {
            parent.with_mut(|parent| {
                if let Some(layout) = parent.as_layout_component_mut() {
                    layout.clear_layout_children();
                }
            });
        }
        crate::mechanical_port::source::nested_artboard::NestedArtboard::update_artboard_occurrence(
            owner, value,
        );
        owner.with_downcast_mut::<Self, _>(|owner| owner.update_width_override());
        owner.with_downcast_mut::<Self, _>(|owner| owner.update_height_override());
        let parent = owner
            .with(|owner| {
                owner
                    .as_component()
                    .expect("NestedArtboardLayout component")
                    .parent_handle()
            })
            .expect("live NestedArtboardLayout");
        if let Some(parent) = parent.filter(|parent| parent.is_type_of(
            crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
        )) {
            crate::mechanical_port::source::layout_component::LayoutComponent::sync_layout_children_occurrence(&parent);
        }
    }

    pub fn is_row(&self) -> bool {
        self.base
            .base
            .parent_handle()
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_layout_component()
                            .map(|layout| layout.main_axis_is_row())
                    })
                    .flatten()
            })
            .unwrap_or(true)
    }

    fn update_width_override(&mut self) {
        if let Some(instance) = self.base.base.artboard_instance_handle(0) {
            StyleOverrider::<NestedArtboardLayout>::update_width_override(self, &instance);
        }
    }

    fn update_height_override(&mut self) {
        if let Some(instance) = self.base.base.artboard_instance_handle(0) {
            StyleOverrider::<NestedArtboardLayout>::update_height_override(self, &instance);
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
        NestedArtboardLayout::is_row(self)
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
    fn mark_hosting_layout_dirty(&mut self, artboard: &RuntimeArtboardInstanceHandle) {
        self.mark_hosting_layout_dirty_instance(artboard);
    }
    fn borrowed_artboard_host(&mut self) -> Option<&mut dyn ArtboardHost> {
        Some(self)
    }
}

impl LayoutNodeProvider for NestedArtboardLayout {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider_state
    }
    fn provider_handle(&self) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
    }
    fn owner_handle(&self) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        self.provider_handle()
    }
    fn layout_bounds(&self) -> Aabb {
        NestedArtboardLayout::layout_bounds(self)
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
    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        NestedArtboardLayout::cascade_layout_style(
            self,
            interpolation,
            interpolator,
            time,
            direction,
        )
    }
}

impl ArtboardHost for NestedArtboardLayout {
    fn data_bind_path_referencer(
        &self,
    ) -> &crate::mechanical_port::source::data_bind_path_referencer::DataBindPathReferencer {
        &self.base.base.data_bind_path_referencer
    }

    fn artboard_count(&self) -> usize {
        self.base.base.artboard_count()
    }

    fn artboard_instance(
        &self,
        index: i32,
    ) -> Option<crate::mechanical_port::source::artboard::RuntimeArtboardInstanceHandle> {
        self.base.base.artboard_instance_handle(index)
    }

    fn internal_data_context(&mut self, data_context: RuntimeDataContextHandle) {
        self.base.base.internal_data_context(Some(data_context));
    }

    fn bind_view_model_instance(
        &mut self,
        view_model_instance: CoreHandle,
        parent: RuntimeDataContextHandle,
    ) {
        self.base
            .base
            .bind_view_model_instance(Some(view_model_instance), Some(parent));
    }

    fn clear_data_context(&mut self) {
        self.base.base.clear_data_context();
    }

    fn unbind(&mut self) {
        self.base.base.unbind();
    }

    fn update_data_binds(&mut self) {
        self.base.base.update_data_binds();
    }

    fn mark_hosting_layout_dirty(&mut self, _artboard_instance: RuntimeArtboardInstanceWeakHandle) {
        if let Some(instance) = self.base.base.artboard_instance_handle(0) {
            self.mark_hosting_layout_dirty_instance(&instance);
        }
    }

    fn parent_artboard(&self) -> Option<CoreHandle> {
        self.base.base.parent_artboard_handle()
    }

    fn hit_test_host(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> bool {
        self.base
            .base
            .hit_test_host(position, skip_on_unclipped, artboard)
    }

    fn host_transform_point(
        &self,
        position: &Vec2D,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Vec2D {
        self.base.base.host_transform_point(position, artboard)
    }

    fn world_transform_for_artboard(&self, artboard: RuntimeArtboardInstanceWeakHandle) -> Mat2D {
        self.base.base.world_transform_for_artboard(artboard)
    }

    fn mark_host_transform_dirty(&mut self) {
        self.base.base.mark_host_transform_dirty();
    }

    fn is_layout_provider(&self) -> bool {
        true
    }

    fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        self.base.base.set_file(value.unwrap_or_default());
    }

    fn file(&self) -> Option<RuntimeFileWeakHandle> {
        let file = self.base.base.file();
        file.upgrade().map(|_| file)
    }

    fn host_component(&self) -> Option<CoreHandle> {
        crate::mechanical_port::source::core::CoreObject::core(self).handle()
    }

    fn relink_data_context(&mut self, view_model_instance: Option<CoreHandle>) {
        self.base.base.relink_data_context(view_model_instance);
    }

    fn type_(&self) -> i32 {
        self.base.base.type_()
    }
}

impl std::ops::Deref for NestedArtboardLayout {
    type Target = NestedArtboardLayoutBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedArtboardLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
