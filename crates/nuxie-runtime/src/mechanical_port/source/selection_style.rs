use crate::mechanical_port::source::generated::selection_style_base::{
    SelectionStyleBase, SelectionStyleBaseCallbacks,
};

/// File-owned selection styling shared by the targets in a selection context.
#[derive(Default)]
pub struct SelectionStyle {
    pub base: SelectionStyleBase,
}
impl SelectionStyle {
    pub fn set_highlight_color(&mut self, value: i32) {
        if self.base.set_highlight_color_value(value) {
            self.highlight_color_changed();
            self.base
                .base
                .notify_property_changed(SelectionStyleBase::HIGHLIGHT_COLOR_PROPERTY_KEY);
        }
    }
    pub fn set_corner_radius(&mut self, value: f32) {
        if self.base.set_corner_radius_value(value) {
            self.corner_radius_changed();
            self.base
                .base
                .notify_property_changed(SelectionStyleBase::CORNER_RADIUS_PROPERTY_KEY);
        }
    }
}
impl SelectionStyleBaseCallbacks for SelectionStyle {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}
impl std::ops::Deref for SelectionStyle {
    type Target = SelectionStyleBase;
    fn deref(&self) -> &SelectionStyleBase {
        &self.base
    }
}
impl std::ops::DerefMut for SelectionStyle {
    fn deref_mut(&mut self) -> &mut SelectionStyleBase {
        &mut self.base
    }
}
