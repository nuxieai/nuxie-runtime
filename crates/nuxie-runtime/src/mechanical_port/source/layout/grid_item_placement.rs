#[cfg(feature = "rive-layout")]
use crate::mechanical_port::source::yoga::YGStyle;
use crate::mechanical_port::source::{
    component::{ComponentDirt, ContainerComponent},
    core_context::{CoreContext, StatusCode},
    generated::layout::grid_item_placement_base::GridItemPlacementBase,
    layout::{
        grid_track::GridTrack,
        layout_node_provider,
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
};

pub struct GridItemPlacement {
    pub base: GridItemPlacementBase,
}
impl GridItemPlacement {
    pub fn from(owner: Option<&ContainerComponent>) -> Option<&GridItemPlacement> {
        for child in owner?.children() {
            if let Some(placement) = child.as_ref::<GridItemPlacement>() {
                return Some(placement);
            }
        }
        None
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        #[cfg(feature = "rive-layout")]
        if let Some(provider) = layout_node_provider::from(self.base.parent_mut()) {
            provider.add_layout_style_applier(self);
        }
        StatusCode::Ok
    }
    pub fn update(&mut self, _value: ComponentDirt) {}
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let Some(parent) = self.base.parent_mut() {
            parent.add_dependent(self.base.as_component_mut_ptr());
        }
    }
    #[cfg(feature = "rive-layout")]
    pub fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        if context.parent_is_stack || !context.parent_is_grid {
            return;
        }
        GridTrack::sync_item_lines(
            style,
            self.base.grid_column(),
            self.base.grid_row(),
            self.base.grid_column_span(),
            self.base.grid_row_span(),
        );
    }
    fn mark_owner_dirty(&mut self) {
        if let Some(provider) = layout_node_provider::from(self.base.parent_mut()) {
            provider.mark_layout_node_dirty(false);
        }
    }
    pub fn grid_column_changed(&mut self) {
        self.mark_owner_dirty();
    }
    pub fn grid_row_changed(&mut self) {
        self.mark_owner_dirty();
    }
    pub fn grid_column_span_changed(&mut self) {
        self.mark_owner_dirty();
    }
    pub fn grid_row_span_changed(&mut self) {
        self.mark_owner_dirty();
    }
}
