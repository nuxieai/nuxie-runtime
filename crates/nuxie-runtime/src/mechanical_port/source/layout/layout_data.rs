#[cfg(feature = "rive-layout")]
use crate::mechanical_port::source::{
    layout::layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    yoga::{YGNode, YGStyle},
};

#[cfg(feature = "rive-layout")]
pub struct LayoutData {
    #[cfg(feature = "rive-tools")]
    pub children: std::collections::HashSet<*mut LayoutData>,
    pub node: YGNode,
    pub style: YGStyle,
    pub appliers: Option<Box<Vec<*mut dyn LayoutStyleApplier>>>,
}

#[cfg(feature = "rive-layout")]
impl LayoutData {
    pub fn add_applier(&mut self, applier: *mut dyn LayoutStyleApplier) {
        let appliers = self.appliers.get_or_insert_with(|| Box::new(Vec::new()));
        if !appliers.contains(&applier) {
            appliers.push(applier);
        }
    }
    pub fn apply_layout_styles(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let Some(appliers) = &self.appliers else {
            return;
        };
        if appliers.is_empty() {
            return;
        }
        for applier in appliers.iter().copied() {
            unsafe { (*applier).apply_base_style(style, context) };
        }
        for applier in appliers.iter().copied() {
            unsafe { (*applier).apply_container_style(style, context) };
        }
        for applier in appliers.iter().copied() {
            unsafe { (*applier).apply_item_style(style, context) };
        }
    }
    #[cfg(feature = "rive-tools")]
    pub fn clear_children(&mut self) {
        for child in self.children.drain() {
            unsafe { (*child).unref() };
        }
    }
}

#[cfg(all(feature = "rive-layout", feature = "rive-tools"))]
impl Drop for LayoutData {
    fn drop(&mut self) {
        self.clear_children();
    }
}

#[cfg(feature = "rive-tools")]
pub type LayoutDataRef = crate::mechanical_port::source::refcnt::Rcp<LayoutData>;
#[cfg(not(feature = "rive-tools"))]
pub type LayoutDataRef = *mut LayoutData;
