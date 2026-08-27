use crate::mechanical_port::source::{
    generated::layout::layout_node_style_base::LayoutNodeStyleBase, layout::layout_node_provider,
};

pub struct LayoutNodeStyle {
    pub base: LayoutNodeStyleBase,
}
impl LayoutNodeStyle {
    pub fn mark_layout_node_dirty(&mut self) {
        if let Some(provider) = layout_node_provider::from(self.base.parent_mut()) {
            provider.mark_layout_node_dirty(false);
        }
    }
    pub fn width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn fractional_width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn fractional_height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn layout_width_scale_type_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn layout_height_scale_type_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn justify_self_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn display_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
}
