use crate::mechanical_port::source::layout::layout_style_applier::YGStyle;
use crate::mechanical_port::source::{
    component::ComponentDirt,
    container_component::ContainerComponent,
    core_context::{CoreContext, StatusCode},
    generated::layout::grid_item_placement_base::GridItemPlacementBase,
    layout::{
        grid_track::GridTrack,
        layout_node_provider,
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
};

impl std::ops::Deref for GridItemPlacement {
    type Target = GridItemPlacementBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GridItemPlacement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl GridItemPlacement {
    pub const TYPE_KEY: u16 = GridItemPlacementBase::TYPE_KEY;
}

#[derive(Default)]
pub struct GridItemPlacement {
    pub base: GridItemPlacementBase,
}
impl GridItemPlacement {
    pub fn from(
        owner: Option<&ContainerComponent>,
    ) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        owner?
            .children()
            .iter()
            .find(|child| {
                child
                    .is_type_of(crate::mechanical_port::source::generated::layout::grid_item_placement_base::GridItemPlacementBase::TYPE_KEY)
            })
            .cloned()
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        if let Some(this) = self.base.handle()
            && let Some(parent) = self.base.parent_handle()
        {
            layout_node_provider::with_mut(&parent, |provider| {
                provider.add_layout_style_applier(this)
            });
        }
        StatusCode::Ok
    }
    pub fn update(&mut self, _value: ComponentDirt) {}
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
    }
    pub fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        if context.parent_is_stack || !context.parent_is_grid {
            return;
        }
        GridTrack::sync_item_lines(
            style,
            i32::from(self.base.grid_column()),
            i32::from(self.base.grid_row()),
            u32::from(self.base.grid_column_span()),
            u32::from(self.base.grid_row_span()),
        );
    }
    fn mark_owner_dirty(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            layout_node_provider::with_mut(&parent, |provider| {
                provider.mark_layout_node_dirty(false)
            });
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

impl LayoutStyleApplier for GridItemPlacement {
    fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        GridItemPlacement::apply_item_style(self, style, context);
    }
}
