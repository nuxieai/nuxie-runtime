use crate::mechanical_port::source::{
    artboard::ArtboardInstance,
    artboard_component_list::ArtboardComponentList,
    generated::layout::artboard_component_list_override_base::ArtboardComponentListOverrideBase,
    layout::style_overrider::{StyleOverrideProvider, StyleOverrider},
};

pub struct ArtboardComponentListOverride {
    pub base: ArtboardComponentListOverrideBase,
    artboards: Vec<*mut ArtboardInstance>,
    style_overrider: StyleOverrider<ArtboardComponentListOverride>,
}

impl ArtboardComponentListOverride {
    pub fn new(base: ArtboardComponentListOverrideBase) -> Self {
        let mut value = Self {
            base,
            artboards: Vec::new(),
            style_overrider: StyleOverrider::detached(),
        };
        value.style_overrider.attach(&mut value);
        value
    }
    pub fn add_artboard(&mut self, artboard: &mut ArtboardInstance) {
        self.artboards.push(artboard);
        self.style_overrider.update_width_override(artboard);
        self.style_overrider.update_height_override(artboard);
    }
    pub fn remove_artboard(&mut self, artboard: &mut ArtboardInstance) {
        let pointer = artboard as *mut _;
        self.artboards.retain(|item| *item != pointer);
    }
    pub fn is_row(&self) -> bool {
        self.base
            .parent()
            .as_ref::<ArtboardComponentList>()
            .map_or(true, |list| list.main_axis_is_row())
    }
    fn update_width_override(&mut self) {
        for artboard in self.artboards.iter().copied() {
            unsafe { self.style_overrider.update_width_override(&mut *artboard) };
        }
    }
    fn update_height_override(&mut self) {
        for artboard in self.artboards.iter().copied() {
            unsafe { self.style_overrider.update_height_override(&mut *artboard) };
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
    pub fn mark_hosting_layout_dirty(&mut self, artboard_instance: &mut ArtboardInstance) {
        if let Some(artboard) = self.base.artboard_mut() {
            artboard.mark_layout_dirty(artboard_instance);
            artboard.mark_layout_style_dirty();
        }
    }
    pub fn actual_instance_width(&mut self, artboard: &ArtboardInstance) -> f32 {
        self.style_overrider.actual_instance_width(artboard)
    }
    pub fn actual_instance_height(&mut self, artboard: &ArtboardInstance) -> f32 {
        self.style_overrider.actual_instance_height(artboard)
    }
}

impl StyleOverrideProvider for ArtboardComponentListOverride {
    fn is_row(&self) -> bool {
        ArtboardComponentListOverride::is_row(self)
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
        ArtboardComponentListOverride::mark_hosting_layout_dirty(self, artboard);
    }
}
