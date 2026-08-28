use crate::mechanical_port::source::layout::layout_style_applier::{
    YGDimension, YGDisplay, YGJustify, YGStyle, YGUnit, YGValue,
};
use crate::mechanical_port::source::{
    generated::{
        component_base::ComponentBaseCallbacks,
        layout::layout_sizing_style_base::{LayoutSizingStyleBase, LayoutSizingStyleBaseCallbacks},
    },
    layout::{
        grid_track::GridTrack,
        layout_enums::LayoutScaleType,
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
};

pub struct LayoutSizingStyle {
    pub base: LayoutSizingStyleBase,
}

impl LayoutStyleApplier for LayoutSizingStyle {
    fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutSizingStyle::apply_base_style(self, style, context);
    }

    fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutSizingStyle::apply_item_style(self, style, context);
    }
}

impl LayoutSizingStyleBaseCallbacks for LayoutSizingStyle {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl ComponentBaseCallbacks for LayoutSizingStyle {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}
impl LayoutSizingStyle {
    pub fn apply_sizing_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        self.apply_base_style(style, context);
    }

    pub fn apply_sizing_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        self.apply_item_style(style, context);
    }

    pub fn display(&self) -> YGDisplay {
        if YGDisplay::from(self.base.display_value()) == YGDisplay::None {
            YGDisplay::None
        } else {
            YGDisplay::Flex
        }
    }
    pub fn max_width_units(&self) -> YGUnit {
        YGUnit::from(self.base.max_width_units_value())
    }
    pub fn max_height_units(&self) -> YGUnit {
        YGUnit::from(self.base.max_height_units_value())
    }
    pub fn min_width_units(&self) -> YGUnit {
        YGUnit::from(self.base.min_width_units_value())
    }
    pub fn min_height_units(&self) -> YGUnit {
        YGUnit::from(self.base.min_height_units_value())
    }
    pub fn apply_base_style(&self, style: &mut YGStyle, _context: &LayoutSyncContext) {
        style.set_display(self.display());
        style.min_dimensions_mut()[YGDimension::Width] =
            YGValue::new(self.base.min_width(), self.min_width_units());
        style.min_dimensions_mut()[YGDimension::Height] =
            YGValue::new(self.base.min_height(), self.min_height_units());
        style.max_dimensions_mut()[YGDimension::Width] =
            YGValue::new(self.base.max_width(), self.max_width_units());
        style.max_dimensions_mut()[YGDimension::Height] =
            YGValue::new(self.base.max_height(), self.max_height_units());
    }
    pub fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        if context.parent_is_stack {
            GridTrack::sync_stack_item_cell(style);
        }
        GridTrack::sync_item_justify_self(
            style,
            self.base.justify_self_value(),
            context.parent_is_stack,
            context.inline_hugs,
            context.container_justify_items,
        );
        if context.parent_is_grid
            && self.base.layout_width_scale_type() == LayoutScaleType::Fill as u32
        {
            style.set_justify_self(YGJustify::Stretch);
        }
    }
}
