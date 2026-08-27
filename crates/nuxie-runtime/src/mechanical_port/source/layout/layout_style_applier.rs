#[derive(Default, Clone, Copy)]
pub struct LayoutSyncContext {
    pub parent_is_grid: bool,
    pub parent_is_stack: bool,
    pub container_justify_items: u32,
    pub inline_hugs: bool,
    pub parent_is_row: bool,
    pub is_ltr: bool,
    pub has_layout_parent: bool,
}

pub trait LayoutStyleApplier {
    #[cfg(feature = "rive-layout")]
    fn apply_base_style(
        &self,
        _style: &mut crate::mechanical_port::source::yoga::YGStyle,
        _context: &LayoutSyncContext,
    ) {
    }
    #[cfg(feature = "rive-layout")]
    fn apply_container_style(
        &self,
        _style: &mut crate::mechanical_port::source::yoga::YGStyle,
        _context: &LayoutSyncContext,
    ) {
    }
    #[cfg(feature = "rive-layout")]
    fn apply_item_style(
        &self,
        _style: &mut crate::mechanical_port::source::yoga::YGStyle,
        _context: &LayoutSyncContext,
    ) {
    }
}
