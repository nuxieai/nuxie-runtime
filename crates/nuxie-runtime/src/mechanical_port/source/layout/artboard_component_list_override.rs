use crate::mechanical_port::source::{
    artboard::{
        ArtboardInstance, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle,
    },
    artboard_component_list::ArtboardComponentList,
    generated::layout::artboard_component_list_override_base::ArtboardComponentListOverrideBase,
    layout::style_overrider::{StyleOverrideProvider, StyleOverrider},
};

impl std::ops::Deref for ArtboardComponentListOverride {
    type Target = ArtboardComponentListOverrideBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ArtboardComponentListOverride {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ArtboardComponentListOverride {
    pub const TYPE_KEY: u16 = ArtboardComponentListOverrideBase::TYPE_KEY;
}

pub struct ArtboardComponentListOverride {
    pub base: ArtboardComponentListOverrideBase,
    artboards: Vec<RuntimeArtboardInstanceWeakHandle>,
    style_overrider: StyleOverrider<ArtboardComponentListOverride>,
}

impl ArtboardComponentListOverride {
    pub fn new(base: ArtboardComponentListOverrideBase) -> Self {
        Self {
            base,
            artboards: Vec::new(),
            style_overrider: StyleOverrider::detached(),
        }
    }
    pub fn add_artboard(&mut self, artboard: &RuntimeArtboardInstanceHandle) {
        self.artboards.push(artboard.downgrade());
        StyleOverrider::<Self>::update_width_override(self, artboard);
        StyleOverrider::<Self>::update_height_override(self, artboard);
    }
    pub fn remove_artboard(&mut self, artboard: &RuntimeArtboardInstanceHandle) {
        let target = artboard.downgrade();
        self.artboards.retain(|item| !item.ptr_eq(&target));
    }
    pub fn is_row(&self) -> bool {
        self.base
            .parent_handle()
            .and_then(|parent| {
                parent.with_downcast::<ArtboardComponentList, _>(|list| list.main_axis_is_row())
            })
            .unwrap_or(true)
    }
    pub fn is_stack(&self) -> bool {
        self.base
            .parent_handle()
            .and_then(|parent| {
                parent.with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::is_stack)
            })
            .unwrap_or(false)
    }
    fn update_width_override(&mut self) {
        for artboard in self.artboards.clone() {
            if let Some(artboard) = artboard.upgrade() {
                StyleOverrider::<Self>::update_width_override(self, &artboard);
            }
        }
    }
    fn update_height_override(&mut self) {
        for artboard in self.artboards.clone() {
            if let Some(artboard) = artboard.upgrade() {
                StyleOverrider::<Self>::update_height_override(self, &artboard);
            }
        }
    }
    pub fn instance_width_changed(&mut self) {
        self.update_width_override();
    }
    pub fn instance_height_changed(&mut self) {
        self.update_height_override();
    }
    pub fn instance_width_units_value_changed(&mut self) {
        self.update_width_override();
    }
    pub fn instance_height_units_value_changed(&mut self) {
        self.update_height_override();
    }
    pub fn instance_width_scale_type_changed(&mut self) {
        self.update_width_override();
    }
    pub fn instance_height_scale_type_changed(&mut self) {
        self.update_height_override();
    }
    pub fn mark_hosting_layout_dirty(&mut self, artboard_instance: &RuntimeArtboardInstanceHandle) {
        if let Some(artboard) = self.base.artboard_handle() {
            crate::mechanical_port::source::artboard::Artboard::mark_layout_dirty_occurrence(
                &artboard,
                artboard_instance.core_handle(),
                None,
            );
            crate::mechanical_port::source::layout_component::LayoutComponent::mark_layout_style_dirty_occurrence(&artboard);
        }
    }
    pub fn actual_instance_width(&mut self, artboard: &ArtboardInstance) -> f32 {
        StyleOverrider::<Self>::actual_instance_width(self, artboard)
    }
    pub fn actual_instance_height(&mut self, artboard: &ArtboardInstance) -> f32 {
        StyleOverrider::<Self>::actual_instance_height(self, artboard)
    }
}

impl Default for ArtboardComponentListOverride {
    fn default() -> Self {
        Self::new(ArtboardComponentListOverrideBase::default())
    }
}

impl StyleOverrideProvider for ArtboardComponentListOverride {
    fn is_row(&self) -> bool {
        ArtboardComponentListOverride::is_row(self)
    }
    fn is_stack(&self) -> bool {
        ArtboardComponentListOverride::is_stack(self)
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
        ArtboardComponentListOverride::mark_hosting_layout_dirty(self, artboard);
    }
    fn borrowed_artboard_host(
        &mut self,
    ) -> Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost> {
        None
    }
}
